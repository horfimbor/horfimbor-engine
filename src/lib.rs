/// re-export import :
pub use serde;

pub mod metadata;
pub mod model_key;
pub mod repository;
pub mod state_db;

const COMMAND_PREFIX: &str = "cmd";
const EVENT_PREFIX: &str = "evt";

use crate::metadata::MetadataError;
use crate::state_db::StateDbError;
use eventstore::Error as EventStoreError;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Error as SerdeError;
use std::error::Error;
use std::fmt::Debug;
use std::str::Utf8Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EventSourceError<S> {
    #[error("Cache error")]
    StateDbError(StateDbError),

    #[error("Event store error")]
    EventStore(EventStoreError),

    #[error("Event store postion error")]
    Position(String),

    #[error("Utf8 error")]
    Utf8(Utf8Error),

    #[error("Metadata error")]
    Metadata(MetadataError),

    #[error("Serde error")]
    Serde(SerdeError),

    #[error("State error")]
    State(S),

    #[error("unknown cache db error")]
    Unknown,
}

pub type CommandName = &'static str;
pub type EventName = &'static str;
pub type StateName = &'static str;

pub enum EventType {
    State,
    Event,
}

pub trait Command: Serialize + DeserializeOwned + Debug + Send + Clone {
    fn command_name(&self) -> CommandName;
}

pub trait Event: Serialize + DeserializeOwned + Debug + Send + Clone {
    fn event_name(&self) -> EventName;

    fn event_list() -> Vec<EventName>;

    fn get_type(&self) -> EventType {
        EventType::State
    }
}

pub trait State: Default + Serialize + DeserializeOwned + Debug + Send + Clone {
    type Event: Event;
    type Command: Command + Sync + Send;
    type Error: Error;

    fn name_prefix() -> StateName;

    fn play_event(&mut self, event: &Self::Event);

    fn try_command(&self, command: Self::Command) -> Result<Vec<Self::Event>, Self::Error>;
}
