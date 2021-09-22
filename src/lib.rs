mod error;
pub mod store;
pub use error::{KVStoreError, Result};
pub use store::KVStore;
pub mod models;
