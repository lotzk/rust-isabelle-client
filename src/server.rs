use std::{
    io::{self, BufRead, BufReader},
    process::{Command, ExitStatus, Stdio},
};

/// A running Isabelle server instance.
pub struct IsabelleServer {
    handle: Option<std::process::Child>,
    port: u32,
    passwd: String,
    name: String,
}

impl IsabelleServer {
    /// Returns the port of the running server instance.
    pub fn port(&self) -> u32 {
        self.port
    }

    /// Returns the password of the running server instance.
    pub fn password(&self) -> &str {
        &self.passwd
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Kills the running server instance, if it was started by this process.
    pub fn exit(&mut self) -> io::Result<()> {
        exit(&self.name)?;

        // Wait for the Child to terminate
        if let Some(mut handle) = self.handle.take() {
            if let Ok(None) = handle.try_wait() {
                handle.kill()?;
            }
        }

        Ok(())
    }
}

/// Runs the Isabelle server and returns an [IsabelleServer] instance containing port and password.
/// If a server is already running with the given name, the function returns the port and password of the instance.
/// If no server is running with the given name, the function starts a new server.
/// The server not shut down, even if this terminates.
/// To stop it, you need to call [IsabelleServer::exit] or [exit].
///
/// # Arguments
///
/// - `name` - The name of the server instance (default is `isabelle`).
///
/// # Returns
///
/// An [IsabelleServer] instance containing name, port, and password.
///
///
/// # Example
///
/// ```rust
/// use isabelle_client::server::run_server;
/// let mut server = run_server(Some("test")).unwrap();
/// assert!(server.port() > 0);
/// assert!(!server.password().is_empty());
/// server.exit();
/// ```
pub fn run_server(name: Option<&str>) -> io::Result<IsabelleServer> {
    let name = name.unwrap_or("isabelle").to_string();
    let mut handle = Command::new("isabelle")
        .arg("server")
        .arg("-n")
        .arg(&name)
        .stdout(Stdio::piped())
        .spawn()?;

    let stdout = handle.stdout.take().unwrap();

    let mut stdout_buf = vec![];
    let newline = b'\n';
    // Read until newline
    BufReader::new(stdout).read_until(newline, &mut stdout_buf)?;

    let stdout_str = String::from_utf8(stdout_buf)
        .unwrap()
        .replace('\\', "")
        .trim()
        .to_string();

    let addr_re = regex::Regex::new(r#".* = .*:(.*) \(password "(.*)"\)"#).unwrap();
    let caps = addr_re.captures(&stdout_str).unwrap();

    let port = caps.get(1).unwrap().as_str().parse::<u32>().unwrap();
    let passwd = caps.get(2).unwrap().as_str().to_owned();

    let server = if handle.try_wait()?.is_none() {
        IsabelleServer {
            handle: Some(handle),
            port,
            passwd,
            name,
        }
    } else {
        IsabelleServer {
            handle: None,
            port,
            passwd,
            name,
        }
    };

    Ok(server)
}

/// Exists the Isabelle server with the given name.
pub fn exit(name: &str) -> io::Result<ExitStatus> {
    let mut child = Command::new("isabelle")
        .arg("server")
        .arg("-n")
        .arg(name)
        .arg("-x")
        .spawn()?;
    child.wait()
}

mod tests {

    #![allow(unused_imports)] // rust-analyzer thinks these are unused, but are not
    use super::run_server;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_run_server() {
        let mut server = run_server(Some("test")).unwrap();
        assert!(server.port > 0);
        assert!(!server.passwd.is_empty());
        server.exit().unwrap();
    }
}
