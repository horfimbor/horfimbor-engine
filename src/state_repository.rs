use std::fmt::Debug;

use eventstore::{
    AppendToStreamOptions, Client as EventDb, Error, EventData, ExpectedRevision,
    ReadStreamOptions, StreamPosition,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::cache::{CacheDb, CacheError};
use crate::metadata::{EventWithMetadata, Metadata};
use crate::model_key::ModelKey;
use crate::state::State;
use crate::EventSourceError;

#[derive(Clone)]
pub struct StateRepository<C> {
    event_db: EventDb,
    cache_db: C,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
struct StateInformation {
    position: Option<u64>,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct StateWithInfo<S> {
    info: StateInformation,
    state: S,
}

impl<S> StateWithInfo<S> {
    pub fn state(&self) -> &S {
        &self.state
    }
}

impl<C> StateRepository<C>
where
    C: CacheDb,
{
    pub fn new(event_db: EventDb, cache_db: C) -> Self {
        Self { event_db, cache_db }
    }

    fn get_from_cache<S>(
        &self,
        key: &ModelKey,
    ) -> Result<StateWithInfo<S>, EventSourceError<S::Error>>
    where
        S: State + DeserializeOwned,
    {
        let data: Result<String, CacheError> = self.cache_db.get(key);

        match data {
            Ok(value) => Ok(serde_json::from_str(value.as_str()).unwrap_or_default()),
            Err(err) => {
                if err == CacheError::NotFound {
                    return Ok(StateWithInfo::default());
                }

                Err(EventSourceError::CacheError(err))
            }
        }
    }

    pub async fn get_model<S>(
        &self,
        key: &ModelKey,
    ) -> Result<StateWithInfo<S>, EventSourceError<S::Error>>
    where
        S: State + DeserializeOwned,
    {
        let value = self.get_from_cache(key)?;

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
            .map_err(|err| EventSourceError::EventStore(err))?;

        while let Ok(Some(json_event)) = stream.next().await {
            let original_event = json_event.get_original_event();

            let metadata: Metadata =
                serde_json::from_slice(original_event.custom_metadata.as_ref()).unwrap();

            if metadata.is_event() {
                let event = original_event
                    .as_json::<S::Event>()
                    .map_err(|err| EventSourceError::Serde(err))?;

                state.play_event(&event);
            }

            info.position = Some(original_event.revision)
        }

        let result = StateWithInfo { info, state };

        Ok(result)
    }

    pub async fn add_command<S>(
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

    async fn try_append<S>(
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
            .map_err(|err| EventSourceError::State(err))?;

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
            .try_append_event_data::<S>(key, &options, events_data)
            .await?;

        Ok((state, res_events, retry))
    }

    pub async fn try_append_event_data<S>(
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
