mod client;
pub mod commands;
pub mod common;
pub mod server;

pub use client::{AsyncResult, IsabelleClient, SyncResult};
pub use commands::*;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
