use std::{
    io::{self, BufRead, BufReader},
    process::{Command, Stdio},
};

/// Runs the Isabelle server command to obtain port and password from a running server instance.
/// If no name is given, the default (`isabelle`) is used.
/// If there is a server running with the given name, function will return the port and password of the instance.
/// If there is no server running with the given name, the function starts a new server.
///
/// If the server was created by this function, it will be terminated once the program exists.
/// If it connected to an already running instance, it won't be terminated after the program exists, but you can use `IsabelleClient.shutdown()` to shut down the server.
pub fn run_server(name: Option<&str>) -> io::Result<(u32, String)> {
    let mut handle = Command::new("isabelle")
        .arg("server")
        .arg("-n")
        .arg(name.unwrap_or("isabelle"))
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

    log::trace!("Using Isabelle server on port: {}", port);

    io::Result::Ok((port, passwd))
}
