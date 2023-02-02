/// Contains the implementation of the Isabelle client.
///
/// Example usage:
///
///```rust
/// use isabelle_client::client::{AsyncResult, IsabelleClient, SyncResult};
/// use isabelle_client::client::commands::*;
/// use isabelle_client::server::run_server;
///
/// let mut server = run_server(Some("test")).unwrap();
/// let mut client = IsabelleClient::for_server(&server);
/// // Do something with the client
/// // ...
/// // Kill the server when done
/// server.exit();
///```
pub mod client;
pub mod commands;
pub mod results;

pub use client::*;
