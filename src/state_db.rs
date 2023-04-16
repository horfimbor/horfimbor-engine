use crate::model_key::ModelKey;
use crate::repository::StateWithInfo;
use crate::State;
use thiserror::Error;

pub trait StateDb<S>: Clone + Send
where
    S: State,
{
    fn get_from_db(&self, key: &ModelKey) -> Result<Option<String>, StateDbError>;
    fn set_in_db(&self, key: &ModelKey, state: String) -> Result<(), StateDbError>;

    fn get(&self, key: &ModelKey) -> Result<StateWithInfo<S>, StateDbError> {
        let data = self.get_from_db(key);

        match data {
            Ok(None) => Ok(StateWithInfo::default()),
            Ok(Some(value)) => Ok(serde_json::from_str(value.as_str()).unwrap_or_default()),
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

    #[error("internal `{0}`")]
    Internal(String),
}
