use crate::model_key::ModelKey;
use crate::state::State;
use crate::state_repository::StateWithInfo;
use std::marker::PhantomData;
use thiserror::Error;

pub trait StateDb<S>: Clone + Send
where
    S: State,
{
    fn get_from_db(&self, key: &ModelKey) -> Result<String, StateDbError>;
    fn set_in_db(&self, key: &ModelKey, state: String) -> Result<(), StateDbError>;

    fn get(&self, key: &ModelKey) -> Result<StateWithInfo<S>, StateDbError> {
        let data: Result<String, StateDbError> = self.get_from_db(key);

        match data {
            Ok(value) => Ok(serde_json::from_str(value.as_str()).unwrap_or_default()),
            Err(err) => Err(err),
        }
    }

    fn set(&self, key: &ModelKey, state: StateWithInfo<S>) -> Result<(), StateDbError> {
        let s = serde_json::to_string(&state).map_err(|_err| todo!())?;
        self.set_in_db(key, s)
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum StateDbError {
    #[error("Not found")]
    NotFound,

    #[error("data store disconnected `{0}`")]
    Disconnect(String),

    #[error("unknown cache db error")]
    Unknown,
}

#[derive(Clone)]
pub struct NoCache<S> {
    pub state: PhantomData<S>,
}

impl<S> NoCache<S> {
    pub fn new() -> Self {
        Self { state: PhantomData }
    }
}

impl<S> Default for NoCache<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> StateDb<S> for NoCache<S>
where
    S: State,
{
    fn get_from_db(&self, _key: &ModelKey) -> Result<String, StateDbError> {
        Ok("".to_string())
    }

    fn set_in_db(&self, _key: &ModelKey, _state: String) -> Result<(), StateDbError> {
        Ok(())
    }
}
