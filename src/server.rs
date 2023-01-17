use std::{
    io::{self, BufRead, BufReader},
    process::{Command, Stdio},
};

/// Runs the Isabelle server command to obtain port and password from a running server instance.
/// If no server is running, then this will start a new instance that will run locally.
/// If there a running Isabelle server, return the port and password from this server.
///
/// Use `IsabelleClient.shutdown()` to quit the server.
pub fn run_server() -> io::Result<(u32, String)> {
    let mut handle = Command::new("isabelle")
        .arg("server")
        .stdout(Stdio::piped())
        .spawn()?;

    let stdout = handle.stdout.take().unwrap();

    let mut stdout_buf = vec![];
    BufReader::new(stdout).read_until(10, &mut stdout_buf)?;

    let stdout_str = String::from_utf8(stdout_buf)
        .unwrap()
        .replace('\\', "")
        .trim()
        .to_string();

    let addr_re = regex::Regex::new(r#".* = .*:(.*) \(password "(.*)"\)"#).unwrap();
    let caps = addr_re.captures(&stdout_str).unwrap();

    let port = caps.get(1).unwrap().as_str().parse::<u32>().unwrap();
    let passwd = caps.get(2).unwrap().as_str().to_owned();

    log::info!("using Isabelle server on port: {}", port,);

    io::Result::Ok((port, passwd))
}
