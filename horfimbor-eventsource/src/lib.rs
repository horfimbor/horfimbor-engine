use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::str::Utf8Error;

use eventstore::Error as EventStoreError;
/// re-export import :
pub use horfimbor_eventsource_derive;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Error as SerdeError;
use thiserror::Error;
use uuid::Uuid;

use crate::cache_db::DbError;
use crate::metadata::Error as MetadataError;
use crate::model_key::ModelKey;

pub mod cache_db;
pub mod helper;
pub mod metadata;
pub mod model_key;
pub mod repository;

pub type StreamName = &'static str;

pub enum Stream {
    Model(ModelKey),
    Stream(StreamName),
    Event(EventName),
    Correlation(Uuid),
}

impl Display for Stream {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Model(m) => {
                write!(f, "{}", m.format())
            }
            Self::Stream(stream_name) => {
                let n = stream_name.replace('-', "_");
                write!(f, "$ce-{n}")
            }
            Self::Event(e) => {
                write!(f, "$et-{e}")
            }
            Self::Correlation(u) => {
                write!(f, "bc-{u}")
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum EventSourceError<S> {
    #[error("Cache error")]
    CacheDbError(DbError),

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

pub trait Command: Serialize + DeserializeOwned + Debug + Send + Clone {
    fn command_name(&self) -> CommandName;
}

pub trait Event: Serialize + DeserializeOwned + Debug + Send + Clone {
    fn event_name(&self) -> EventName;
}

pub trait Dto: Default + Serialize + DeserializeOwned + Debug + Send + Clone + Sync {
    type Event: Event + Sync + Send;
    type Error: Error + Sync + Send; // FIXME : move ERROR to State

    fn play_event(&mut self, event: &Self::Event);
}

pub trait StateNamed {
    fn state_name() -> StateName;
}

pub trait State: Dto + StateNamed {
    type Command: Command + Sync + Send;

    /// # Errors
    ///
    /// Will return `Err` if Command cannot currently occur OR something is wrong with DB
    fn try_command(&self, command: Self::Command) -> Result<Vec<Self::Event>, Self::Error>;
}
