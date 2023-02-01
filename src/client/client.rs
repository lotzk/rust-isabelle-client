use serde::Deserialize;
use serde::Serialize;

use super::commands::*;
use super::results::*;
use std::fmt::Display;
use std::io;
use std::{
    io::{BufRead, BufReader, BufWriter, Write},
    net::TcpStream,
};

/// A command to be sent to the Isabelle server.
/// It consists of a `name` and optional arguments `args` which are serialized as JSON.
struct Command<T: serde::Serialize> {
    pub name: String,
    pub args: Option<T>,
}

impl<T: serde::Serialize> Command<T> {
    /// Converts the command to a `\n`-terminated string the Isabelle server understands
    fn as_string(&self) -> String {
        let args = match &self.args {
            Some(arg) => serde_json::to_string(&arg).expect("Could not serialize"),
            None => "".to_owned(),
        };
        format!("{} {}\n", self.name, args)
    }

    /// Converts the command to a `\n`-terminated sequence of Bytes the Isabelle server understands
    fn as_bytes(&self) -> Vec<u8> {
        self.as_string().as_bytes().to_owned()
    }
}

impl<T: serde::Serialize> Display for Command<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_string().trim())
    }
}

/// Result of a synchronous command sent to the Isabelle server.
#[derive(Debug)]
pub enum SyncResult<T, E> {
    /// If the command was successful (server returned `OK`), contains the result value of type `T`.
    Ok(T),
    /// If the command was unsuccessful (server returned `ERROR`), contains an error value of type `E`.
    Error(E),
}

impl<T, E> SyncResult<T, E> {
    pub fn unwrap(&self) -> &T {
        match self {
            SyncResult::Ok(t) => t,
            SyncResult::Error(_) => panic!("Called unwrap on error value"),
        }
    }
}

/// Result of an asynchronous command sent to the Isabelle server.
#[derive(Debug)]
pub enum AsyncResult<T, F> {
    /// If the task associated with the command was successful (server returned `FINISHED`), contains the result value of type `T`.
    Finished(T),
    /// If the task associated with the command failed (server returned `FAILED`), contains a [FailedResult] value of type `F`.
    Failed(FailedResult<F>),
    /// If the async command fails immediately, contains the message
    Error(Message),
}

impl<T, F> AsyncResult<T, F> {
    pub fn unwrap(&self) -> &T {
        match self {
            AsyncResult::Finished(t) => t,
            AsyncResult::Failed(_) => panic!("Called unwrap on Failed result"),
            AsyncResult::Error(_) => panic!("Called unwrap on Error result"),
        }
    }
}

/// Result of a failed asynchronous task.
#[derive(Serialize, Deserialize, Debug)]
pub struct FailedResult<T> {
    /// Task identifier
    #[serde(flatten)]
    task: Task,
    /// Information about the error as returned from the server
    #[serde(flatten)]
    pub message: Message,
    /// Context information returned from the server
    #[serde(flatten)]
    pub context: Option<T>,
}

/// Provides interaction with Isabelle servers.
pub struct IsabelleClient {
    addr: String,
    pass: String,
}

impl IsabelleClient {
    /// Connect to an Isabelle server.
    ///
    /// - `address`: specifies the server address. If it is `None`, "127.0.0.1" is use as a default
    /// - `port`: specifies the server port
    /// - `pass`: the password
    pub fn connect(address: Option<&str>, port: u32, pass: &str) -> Self {
        let addr = format!("{}:{}", address.unwrap_or("127.0.0.1"), port);

        Self {
            addr,
            pass: pass.to_owned(),
        }
    }

