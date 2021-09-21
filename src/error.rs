use thiserror::Error;

#[derive(Error, Debug)]
pub enum KVStoreError {
    #[error("IO Error: `{0}`")]
    Io(#[from] std::io::Error),
    #[error("Error while serializing/deserializing: `{0}`")]
    Serde(#[from] serde_json::Error),
    #[error("Key `{0}` does not exist.")]
    KeyNotFound(String),
    #[error("`{0}` is not a valid action.")]
    InvalidAction(String)
}

pub type Result<T> = std::result::Result<T, KVStoreError>;
