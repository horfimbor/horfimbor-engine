use std::marker::PhantomData;

use redis::{Client, Commands};

use crate::cache_db::{CacheDb, CacheDbError};
use crate::model_key::ModelKey;
use crate::Dto;

#[derive(Clone)]
pub struct RedisStateDb<S> {
    client: Client,
    state: PhantomData<S>,
}

impl<S> RedisStateDb<S> {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            state: PhantomData,
        }
    }
}

impl<S> CacheDb<S> for RedisStateDb<S>
where
    S: Dto,
{
    fn get_from_db(&self, key: &ModelKey) -> Result<Option<String>, CacheDbError> {
        let mut connection = self
            .client
            .get_connection()
            .map_err(|e| CacheDbError::Disconnect(e.to_string()))?;

        let data: Option<String> = connection
            .get(key.format())
            .map_err(|e| CacheDbError::Internal(e.to_string()))?;

        Ok(data)
    }

    fn set_in_db(&self, key: &ModelKey, state: String) -> Result<(), CacheDbError> {
        let mut connection = self
            .client
            .get_connection()
            .map_err(|e| CacheDbError::Disconnect(e.to_string()))?;

        connection
            .set(key.format(), state)
            .map_err(|err| CacheDbError::Internal(err.to_string()))?;

        Ok(())
    }
}