    /// Performs the initial password exchange(i.e. password exchange) between a new client client and server.
    /// Returns a `Result` indicating the success or failure of the handshake.
    fn handshake(&self, stream: &TcpStream) -> io::Result<()> {
        let mut writer = BufWriter::new(stream.try_clone().unwrap());
        let mut reader = BufReader::new(stream.try_clone().unwrap());

        writer.write_all(format!("{}\n", self.pass).as_bytes())?;
        writer.flush()?;

        if let Some(e) = stream.take_error()? {
            return Err(e);
        }

        let mut res = String::new();
        reader.read_line(&mut res)?;
        log::trace!("Handshake result: {}", res.trim());
        if !res.starts_with("OK") {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Handshake failed",
            ));
        }
        log::trace!("Handshake ok");
        Ok(())
    }

    /// Facility to parse JSON responses from the Isabelle server into Rust types
    fn parse_response<T: serde::de::DeserializeOwned>(
        &self,
        mut res: &str,
    ) -> Result<T, io::Error> {
        if res.is_empty() {
            // Workaround for json compliance, unit type is `null` not empty string
            res = "null";
        }
        match serde_json::from_str::<T>(res) {
            Ok(r) => Ok(r),
            Err(e) => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{}: {}", e, res),
            )),
        }
    }

    /// Creates a new connection to the server and performs the initial password exchange
    /// handshake. Returns a tuple of buffered reader and writer wrapped around the TcpStream
    /// connection.
    fn new_connection(&self) -> io::Result<(BufReader<TcpStream>, BufWriter<TcpStream>)> {
        let con = TcpStream::connect(&self.addr)?;

        // Perform password exchange
        self.handshake(&con)?;

        let writer = BufWriter::new(con.try_clone().unwrap());
        let reader = BufReader::new(con.try_clone().unwrap());

        Ok((reader, writer))
    }

    /// Dispatches asynchronous [Command] `cmd` to start the task on the server.
    ///
    /// The method dispatches the `cmd` which starts an asynchronous task at the server.
    /// The method then waits for the task to finish or fail by reading the response and returns the result
    /// as an `AsyncResult<R, F>` where `R` is the type of the response when the task is finished and
    /// `F` is the type of the response when the task fails.
    ///
    /// Notes printed by the server are logged and cannot be accessed.
    ///
    /// Returns an `io::Error` if communication with the server failed.
    async fn dispatch_async<
        T: Serialize,
        R: serde::de::DeserializeOwned,
        F: serde::de::DeserializeOwned,
    >(
        &self,
        cmd: &Command<T>,
        reader: &mut BufReader<TcpStream>,
        writer: &mut BufWriter<TcpStream>,
    ) -> Result<AsyncResult<R, F>, io::Error> {
        // Dispatch the command as sync to start the task. Return Error if it failed
        if let SyncResult::Error(e) = self
            .dispatch_sync::<T, Task, Message>(&cmd, reader, writer)
            .await?
        {
            // Cast to async result
            return Ok(AsyncResult::Error(e));
        };

        // Wait for the task to finish or fail, and collect notes along the way
        let mut res = String::new();
        loop {
            res.clear();
            reader.read_line(&mut res)?;
            let res = res.trim();
            if let Some(finish_response) = res.strip_prefix("FINISHED") {
                // If the task has finished, parse the response
                let parsed = self.parse_response(finish_response.trim())?;
                return Ok(AsyncResult::Finished(parsed));
            } else if let Some(failed_response) = res.strip_prefix("FAILED") {
                // If the task has failed, parse the response
                let parsed = self.parse_response(failed_response.trim())?;
                return Ok(AsyncResult::Failed(parsed));
            } else if let Some(note) = res.strip_prefix("NOTE") {
                // If it's a note, log it and continue the loop
                log::trace!("{}", note);
            } else {
                // Occasionally the server omits some seemingly random numeric logs.
                // Log and discard them, then continue the loop.
                log::trace!("Unknown message format: {}", res);
            }
        }
    }

    /// Dispatches synchronous [Command] `cmd` to the server in and return the result.
    ///
    /// Sends the `cmd` to the server and reads the response, which is either "OK" or "ERROR".
    /// Returns the corresponding result wrapped in a [SyncResult] enum.
    ///
    /// Returns an `io::Error` if communication with the server failed.
    async fn dispatch_sync<
        T: Serialize,
        R: serde::de::DeserializeOwned,
        E: serde::de::DeserializeOwned,
    >(
        &self,
        cmd: &Command<T>,
        reader: &mut BufReader<TcpStream>,
        writer: &mut BufWriter<TcpStream>,
    ) -> Result<SyncResult<R, E>, io::Error> {
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
                // Occasionally the server omits some seemingly random numeric logs.
                // Log and discard them, then continue the loop.
                log::trace!("Unknown message format: {}", res);
            }
        }
    }

    /// Identity function: Returns its argument as result
    pub async fn echo(&mut self, echo: &str) -> Result<SyncResult<String, String>, io::Error> {
        let cmd = Command {
            name: "echo".to_owned(),
            args: Some(echo.to_owned()),
        };
        let (mut reader, mut writer) = self.new_connection()?;
        self.dispatch_sync(&cmd, &mut reader, &mut writer).await
    }

    /// Forces a shut- down of the connected server process, stopping all open sessions and closing the server socket.
    /// This may disrupt pending commands on other connections.
    pub async fn shutdown(&mut self) -> Result<SyncResult<(), String>, io::Error> {
        let cmd: Command<()> = Command {
            name: "shutdown".to_owned(),
            args: None,
        };
        let (mut reader, mut writer) = self.new_connection()?;
        self.dispatch_sync(&cmd, &mut reader, &mut writer).await
    }

    /// Attempts to cancel the specified task.
    /// Cancellation is merely a hint that the client prefers an ongoing process to be stopped.
    pub async fn cancel(&mut self, task_id: String) -> Result<SyncResult<(), ()>, io::Error> {
        let cmd = Command {
            name: "cancel".to_owned(),
            args: Some(CancelArgs { task: task_id }),
        };
        let (mut reader, mut writer) = self.new_connection()?;
        self.dispatch_sync(&cmd, &mut reader, &mut writer).await
    }

    /// Prepares a session image for interactive use of theories.
    pub async fn session_build(
        &mut self,
        args: &SessionBuildArgs,
    ) -> Result<AsyncResult<SessionBuildResults, SessionBuildResults>, io::Error> {
        let cmd = Command {
            name: "session_build".to_owned(),
            args: Some(args),
        };
        let (mut reader, mut writer) = self.new_connection()?;
        self.dispatch_async(&cmd, &mut reader, &mut writer).await
    }

    /// Starts a new Isabelle/PIDE session with underlying Isabelle/ML process, based on a session image that it produces on demand using `session_build`.
    /// Returns the `session_id`, which provides the internal identification of the session object within the server process.
    pub async fn session_start(
        &mut self,
        args: &SessionBuildArgs,
    ) -> Result<AsyncResult<SessionStartResult, ()>, io::Error> {
        let cmd = Command {
            name: "session_start".to_owned(),
            args: Some(args),
        };

        let (mut reader, mut writer) = self.new_connection()?;
        self.dispatch_async(&cmd, &mut reader, &mut writer).await
    }

    /// Forces a shutdown of the identified session.
    pub async fn session_stop(
        &mut self,
        args: &SessionStopArgs,
    ) -> Result<AsyncResult<SessionStopResult, SessionStopResult>, io::Error> {
        let cmd = Command {
            name: "session_stop".to_owned(),
            args: Some(args),
        };

        let (mut reader, mut writer) = self.new_connection()?;
        self.dispatch_async(&cmd, &mut reader, &mut writer).await
    }

    /// Updates the identified session by adding the current version of theory files to it, while dependencies are resolved implicitly.
    pub async fn use_theories(
        &mut self,
        args: &UseTheoriesArgs,
    ) -> Result<AsyncResult<UseTheoryResults, ()>, io::Error> {
        let cmd = Command {
            name: "use_theories".to_owned(),
            args: Some(args),
        };

        let (mut reader, mut writer) = self.new_connection()?;
        self.dispatch_async(&cmd, &mut reader, &mut writer).await
    }

    /// Updates the identified session by removing theories.
    /// Theories that are used in pending use_theories tasks or imported by other theories are retained.
    pub async fn purge_theories(
        &mut self,
        args: PurgeTheoryArgs,
    ) -> Result<SyncResult<PurgeTheoryResults, ()>, io::Error> {
        let cmd = Command {
            name: "purge_theories".to_owned(),
            args: Some(args),
        };

        let (mut reader, mut writer) = self.new_connection()?;
        self.dispatch_sync(&cmd, &mut reader, &mut writer).await
    }
}

