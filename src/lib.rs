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
    log_file_path: String,
}

const SERVER_HOST: &str = "http://127.0.0.1:8000";
const NATS_HOST: &str = "127.0.0.1:4444";
const LOG_FILE_PATH: &str = "kvs.log";

impl ConnStrings {
    // Try to load the strings from environment. Use specified defaults if not found.
    pub fn load() -> Self {
        dotenv::dotenv().ok();
        let mut server_host = String::from(SERVER_HOST);
        if let Ok(val) = std::env::var("KVSTORE_SERVER_HOST") {
            server_host = val;
        }
        let mut nats_host = String::from(NATS_HOST);
        if let Ok(val) = std::env::var("KVSTORE_NATS_HOST") {
            nats_host = val;
        }
        let mut log_file_path = String::from(LOG_FILE_PATH);
        if let Ok(val) = std::env::var("KVSTORE_LOG_FILE_PATH") {
            log_file_path = val;
        }
        ConnStrings {
            server_host,
            nats_host,
            log_file_path,
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
