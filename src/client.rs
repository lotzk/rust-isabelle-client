use serde::Deserialize;
use serde::Serialize;

use crate::commands::*;
use crate::common::*;
use std::fmt::Display;
use std::io;
use std::{
    io::{BufRead, BufReader, BufWriter, Write},
    net::TcpStream,
};

struct Command<T: serde::Serialize> {
    pub name: String,
    pub args: Option<T>,
}

impl<T: serde::Serialize> Command<T> {
    /// Converts the command to a `\n`-terminated string the Isabelle server understands
    pub fn as_string(&self) -> String {
        let args = match &self.args {
            Some(arg) => serde_json::to_string(&arg).expect("Could not serialize"),
            None => "".to_owned(),
        };
        format!("{} {}\n", self.name, args)
    }

    /// Converts the command to a `\n`-terminated sequence of Bytes the Isabelle server understands
    pub fn as_bytes(&self) -> Vec<u8> {
        self.as_string().as_bytes().to_owned()
    }
}

impl<T: serde::Serialize> Display for Command<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_string().trim())
    }
}

#[derive(Debug)]
pub enum SyncResult<T, E> {
    Ok(T),
    Error(E),
}

#[derive(Debug)]
pub enum AsyncResult<T, F> {
    Error(Message),
    Finished(T),
    Failed(FailedResult<F>),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FailedResult<T> {
    #[serde(flatten)]
    task: Task,
    #[serde(flatten)]
    pub message: Message,

    #[serde(flatten)]
    pub context: Option<T>,
}

/// Provides interaction with Isabelle servers.
///
/// The Isabelle server listens on a TCP socket.
/// A command always produces a results.
/// Asynchronous commands return a task identifier indicating an working process that is joined later.
pub struct IsabelleClient {
    addr: String,
    pass: String,
}

impl IsabelleClient {
    /// Connect to an Isabelle server.
    /// The server name is sufficient for identification, as the client can determine the connection details from the local database of active servers.
    ///
    /// - `address`: specifies the server address, default is 127.0.0.1
    /// - `port`: specifies the server port
    pub fn connect(address: Option<&str>, port: u32, pass: &str) -> Self {
        let addr = format!("{}:{}", address.unwrap_or("127.0.0.1"), port);

        Self {
            addr,
            pass: pass.to_owned(),
        }
    }

    fn handshake(&self, stream: &TcpStream) -> Result<(), io::Error> {
        let mut writer = BufWriter::new(stream.try_clone().unwrap());
        let mut reader = BufReader::new(stream.try_clone().unwrap());

        writer.write_all(format!("{}\n", self.pass).as_bytes())?;
        writer.flush()?;

        if let Ok(Some(e)) = stream.take_error() {
            panic!("Invalid password {}", e);
        }

        let mut res = String::new();
        reader.read_line(&mut res)?;
        log::trace!("Handshake result: {}", res.trim());
        if !res.starts_with("OK") {
            return Err(io::Error::new(
                io::ErrorKind::ConnectionReset,
                "Invalid password".to_owned(),
            ));
        }
        Ok(())
    }

