use std::fmt::Debug;

use anyhow::{anyhow, Context, Result};
use eventstore::{
    AppendToStreamOptions, Client as EventDb, Error, EventData, ExpectedRevision,
    ReadStreamOptions, StreamPosition,
};
use redis::Client as CacheDb;
use redis::Commands;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::metadata::{EventWithMetadata, Metadata};
use crate::model_key::ModelKey;
use crate::state::State;

#[derive(Clone)]
pub struct StateRepository {
    event_db: EventDb,
    cache_db: CacheDb,
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

impl StateRepository {
    pub fn new(event_db: EventDb, cache_db: CacheDb) -> Self {
        Self { event_db, cache_db }
    }

    pub async fn get_model<S>(&self, key: &ModelKey) -> Result<StateWithInfo<S>>
    where
        S: State + DeserializeOwned,
    {
        let value = if S::state_cache_interval().is_some() {
            let mut cache_connection = self
                .cache_db
                .get_connection()
                .context("connect to cache db")?;
            let data: String = cache_connection
                .get(key.format())
                .context("get from cache")
                .unwrap_or_default();
            serde_json::from_str(data.as_str()).unwrap_or_default()
        } else {
            StateWithInfo::default()
        };

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
            .context("connect to event db")?;

        let mut nb_change = 0;

        while let Ok(Some(json_event)) = stream.next().await {
            let original_event = json_event.get_original_event();

            let metadata: Metadata =
                serde_json::from_slice(original_event.custom_metadata.as_ref()).unwrap();

            if metadata.is_event() {
                let event = original_event
                    .as_json::<S::Event>()
                    .context(format!("decode event : {:?}", json_event))?;

                state.play_event(&event);
                nb_change += 1;
            }

            info.position = Some(original_event.revision)
        }

        let result = StateWithInfo { info, state };

        if S::state_cache_interval().is_some() && nb_change > S::state_cache_interval().unwrap() {
            let mut cache_connection = self
                .cache_db
                .get_connection()
                .context("connect to cache db")?;

            cache_connection
                .set(key.format(), serde_json::to_string(&result)?)
                .context("set cache value")?;
        }

        Ok(result)
    }

    pub async fn add_command<T>(
        &self,
        key: &ModelKey,
        command: T::Command,
        previous_metadata: Option<&Metadata>,
    ) -> Result<T>
    where
        T: State,
    {
        let mut model: T;
        let events: Vec<T::Event>;

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
    ) -> Result<(S, Vec<S::Event>, bool)>
    where
        S: State,
    {
        let model: StateWithInfo<S> = self.get_model(key).await.context("adding command")?;

        let state = model.state;
        let info = model.info;

        let events = state.try_command(command.clone()).context("try command")?;

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

    pub async fn try_append_event_data(
        &self,
        key: &ModelKey,
        options: &AppendToStreamOptions,
        events_with_data: Vec<EventWithMetadata>,
    ) -> Result<bool> {
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
                    return Err(anyhow!("error while appending : {:?}", err));
                }
            }
        }
        Ok(retry)
    }

    pub fn event_db(&self) -> &EventDb {
        &self.event_db
    }
    pub fn cache_db(&self) -> &CacheDb {
        &self.cache_db
    }
}
