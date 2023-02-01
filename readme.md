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

### Client and Server

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

#### Example

```rust
use isabelle_client::client::{AsyncResult, IsabelleClient, SyncResult};
use isabelle_client::client::commands::*;
use isabelle_client::server::run_server;

// Start a new Isabelle server locally, returning the port and the password
let (port, pw) = run_server(Some("Test")).unwrap();

// Connect to the server
let mut client = IsabelleClient::connect(None, port, &pw);

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

To start an Isabelle server, create an instance of the server and then call the start method. This will launch a new thread for the server and return a handle for the thread.

#### Example

```rust

```

### Batch Mode

To run Isabelle in batch mode, use the run_batch function. This function takes the path to the Isabelle installation and the batch mode options, and returns a Result type with the output of the process.

#### Example

```rust

```

## License

This library is licensed under the MIT license. See the LICENSE file for details.