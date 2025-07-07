#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

/// re-export import :
pub use horfimbor_eventsource_derive;
use kurrentdb::Error as EventStoreError;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Error as SerdeError;
use thiserror::Error;
use uuid::{Error as UuidError, Uuid};

use crate::cache_db::DbError;
use crate::model_key::ModelKey;

pub mod cache_db;
pub mod helper;
pub mod metadata;
pub mod model_key;
pub mod repository;

/// str wrapper
pub type StreamName = &'static str;

/// `Stream` allow to create a subscription for different kind.
pub enum Stream {
    /// subscribe to an entity
    Model(ModelKey),
    /// subscribe to all the event of a stream
    Stream(StreamName),
    /// subscribe to an event across entities
    Event(EventName),
    /// subscribe to all the descendant of a specific event
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

/// error for the repository
#[derive(Error, Debug)]
pub enum EventSourceError {
    /// the cache is gone ?
    #[error("Cache error")]
    CacheDbError(#[from] DbError),

    /// the db is gone ?
    #[error("Event store error")]
    EventStore(#[from] EventStoreError),

    /// Bad position can append when race condition occur
    /// this error is retryable
    #[error("Event store position error : {0}")]
    Position(String),

    /// Error for serialization
    #[error("Serde error")]
    Serde(#[from] SerdeError),

    /// Error when converting uuid
    #[error("Uuid error")]
    Uuid(#[from] UuidError),
}

/// error coming from the `StateRepository`
#[derive(Error, Debug)]
pub enum EventSourceStateError {
    /// error from `EventSourceError`
    #[error("Event source error")]
    EventSourceError(#[from] EventSourceError),

    /// error depending on the `State`
    #[error("State error : {0}")]
    State(String),
}

/// str wrapper
pub type CommandName = &'static str;
/// str wrapper
pub type EventName = &'static str;
/// str wrapper
pub type StateName = &'static str;

/// `Command` are an enum for all the action possible
pub trait Command: Serialize + DeserializeOwned + Debug + Send + Clone {
    /// the `CommandName` must be unique for each variant of the enum
    fn command_name(&self) -> CommandName;
}

/// `Event` are an enum for all the change possible
pub trait Event: Serialize + DeserializeOwned + Debug + Send + Clone {
    /// the `EventName` must be unique for each variant of the enum
    fn event_name(&self) -> EventName;
}

/// the `Dto` trait provide a reader on the database
pub trait Dto: Default + Serialize + DeserializeOwned + Debug + Send + Clone + Sync {
    /// Event is the enum for all possible change on the `Dto`
    type Event: Event + Sync + Send;

    /// events are played one by one
    fn play_event(&mut self, event: &Self::Event);
}

/// `StateNamed` just provide a getter for the `StateName`
pub trait StateNamed {
    /// that's all
    fn state_name() -> StateName;
}

/// the `State` trait add update to the state.
pub trait State: Dto + StateNamed {
    /// the command are all the possible operation on the State
    type Command: Command + Sync + Send;

    /// the error that can occurs from the command
    type Error: Error + Sync + Send;

    /// command can produce multiple events or an error if condition are not met
    ///
    /// # Errors
    ///
    /// Will return `Err` if Command cannot currently occur OR something is wrong with DB
    fn try_command(&self, command: Self::Command) -> Result<Vec<Self::Event>, Self::Error>;
}
