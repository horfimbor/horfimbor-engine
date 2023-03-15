use crate::model_key::ModelKey;
use thiserror::Error;

pub trait CacheDb : Clone {
    fn get(&self, key: &ModelKey) -> Result<String, CacheError>;
}

#[derive(Error, Debug, PartialEq)]
pub enum CacheError {
    #[error("Not found")]
    NotFound,

    #[error("data store disconnected `{0}`")]
    Disconnect(String),

    #[error("unknown cache db error")]
    Unknown,
}

#[derive(Clone)]
pub struct NoCache {}

impl CacheDb for NoCache {
    fn get(&self, _: &ModelKey) -> Result<String, CacheError> {
        Err(CacheError::NotFound)
    }
}