    fn parse_response<T: serde::de::DeserializeOwned>(
        &self,
        mut res: &str,
    ) -> Result<T, io::Error> {
        if res.is_empty() {
            // Workaround for json compliance, unit type is `null` not empty string
            res = "null";
        }
        log::trace!("Parsing response: '{}'", res);
        match serde_json::from_str::<T>(res) {
            Ok(r) => Ok(r),
            Err(e) => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{}: {}", e, res),
            )),
        }
    }

    /// Opens a new connection and performs the initial password exchange
    fn new_connection(&self) -> io::Result<(BufReader<TcpStream>, BufWriter<TcpStream>)> {
        let con = TcpStream::connect(&self.addr)?;

        // Perform password exchange
        self.handshake(&con)?;

        let writer = BufWriter::new(con.try_clone().unwrap());
        let reader = BufReader::new(con.try_clone().unwrap());

        Ok((reader, writer))
    }

    async fn dispatch_async<
        T: Serialize,
        R: serde::de::DeserializeOwned,
        F: serde::de::DeserializeOwned,
    >(
        &self,
        cmd: Command<T>,
    ) -> Result<AsyncResult<R, F>, io::Error> {
        let (mut reader, mut writer) = self.new_connection()?;

        log::trace!("Dispatching command: {}", cmd.as_string().trim());
        writer.write_all(&cmd.as_bytes())?;
        writer.flush()?;

        loop {
            // Loop here until either OK or ERROR is returned by the server
            // The server will sometimes produces output messages that should not happen, e.g., some random numbers
            let mut res = String::new();
            reader.read_line(&mut res)?;
            log::trace!("Got immediate result: {}", res);
            let res = res.trim();

            if let Some(ok_response) = res.strip_prefix("OK") {
                let task: Task = self.parse_response(ok_response.trim())?;
                log::trace!("Got the task: {:?}", task);
                break;
            } else if let Some(err_response) = res.strip_prefix("ERROR") {
                println!("hier");
                let res = self.parse_response(err_response.trim())?;
                return Ok(AsyncResult::Error(res));
            } else {
                log::trace!("Unknown message format: {}", res);
            }
        }

        // Wait until finished or failed, collect notes in between
        let mut res = String::new();
        loop {
            res.clear();
            reader.read_line(&mut res)?;
            let res = res.trim();
            log::trace!("Read: {}", res);
            if let Some(finish_response) = res.strip_prefix("FINISHED") {
                let parsed = self.parse_response(finish_response.trim())?;
                return Ok(AsyncResult::Finished(parsed));
            } else if let Some(failed_response) = res.strip_prefix("FAILED") {
                let parsed = self.parse_response(failed_response.trim())?;
                return Ok(AsyncResult::Failed(parsed));
            } else if let Some(note) = res.strip_prefix("NOTE") {
                // handle note
                log::trace!("{}", note);
            } else {
                log::trace!("Unknown message format: {}", res);
            }
        }
    }

    async fn dispatch_sync<
        T: Serialize,
        R: serde::de::DeserializeOwned,
        E: serde::de::DeserializeOwned,
    >(
        &self,
        cmd: Command<T>,
    ) -> Result<SyncResult<R, E>, io::Error> {
        let (mut reader, mut writer) = self.new_connection()?;

        log::trace!("Dispatching command: {}", cmd.as_string().trim());
        writer.write_all(&cmd.as_bytes())?;
        writer.flush()?;
        loop {
            let mut res = String::new();
            reader.read_line(&mut res)?;
            let res = res.trim();
            if let Some(response_ok) = res.strip_prefix("OK") {
                let res = self.parse_response(response_ok.trim())?;
                return Ok(SyncResult::Ok(res));
            } else if let Some(response_er) = res.strip_prefix("ERROR") {
                let res = self.parse_response(response_er.trim())?;
                return Ok(SyncResult::Error(res));
            } else {
                log::trace!("Unknown message format: {}", res);
            }
        }
    }

    pub async fn echo(&mut self, echo: &str) -> Result<SyncResult<String, String>, io::Error> {
        let cmd = Command {
            name: "echo".to_owned(),
            args: Some(echo.to_owned()),
        };

        self.dispatch_sync(cmd).await
    }

    /// It forces a shut- down of the connected server process, stopping all open sessions and closing the server socket.
    /// This may disrupt pending commands on other connections.
    pub async fn shutdown(&mut self) -> Result<SyncResult<(), String>, io::Error> {
        let cmd: Command<()> = Command {
            name: "shutdown".to_owned(),
            args: None,
        };
        self.dispatch_sync(cmd).await
    }

    pub async fn cancel(&mut self, task_id: String) -> Result<SyncResult<(), ()>, io::Error> {
        let cmd = Command {
            name: "cancel".to_owned(),
            args: Some(CancelArgs { task: task_id }),
        };
        self.dispatch_sync(cmd).await
    }

    pub async fn session_build(
        &mut self,
        args: &SessionBuildStartArgs,
    ) -> Result<AsyncResult<SessionBuildResults, SessionBuildResults>, io::Error> {
        let cmd = Command {
            name: "session_build".to_owned(),
            args: Some(args),
        };

        self.dispatch_async(cmd).await
    }

    pub async fn session_start(
        &mut self,
        args: &SessionBuildStartArgs,
    ) -> Result<AsyncResult<SessionStartResult, ()>, io::Error> {
        let cmd = Command {
            name: "session_start".to_owned(),
            args: Some(args),
        };

        self.dispatch_async(cmd).await
    }

    pub async fn session_stop(
        &mut self,
        args: &SessionStopArgs,
    ) -> Result<AsyncResult<SessionStopResult, SessionStopResult>, io::Error> {
        let cmd = Command {
            name: "session_stop".to_owned(),
            args: Some(args),
        };

        self.dispatch_async(cmd).await
    }

    pub async fn use_theories(
        &mut self,
        args: &UseTheoryArgs,
    ) -> Result<AsyncResult<UseTheoryResults, ()>, io::Error> {
        let cmd = Command {
            name: "use_theories".to_owned(),
            args: Some(args),
        };

        self.dispatch_async(cmd).await
    }

    pub async fn purge_theories(
        &mut self,
        args: PurgeTheoryArgs,
    ) -> Result<SyncResult<PurgeTheoryResults, ()>, io::Error> {
        let cmd = Command {
            name: "purge_theories".to_owned(),
            args: Some(args),
        };

        self.dispatch_sync(cmd).await
    }
}

#[cfg(test)]
mod test {

