use serde::Serialize;

use crate::command::*;
use crate::common::*;
use std::io;
use std::{
    io::{BufRead, BufReader, BufWriter, Write},
    net::TcpStream,
};

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
    /// - `name`: specifies an explicit server name as in isabelle server
    /// - `port`: specifies an explicit server port as in isabelle server.
    pub fn connect(name: &str, port: u32, pass: &str) -> Self {
        let addr = format!("{}:{}", name, port);

        Self {
            addr,
            pass: pass.to_owned(),
        }
    }

    fn handshake(&self, stream: &TcpStream) -> Result<(), io::Error> {
        let mut writer = BufWriter::new(stream.try_clone().unwrap());
        let mut reader = BufReader::new(stream.try_clone().unwrap());

        writer.write(format!("{}\n", self.pass).as_bytes())?;
        writer.flush();

        if let Ok(Some(e)) = stream.take_error() {
            panic!("Invalid password {}", e);
        }

        let mut res = String::new();
        reader.read_line(&mut res);
        log::info!("Handshake result: {}", res.trim());
        if !res.starts_with("OK") {
            return Err(io::Error::new(
                io::ErrorKind::ConnectionReset,
                "Invalid password".to_owned(),
            ));
        }
        Ok(())
    }

    fn parse_response<T: serde::de::DeserializeOwned>(&self, res: &str) -> Result<T, io::Error> {
        log::info!("Parsing response: {}", res);
        match serde_json::from_str::<T>(&res) {
            Ok(r) => Ok(r),
            Err(e) => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{}: {}", e.to_string(), res),
            )),
        }
    }

    async fn dispatch_async<
        T: Serialize,
        R: serde::de::DeserializeOwned,
        E: serde::de::DeserializeOwned,
        F: serde::de::DeserializeOwned,
    >(
        &self,
        cmd: Command<T>,
    ) -> Result<AsyncResult<R, E, F>, io::Error> {
        log::info!("Staring async command");
        let con = TcpStream::connect(&self.addr)?;
        let mut writer = BufWriter::new(con.try_clone().unwrap());
        let mut reader = BufReader::new(con.try_clone().unwrap());

        let args = match cmd.args {
            Args::Json(t) => serde_json::to_string(&t)?,
            Args::String(s) => serde_json::to_string(&s)?,
            Args::None => "".to_owned(),
        };

        self.handshake(&con)?;
        let cmdstring = format!("{} {}\n", cmd.name, args);
        log::info!("Dispatching command: {}", cmdstring.trim());
        writer.write(cmdstring.as_bytes())?;
        writer.flush()?;

        let mut res = String::new();
        reader.read_line(&mut res);
        log::info!("Got immediate result: {}", res);
        if res.trim().starts_with("OK") {
            let task: Task = self.parse_response(&res.strip_prefix("OK").unwrap().trim())?;
            log::info!("Got the task: {:?}", task);
            // log we go the task
        } else if res.trim().starts_with("ERROR") {
            let res = res.strip_prefix("ERROR").unwrap().trim().to_owned();
            let res = self.parse_response(&res)?;
            return Ok(AsyncResult::Error(res));
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unknown message format: {}", res),
            ));
        }
        // Wait until finished or failed, collect notes in between
        loop {
            res.clear();
            reader.read_line(&mut res);
            let res = res.trim();
            log::info!("Read: {}", res);
            if res.starts_with("FINISHED") {
                let parsed = self.parse_response(&res)?;
                return Ok(AsyncResult::Finished(parsed));
            } else if res.starts_with("FAILED") {
                let parsed = self.parse_response(&res)?;
                return Ok(AsyncResult::Failed(parsed));
            } else if res.starts_with("NOTE") {
                // handle note
            } else {
                log::warn!("Unknown message format: {}", res);
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
        let con = TcpStream::connect(&self.addr)?;
        let mut writer = BufWriter::new(con.try_clone().unwrap());
        let mut reader = BufReader::new(con.try_clone().unwrap());

        let args = match cmd.args {
            Args::Json(t) => serde_json::to_string(&t)?,
            Args::String(s) => serde_json::to_string(&s)?,
            Args::None => "".to_owned(),
        };

        self.handshake(&con)?;
        let cmdstring = format!("{} {}\n", cmd.name, args);
        writer.write(cmdstring.as_bytes())?;
        writer.flush();

        let mut res = String::new();
        reader.read_line(&mut res);
        if res.trim().starts_with("OK") {
            let res = self.parse_response(&res.strip_prefix("OK").unwrap().trim())?;
            Ok(SyncResult::Ok(res))
        } else if res.trim().starts_with("ERROR") {
            let res = res.strip_prefix("ERROR").unwrap().trim().to_owned();
            let res = self.parse_response(&res)?;
            Ok(SyncResult::Error(res))
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unknown message format: {}", res),
            ))
        }
    }

    pub async fn echo(&mut self, echo: &str) -> Result<SyncResult<String, String>, io::Error> {
        let cmd = Command {
            name: "echo".to_owned(),
            args: Args::Json(echo.to_owned()),
        };

        self.dispatch_sync(cmd).await
    }

    /// It forces a shut- down of the connected server process, stopping all open sessions and closing the server socket.
    /// This may disrupt pending commands on other connections.
    pub async fn shutdown(&mut self) -> Result<SyncResult<(), String>, io::Error> {
        let cmd: Command<()> = Command {
            name: "shutdown".to_owned(),
            args: Args::None,
        };
        self.dispatch_sync(cmd).await
    }

    pub async fn cancel(&mut self, task_id: String) -> Result<SyncResult<(), ()>, io::Error> {
        let cmd = Command {
            name: "cancel".to_owned(),
            args: Args::Json(CancelArgs { task: task_id }),
        };
        self.dispatch_sync(cmd).await
    }

    pub async fn session_build(
        &mut self,
        args: SessionBuildArgs,
    ) -> Result<AsyncResult<SessionBuildResults, (), SessionBuildResults>, io::Error> {
        let cmd = Command {
            name: "session_build".to_owned(),
            args: Args::Json(args),
        };

        self.dispatch_async(cmd).await
    }

    pub async fn session_start(
        &mut self,
        args: SessionStartArgs,
    ) -> Result<AsyncResult<SessionStartResult, (), ()>, io::Error> {
        let cmd = Command {
            name: "session_start".to_owned(),
            args: Args::Json(args),
        };

        self.dispatch_async(cmd).await
    }

    pub async fn session_stop(
        &mut self,
        args: SessionStopArgs,
    ) -> Result<AsyncResult<SessionStopResult, (), SessionStopResult>, io::Error> {
        let cmd = Command {
            name: "session_stop".to_owned(),
            args: Args::Json(args),
        };

        self.dispatch_async(cmd).await
    }

    pub async fn use_theories(
        &mut self,
        args: UseTheoryArgs,
    ) -> Result<AsyncResult<UseTheoryResults, (), ()>, io::Error> {
        let cmd = Command {
            name: "use_theories".to_owned(),
            args: Args::Json(args),
        };

        self.dispatch_async(cmd).await
    }

    pub async fn purge_theories(
        &mut self,
        args: PurgeTheoryArgs,
    ) -> Result<AsyncResult<PurgeTheoryResult, (), ()>, io::Error> {
        let cmd = Command {
            name: "purge_theories".to_owned(),
            args: Args::Json(args),
        };

        self.dispatch_async(cmd).await
    }
}
