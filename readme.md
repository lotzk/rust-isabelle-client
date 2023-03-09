# Isabelle Client Library

An implementation of the Isabelle client in Rust, along with facilities for starting Isabelle servers and running the Isabelle process in batch mode.

Refer to the [Isabelle System Manual](https://isabelle.in.tum.de/dist/Isabelle2022/doc/system.pdf) for further information.

## Key Features

- TCP client for async interactions with Isabelle servers
- Local Isabelle server startup and credential retrieval
- Raw Isabelle process batch mode wrapper

## Installation

To use this library, simply include it as a dependency in your project's Cargo.toml file.

```toml
[dependencies]
isabelle-client = "0.1.0"
```

or run `cargo add isabelle-client` in your project root.

## Usage

To use the server or batch utilities, an Isabelle installation is required.

### Client

To connect to an Isabelle server, first create an instance of the client by calling the `IsabelleClient::connect` method.
The client implement methods for various commands supported by the Isabelle server. Currently, the following commands are supported

- `echo`
- `shutdown`
- `cancel`
- `session_build`
- `session_start`
- `session_stop`
- `use_theories`
- `purger_theories`

All methods are `async` and an `await` call is required to wait until execution finishes and to obtain the result.
The synchronous commands (`echo`, `shutdown`, `cancel`, and `purge_theories`) usually terminate immediately.
They return a `SyncResult` which indicates whether the Isabelle run the command successfully or not, and contains the result.
The asynchronous commands (`session_build`, `session_start`, `session_stop`, and `use_theories`) spawn a new task on the server.
The client waits for that task to terminate and returns an `AsyncResult` containing the result.

Here is an example:

```rust
use isabelle_client::client::{AsyncResult, IsabelleClient, SyncResult};
use isabelle_client::client::args::*;
use isabelle_client::server::run_server;
use tokio_test::block_on;

// Start a server and connect to it
let mut server = run_server(Some("test-server")).unwrap();
let mut client = IsabelleClient::for_server(&server);

// Start session HOL
let session_args = SessionBuildArgs::session("HOL");
let session = block_on(client.session_start(&session_args)).unwrap();
// Load `Drinker` theory into the HOL session
let th_args = UseTheoriesArgs::for_session(&session.finished().session_id, &["~~/src/HOL/Examples/Drinker"]);
let load_th = block_on(client.use_theories(&th_args)).unwrap();
// Assert loading theory was successful
assert!(load_th.finished().ok);

// Exit the server 
server.exit();
```

### Server

Use the `run_server` function to start an Isabelle server or obtain the credentials (port, password) of a locally running instance, if the name is known.
A running server can be exited using the `exit` method.

Here is an example for starting a server name "my-server".

```rust
use isabelle_client::server::run_server;

// Run an Isabelle server named "my-server" locally
let mut server = run_server(Some("my-server")).unwrap();

// ...

// Exit the server
server.exit();

```

Note that this is just a wrapper for `isabelle server -n my-server`.
In particular, if a server named "my-server" is already running locally, the function will return the port and password of the existing server.

### Batch Mode

The `batch_process` function is a wrapper for asynchronously calling the `isabelle process` tool.
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
use tokio_test::block_on;

let args = ProcessArgs::load_theories(&[String::from("~~/src/HOL/Examples/Drinker")]);
let output = block_on(batch_process(&args, None));
assert!(output.unwrap().status.success());
```

## License

This library is licensed under the MIT license. See the LICENSE file for details.
