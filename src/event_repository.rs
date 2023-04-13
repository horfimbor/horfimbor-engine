use std::fmt::Debug;
use std::marker::PhantomData;

use eventstore::{
    AppendToStreamOptions, Client as EventDb, Error, EventData, ExpectedRevision,
    ReadStreamOptions, StreamPosition,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::metadata::{EventWithMetadata, Metadata};
use crate::model_key::ModelKey;
use crate::state::State;
use crate::state_db::StateDb;
use crate::EventSourceError;

#[derive(Clone)]
pub struct EventRepository<C, S>
where
    S: State,
    C: StateDb<S>,
{
    event_db: EventDb,
    state_db: C,
    state: PhantomData<S>,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
struct StateInformation {
    position: Option<u64>,
}

impl StateInformation {
    fn next(&mut self) {
        self.position = match self.position {
            None => Some(0),
            Some(i) => Some(i + 1),
        }
    }
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct StateWithInfo<S> {
    info: StateInformation,
    state: S,
}

impl<S> StateWithInfo<S>
where
    S: State,
{
    pub fn state(&self) -> &S {
        &self.state
    }

    pub fn play_event(&mut self, event: &S::Event) {
        self.state.play_event(event);
        self.info.next();
    }
}

impl<C, S> EventRepository<C, S>
where
    S: State,
    C: StateDb<S>,
{
    pub fn new(event_db: EventDb, state_db: C) -> Self {
        Self {
            event_db,
            state_db,
            state: Default::default(),
        }
    }

    pub async fn get_model(
        &self,
        key: &ModelKey,
    ) -> Result<StateWithInfo<S>, EventSourceError<S::Error>>
    where
        S: State + DeserializeOwned,
    {
        let value = self
            .state_db
            .get(key)
            .map_err(EventSourceError::StateDbError)?;

        let mut state: S = value.state;
        let mut info = value.info;

        let options = ReadStreamOptions::default();
        let options = if let Some(position) = info.position {
            options.position(StreamPosition::Position(position + 1))
        } else {
            options.position(StreamPosition::Start)
        };

        let mut stream = self
            .event_db
            .read_stream(key.format(), &options)
            .await
            .map_err(EventSourceError::EventStore)?;

        while let Ok(Some(json_event)) = stream.next().await {
            let original_event = json_event.get_original_event();

            let metadata: Metadata =
                serde_json::from_slice(original_event.custom_metadata.as_ref()).unwrap();

            if metadata.is_event() {
                let event = original_event
                    .as_json::<S::Event>()
                    .map_err(EventSourceError::Serde)?;

                state.play_event(&event);
            }

            info.position = Some(original_event.revision)
        }

        let result = StateWithInfo { info, state };

        Ok(result)
    }

    pub async fn add_command(
        &self,
        key: &ModelKey,
        command: S::Command,
        previous_metadata: Option<&Metadata>,
    ) -> Result<S, EventSourceError<S::Error>>
    where
        S: State,
    {
        let mut model: S;
        let events: Vec<S::Event>;

        loop {
            let (l_model, l_events, retry) = self
                .try_append(key, command.clone(), previous_metadata)
                .await?;
            if retry {
                continue;
            }

            model = l_model;
            events = l_events;

            break;
        }

        for event in &events {
            model.play_event(event);
        }

        Ok(model)
    }

    async fn try_append(
        &self,
        key: &ModelKey,
        command: S::Command,
        previous_metadata: Option<&Metadata>,
    ) -> Result<(S, Vec<S::Event>, bool), EventSourceError<S::Error>>
    where
        S: State,
    {
        let model: StateWithInfo<S> = self.get_model(key).await?;

        let state = model.state;
        let info = model.info;

        let events = state
            .try_command(command.clone())
            .map_err(EventSourceError::State)?;

        let options = if let Some(position) = info.position {
            AppendToStreamOptions::default().expected_revision(ExpectedRevision::Exact(position))
        } else {
            AppendToStreamOptions::default().expected_revision(ExpectedRevision::NoStream)
        };

        let command_metadata =
            EventWithMetadata::from_command(command, previous_metadata, S::name_prefix());

        let mut events_data = vec![command_metadata.clone()];

        let mut previous_metadata = command_metadata.metadata().to_owned();

        let res_events = events.clone();

        for event in events {
            let event_metadata =
                EventWithMetadata::from_event(event, &previous_metadata, S::name_prefix());

            events_data.push(event_metadata.clone());
            previous_metadata = event_metadata.metadata().to_owned();
        }

        let retry = self
            .try_append_event_data(key, &options, events_data)
            .await?;

        Ok((state, res_events, retry))
    }

    async fn try_append_event_data(
        &self,
        key: &ModelKey,
        options: &AppendToStreamOptions,
        events_with_data: Vec<EventWithMetadata>,
    ) -> Result<bool, EventSourceError<S::Error>>
    where
        S: State,
    {
        let events: Vec<EventData> = events_with_data
            .into_iter()
            .map(|e| e.full_event_data())
            .collect();

        let appended = self
            .event_db
            .append_to_stream(key.format(), options, events)
            .await;

        let mut retry = false;

        if appended.is_err() {
            let err = appended.unwrap_err();
            match err {
                Error::WrongExpectedVersion { expected, current } => {
                    println!("{current} instead of {expected}");

                    retry = true;
                }
                _ => {
                    return Err(EventSourceError::EventStore(err));
                }
            }
        }
        Ok(retry)
    }
}
