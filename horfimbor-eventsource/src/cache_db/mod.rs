use std::marker::PhantomData;

use thiserror::Error;

use crate::model_key::ModelKey;
use crate::repository::ModelWithPosition;
use crate::Dto;

#[cfg(feature = "cache-redis")]
pub mod redis;

pub trait CacheDb<S>: Clone + Send + Sync
where
    S: Dto,
{
    /// # Errors
    ///
    /// Will return `Err` if any error append when calling the DB.
    fn get_from_db(&self, key: &ModelKey) -> Result<Option<String>, DbError>;

    /// # Errors
    ///
    /// Will return `Err` if any error append when calling the DB.
    fn set_in_db(&self, key: &ModelKey, state: String) -> Result<(), DbError>;

    /// # Errors
    ///
    /// Will return `Err` if any error append when calling the DB.
    fn get(&self, key: &ModelKey) -> Result<ModelWithPosition<S>, DbError> {
        let data = self.get_from_db(key);

        match data {
            Ok(None) => Ok(ModelWithPosition::default()),
            Ok(Some(value)) => Ok(serde_json::from_str(value.as_str()).unwrap_or_default()),
            Err(err) => Err(err),
        }
    }

    /// # Errors
    ///
    /// Will return `Err` if any error append when calling the DB.
    fn set(&self, key: &ModelKey, data: ModelWithPosition<S>) -> Result<(), DbError> {
        let s = serde_json::to_string(&data)
            .map_err(|_err| todo!("error in StateDb.set is not handled yet"))?;
        self.set_in_db(key, s)
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum DbError {
    #[error("Not found")]
    NotFound,

    #[error("data store disconnected `{0}`")]
    Disconnect(String),

    #[error("unknown cache db error")]
    Unknown,

    #[error("internal `{0}`")]
    Internal(String),
}

#[derive(Clone)]
pub struct NoCache<S> {
    state: PhantomData<S>,
}

impl<S> NoCache<S> {
    #[must_use]
    pub const fn new() -> Self {
        Self { state: PhantomData }
    }
}

impl<S> Default for NoCache<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> CacheDb<S> for NoCache<S>
where
    S: Dto,
{
    fn get_from_db(&self, _key: &ModelKey) -> Result<Option<String>, DbError> {
        Ok(None)
    }

    fn set_in_db(&self, _key: &ModelKey, _state: String) -> Result<(), DbError> {
        Ok(())
    }
}
