use eventstore::EventData;
use serde::{Deserialize, Serialize};
use serde_json::Error as SerdeError;
use thiserror::Error;
use uuid::Uuid;

use crate::{Command, Event, COMMAND_PREFIX, EVENT_PREFIX};

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

impl Metadata {
    pub fn correlation_id(&self) -> Uuid {
        self.correlation_id
    }
    pub fn causation_id(&self) -> Uuid {
        self.causation_id
    }
    pub fn set_id(&mut self, id: Option<Uuid>) {
        self.id = id;
    }
    pub fn id(&self) -> Option<Uuid> {
        self.id
    }

    pub fn new(id: Option<Uuid>, correlation_id: Uuid, causation_id: Uuid, is_event: bool) -> Self {
        Self {
            id,
            correlation_id,
            causation_id,
            is_event,
        }
    }
    pub fn is_event(&self) -> bool {
        self.is_event
    }
}

#[derive(Clone)]
pub struct EventWithMetadata {
    event_data: EventData,
    metadata: Metadata,
}

impl EventWithMetadata {
    pub fn event_data(&self) -> &EventData {
        &self.event_data
    }
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn full_event_data(&self) -> Result<EventData, MetadataError> {
        self.event_data
            .clone()
            .metadata_as_json(self.metadata())
            .map_err(MetadataError::SerdeError)
    }

    pub fn from_command<C>(
        command: C,
        previous_metadata: Option<&Metadata>,
    ) -> Result<Self, MetadataError>
    where
        C: Command,
    {
        let event_data = EventData::json(
            format!("{}.{}", COMMAND_PREFIX, command.command_name()),
            command,
        )
        .map_err(MetadataError::SerdeError)?;

        Ok(Self::from_event_data(event_data, previous_metadata, false))
    }

    pub fn from_event<E>(event: E, previous_metadata: &Metadata) -> Result<Self, MetadataError>
    where
        E: Event,
    {
        let key = format!("{}.{}", EVENT_PREFIX, event.event_name());

        let event_data = EventData::json(key, event).map_err(MetadataError::SerdeError)?;

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

        let metadata = match previous_metadata {
            None => Metadata {
                id: Some(id),
                correlation_id: id,
                causation_id: id,
                is_event,
            },
            Some(previous) => Metadata {
                id: Some(id),
                correlation_id: previous.correlation_id,
                causation_id: match previous.id {
                    None => id,
                    Some(p) => p,
                },
                is_event,
            },
        };

        Self {
            event_data,
            metadata,
        }
    }
}

#[derive(Error, Debug)]
pub enum MetadataError {
    #[error("Not found")]
    NotFound,

    #[error("internal `{0}`")]
    SerdeError(SerdeError),
}