    use crate::client::{AsyncResult, IsabelleClient, SyncResult};
    use crate::commands::{SessionBuildStartArgs, SessionStopArgs, UseTheoryArgs};
    use crate::server::run_server;
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    async fn test_echo() {
        let (port, pw) = run_server(Some("Test")).unwrap();
        let mut client = IsabelleClient::connect(None, port, &pw);

        let res = client.echo("echo").await.unwrap();
        match res {
            SyncResult::Ok(r) => assert_eq!(r, "echo".to_owned()),
            SyncResult::Error(_) => unreachable!(),
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_shutdown() {
        let (port, pw) = run_server(Some("Test")).unwrap();
        let mut client = IsabelleClient::connect(None, port, &pw);

        let res = client.shutdown().await.unwrap();
        assert!(matches!(res, SyncResult::Ok(())));
    }

    #[tokio::test]
    #[serial]
    async fn test_session_build_hol() {
        let (port, pw) = run_server(Some("Test")).unwrap();
        let mut client = IsabelleClient::connect(None, port, &pw);

        let arg = SessionBuildStartArgs::session("HOL");

        let res = client.session_build(&arg).await.unwrap();
        match res {
            AsyncResult::Finished(res) => {
                assert!(res.ok);
                for s in res.sessions {
                    assert!(s.ok);
                    assert!(s.return_code == 0);
                }
            }
            AsyncResult::Failed(_) | AsyncResult::Error(_) => unreachable!(),
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_session_build_unknown() {
        let (port, pw) = run_server(Some("Test")).unwrap();
        let mut client = IsabelleClient::connect(None, port, &pw);

        let arg = SessionBuildStartArgs::session("unknown_sessions");

        let res = client.session_build(&arg).await.unwrap();

        assert!(matches!(res, AsyncResult::Failed(_)));
    }

    #[tokio::test]
    #[serial]
    async fn test_session_start_hol() {
        let (port, pw) = run_server(Some("Test")).unwrap();
        let mut client = IsabelleClient::connect(None, port, &pw);

        let arg = SessionBuildStartArgs::session("HOL");

        let res = client.session_start(&arg).await.unwrap();
        assert!(matches!(res, AsyncResult::Finished(_)));
    }

    #[tokio::test]
    #[serial]
    async fn test_session_start_unknown() {
        let (port, pw) = run_server(Some("Test")).unwrap();
        let mut client = IsabelleClient::connect(None, port, &pw);

        let arg = SessionBuildStartArgs::session("unknown_sessions");

        let res = client.session_start(&arg).await.unwrap();

        assert!(matches!(res, AsyncResult::Failed(_)));
    }

    #[tokio::test]
    #[serial]
    async fn test_session_stop_active() {
        let (port, pw) = run_server(Some("Test")).unwrap();
        let mut client = IsabelleClient::connect(None, port, &pw);

        let arg = SessionBuildStartArgs::session("HOL");
        let res = client.session_start(&arg).await.unwrap();
        if let AsyncResult::Finished(res) = res {
            let arg = SessionStopArgs {
                session_id: res.session_id,
            };
            if let AsyncResult::Finished(stop_res) = client.session_stop(&arg).await.unwrap() {
                assert!(stop_res.ok);
            } else {
                unreachable!();
            }
        } else {
            unreachable!()
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_session_stop_inactive() {
        let (port, pw) = run_server(Some("Test")).unwrap();
        let mut client = IsabelleClient::connect(None, port, &pw);
        let arg = SessionStopArgs {
            session_id: "03202b1a-bde6-4d84-926b-d435aac365fe".to_owned(),
        };
        let got = client.session_stop(&arg).await.unwrap();
        assert!(matches!(got, AsyncResult::Failed(_)));
    }

    #[tokio::test]
    #[serial]
    async fn test_session_stop_invalid() {
        let (port, pw) = run_server(Some("Test")).unwrap();
        let mut client = IsabelleClient::connect(None, port, &pw);
        let arg = SessionStopArgs {
            session_id: "abc".to_owned(),
        };
        let got = client.session_stop(&arg).await.unwrap();
        assert!(matches!(got, AsyncResult::Error(_)));
    }

    #[tokio::test]
    #[serial]
    async fn use_theory_in_hol() {
        let (port, pw) = run_server(Some("Test")).unwrap();
        let mut client = IsabelleClient::connect(None, port, &pw);

        let arg = SessionBuildStartArgs::session("HOL");
        let res = client.session_start(&arg).await.unwrap();
        if let AsyncResult::Finished(res) = res {
            let arg = UseTheoryArgs::for_session(&res.session_id, &["~~/src/HOL/Examples/Drinker"]);

            match client.use_theories(&arg).await.unwrap() {
                AsyncResult::Error(e) => unreachable!("{:?}", e),
                AsyncResult::Finished(got) => assert!(got.ok),
                AsyncResult::Failed(f) => unreachable!("{:?}", f),
            }
        } else {
            unreachable!()
        }
    }

    #[tokio::test]
    #[serial]
    async fn use_theory_unknown() {
        let (port, pw) = run_server(Some("Test")).unwrap();
        let mut client = IsabelleClient::connect(None, port, &pw);

        let arg = SessionBuildStartArgs::session("HOL");
        let res = client.session_start(&arg).await.unwrap();
        if let AsyncResult::Finished(res) = res {
            let arg = UseTheoryArgs::for_session(&res.session_id, &["~~/src/HOL/foo"]);
            let got = client.use_theories(&arg).await.unwrap();

            assert!(matches!(got, AsyncResult::Failed(_)));
        } else {
            unreachable!()
        }
    }

    #[tokio::test]
    #[serial]
    #[ignore]
    async fn use_theory_invalid() {
        //TODO: Try to load a theory with an invalid proof
    }
}
