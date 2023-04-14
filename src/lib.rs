pub mod event_repository;
pub mod metadata;
pub mod model_key;
pub mod state;
pub mod state_db;
pub mod state_repository;

const COMMAND_PREFIX: &str = "cmd";
const EVENT_PREFIX: &str = "evt";

use crate::state_db::StateDbError;
use eventstore::Error as EventStoreError;
use serde_json::Error as SerdeError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EventSourceError<S> {
    #[error("Cache error")]
    StateDbError(StateDbError),

    #[error("Event store error")]
    EventStore(EventStoreError),

    #[error("Serde error")]
    Serde(SerdeError),

    #[error("State error")]
    State(S),

    #[error("unknown cache db error")]
    Unknown,
}