#[cfg(test)]
mod test {

    use super::*;
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

        let arg = SessionBuildArgs::session("HOL");

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

        let arg = SessionBuildArgs::session("unknown_sessions");

        let res = client.session_build(&arg).await.unwrap();

        assert!(matches!(res, AsyncResult::Failed(_)));
    }

    #[tokio::test]
    #[serial]
    async fn test_session_start_hol() {
        let (port, pw) = run_server(Some("Test")).unwrap();
        let mut client = IsabelleClient::connect(None, port, &pw);

        let arg = SessionBuildArgs::session("HOL");

        let res = client.session_start(&arg).await.unwrap();
        assert!(matches!(res, AsyncResult::Finished(_)));
    }

    #[tokio::test]
    #[serial]
    async fn test_session_start_unknown() {
        let (port, pw) = run_server(Some("Test")).unwrap();
        let mut client = IsabelleClient::connect(None, port, &pw);

        let arg = SessionBuildArgs::session("unknown_sessions");

        let res = client.session_start(&arg).await.unwrap();

        assert!(matches!(res, AsyncResult::Failed(_)));
    }

    #[tokio::test]
    #[serial]
    async fn test_session_stop_active() {
        let (port, pw) = run_server(Some("Test")).unwrap();
        let mut client = IsabelleClient::connect(None, port, &pw);

        let arg = SessionBuildArgs::session("HOL");
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

        let arg = SessionBuildArgs::session("HOL");
        let res = client.session_start(&arg).await.unwrap();
        if let AsyncResult::Finished(res) = res {
            let arg =
                UseTheoriesArgs::for_session(&res.session_id, &["~~/src/HOL/Examples/Drinker"]);

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

        let arg = SessionBuildArgs::session("HOL");
        let res = client.session_start(&arg).await.unwrap();
        if let AsyncResult::Finished(res) = res {
            let arg = UseTheoriesArgs::for_session(&res.session_id, &["~~/src/HOL/foo"]);
            let got = client.use_theories(&arg).await.unwrap();

            assert!(matches!(got, AsyncResult::Failed(_)));
        } else {
            unreachable!()
        }
    }
}
