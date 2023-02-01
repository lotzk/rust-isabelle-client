/// Contains the implementation of the Isabelle client.
///
/// Example usage:
///
///```rust
/// use isabelle_client::client::{AsyncResult, IsabelleClient, SyncResult};
/// use isabelle_client::client::commands::*;
/// use isabelle_client::server::run_server;
///
/// let (port, pw) = run_server(Some("Test")).unwrap();
/// let mut client = IsabelleClient::connect(None, port, &pw);
/// // Do something with the client
/// client.shutdown();
///```
pub mod client;
pub mod commands;
pub mod results;

pub use client::*;
