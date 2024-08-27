//! common metadata for all the events and command

use eventstore::EventData;
use serde::{Deserialize, Serialize};
use serde_json::Error as SerdeError;
use uuid::Uuid;

use crate::{Command, Event};

/// `Metadata` must be serialized a certain way to allow build-in projections
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Metadata {
    #[serde(skip_serializing)]
    id: Option<Uuid>,
    #[serde(rename = "$correlationId")]
    correlation_id: Uuid,
    #[serde(rename = "$causationId")]
    causation_id: Uuid,
    #[serde(rename = "is_event")]
    is_event: bool,
}

/// `Metadata` provide genealogy of the events
impl Metadata {
    /// the `correlation_id` is the oldest event in the genealogy
    #[must_use]
    pub const fn correlation_id(&self) -> Uuid {
        self.correlation_id
    }

    /// the `causation_id` is the youngest event in the genealogy
    #[must_use]
    pub const fn causation_id(&self) -> Uuid {
        self.causation_id
    }

    /// id can be set afterward
    pub fn set_id(&mut self, id: Option<Uuid>) {
        self.id = id;
    }

    /// simple getter
    #[must_use]
    pub const fn id(&self) -> Option<Uuid> {
        self.id
    }

    /// straight forward constructor
    #[must_use]
    pub const fn new(
        id: Option<Uuid>,
        correlation_id: Uuid,
        causation_id: Uuid,
        is_event: bool,
    ) -> Self {
        Self {
            id,
            correlation_id,
            causation_id,
            is_event,
        }
    }

    /// the event in the database can be an event or a command
    #[must_use]
    pub const fn is_event(&self) -> bool {
        self.is_event
    }
}

/// event in the db are composed of the `EventData` and the `Metadata`
#[derive(Clone)]
pub struct CompleteEvent {
    event_data: EventData,
    metadata: Metadata,
}

impl CompleteEvent {
    /// public readonly getter for the `EventData`
    pub const fn event_data(&self) -> &EventData {
        &self.event_data
    }

    /// public readonly getter for the `Metadata`
    pub const fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    /// # Errors
    ///
    /// Will return `Err` if `Metadata` cannot be de into json
    pub fn full_event_data(&self) -> Result<EventData, SerdeError> {
        self.event_data.clone().metadata_as_json(self.metadata())
    }

    /// # Errors
    ///
    /// Will return `Err` if `Metadata` cannot be de into json
    pub fn from_command<C>(
        command: C,
        previous_metadata: Option<&Metadata>,
    ) -> Result<Self, SerdeError>
    where
        C: Command,
    {
        let event_data = EventData::json(command.command_name(), command)?;

        Ok(Self::from_event_data(event_data, previous_metadata, false))
    }

    /// # Errors
    ///
    /// Will return `Err` if `Metadata` cannot be serialized into json
    pub fn from_event<E>(event: E, previous_metadata: &Metadata) -> Result<Self, SerdeError>
    where
        E: Event,
    {
        let event_data = EventData::json(event.event_name(), event)?;

        Ok(Self::from_event_data(
            event_data,
            Some(previous_metadata),
            true,
        ))
    }

    fn from_event_data(
        mut event_data: EventData,
        previous_metadata: Option<&Metadata>,
        is_event: bool,
    ) -> Self {
        let id = Uuid::new_v4();

        event_data = event_data.id(id);

        let metadata = previous_metadata.map_or(
            Metadata {
                id: Some(id),
                correlation_id: id,
                causation_id: id,
                is_event,
            },
            |previous| Metadata {
                id: Some(id),
                correlation_id: previous.correlation_id,
                causation_id: previous.id.unwrap_or(id),
                is_event,
            },
        );

        Self {
            event_data,
            metadata,
        }
    }
}
