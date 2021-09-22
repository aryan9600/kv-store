mod error;
pub mod store;
pub use error::{KVStoreError, Result};
pub use store::KVStore;
pub mod models;
pub mod pubsub;

#[derive(Debug, Clone, Default)]
pub struct ConnStrings {
    server_host: String,
    nats_host: String,
    log_file_path: String
}

impl ConnStrings {
    pub fn load() -> Self {
        let mut server_host = String::from("127.0.0.1:8000");
        if let Ok(val) = std::env::var("KVSTORE_HOST") {
            server_host = val;
        }
        let mut nats_host = String::from("127.0.0.1:4444");
        if let Ok(val) = std::env::var("KVSTORE_NATS_HOST") {
            nats_host = val;
        }
        let mut log_file_path = String::from("kvs.log");
        if let Ok(val) = std::env::var("KVSTORE_LOG_PATH") {
            log_file_path = val;
        }
        ConnStrings {
            server_host,
            nats_host,
            log_file_path
        }
    }

    pub fn server_host(&self) -> String {
        self.server_host.clone()
    }

    pub fn nats_host(&self) -> String {
        self.nats_host.clone()
    }
    
    pub fn log_file_path(&self) -> String {
        self.log_file_path.clone()
    }
}
