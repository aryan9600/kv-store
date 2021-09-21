pub mod store;
mod error;
pub use error::{KVStoreError, Result};
pub use store::KVStore;
