# Isabelle Client Library

A complete implementation of an Isabelle client in Rust, along with facilities for starting Isabelle servers and running the Isabelle process in batch mode.

Refer to the [Isabelle System Manual](https://isabelle.in.tum.de/dist/Isabelle2022/doc/system.pdf) for further information.

## Key Features

- Use a TCP client to asynchronously interact with Isabelle servers
- Connect to running server instances or start new local servers
- Automatically fetch the password of servers running locally
- Run the Isabelle process in batch mode

## Installation

To use this library, simply include it as a dependency in your project's Cargo.toml file.

```toml
[dependencies]
isabelle = "0.1"
```

or run `cargo add isabelle-client` in your project root.

## Usage

### Client

To connect to an Isabelle server, first create an instance of the client by calling the `IsabelleClient::connect` method.
The client implements a method for various commands supported by the Isabelle server. Currently, the following commands are supported

- `echo`
- `shutdown`
- `cancel`
- `session_build`
- `session_start`
- `session_stop`
- `use_theories`
- `purger_theories`

The corresponding methods are `async`, that is, you need to call `await` to wait until execution finishes and to obtain the result.
The synchronous commands (`echo`, `shutdown`, `cancel`, and `purge_theories`) usually terminate immediately.
They return a `SyncResult` which indicates whether the Isabelle run the command successfully or not, and contains the result.
The asynchronous command (`session_build`, `session_start`, `session_stop`, and `use_theories`) spawn a new task on the server.
The client waits for that task to terminate and returns an `AsyncResult` containing the result.

Here is an example:

```rust
use isabelle_client::client::{AsyncResult, IsabelleClient, SyncResult};
use isabelle_client::client::commands::*;
use isabelle_client::server::run_server;

let addr = "127.0.0.1";
let port = 123456;
let password = "server_password"
// Connect to the server
let mut client = IsabelleClient::connect(Some(addr), port, password);

// Start session HOL
let session_args = SessionBuildArgs::session("HOL");
let session = client.session_build(&session_args).await.unwrap().unwrap();
// Load `Drinker` theory into the HOL session
let th_args = UseTheoriesArgs::for_session(&session.session_id, &["~~/src/HOL/Examples/Drinker"]);
let load_th = client.use_theories(&th_args).await.unwrap().unwrap();
// Assert loading theory was successful
assert!(load_th.ok)

// Shut down the server 
client.shutdown();
```

### Server

Use the `run_server` function to start an Isabelle server or obtain the information (port, password) of a locally running instance, if the name is known.

Here is an example for starting a server name "my-server".

```rust
use isabelle_client::server::run_server;

// Run an Isabelle server named "my-server" locally
let (port, password) = run_server(Some("my-server")).unwrap();
```

Note that this is just a wrapper for `isabelle server -n my-server`.
In particular, if a server named "my-server" is already running locally, the function will return the port and password of the existing server.

### Batch Mode

The `batch_process` function is a wrapper for asynchronously calling for `isabelle process`.
It takes a `ProcessArgs` as an argument which consists of

- The theories to load
- The session directories
- Optionally the logic session name, and
- Options, given key value pairs

The available options can be found in the system manual or using the `isabelle options` command.
The `OptionsBuilder` provides a convenient way to construct common options.

Here is an example:

```rust
use isabelle_client::process::{batch_process, ProcessArgs};

let args = ProcessArgs::load_theories(&[String::from("~~/src/HOL/Examples/Drinker")]);
let output = batch_process(&args, None).await;
assert!(output.unwrap().status.success());
```

## License

This library is licensed under the MIT license. See the LICENSE file for details.