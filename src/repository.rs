use std::fmt::Debug;
use std::marker::PhantomData;

use eventstore::{
    AppendToStreamOptions, Client as EventDb, Error, EventData, ExpectedRevision,
    ReadStreamOptions, ResolvedEvent, StreamPosition,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::metadata::{EventWithMetadata, Metadata};
use crate::model_key::ModelKey;
use crate::state_db::StateDb;
use crate::EventSourceError;
use crate::State;

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

    pub fn play_event(&mut self, event: &S::Event, position: Option<u64>) {
        self.state.play_event(event);

        self.info.position = position
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
                serde_json::from_slice(original_event.custom_metadata.as_ref())
                    .map_err(EventSourceError::Serde)?;

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

    pub async fn create_subscription(&self, group_name: &str) -> Result<(), EventSourceError<S>> {
        dbg!(format!("$et-evt.{}", S::name_prefix()));

        self.event_db
            .create_persistent_subscription(
                format!("$et-evt.{}", S::name_prefix()),
                group_name,
                &Default::default(),
            )
            .await
            .map_err(EventSourceError::EventStore)?;

        Ok(())
    }

    pub async fn listen(&self, group_name: &str) -> Result<(), EventSourceError<S>> {
        let mut sub = self
            .event_db
            .subscribe_to_persistent_subscription(
                format!("$et-evt.{}", S::name_prefix()),
                group_name,
                &Default::default(),
            )
            .await
            .map_err(EventSourceError::EventStore)?;

        loop {
            let event = sub.next().await.map_err(EventSourceError::EventStore)?;
            dbg!(&event);

            let original_event = event.get_original_event().data.clone();

            let event_id =
                std::str::from_utf8(original_event.as_ref()).map_err(EventSourceError::Utf8)?;

            let (index, stream_id) = Self::split_event_id(event_id)?;
            dbg!(&stream_id);

            let pos: u64 = index.parse().map_err(|_e| EventSourceError::Unknown)?;

            let options = ReadStreamOptions::default().position(StreamPosition::Position(pos));

            let mut stream = self
                .event_db
                .read_stream(stream_id, &options)
                .await
                .map_err(EventSourceError::EventStore)?;

            let json_event: ResolvedEvent = stream
                .next()
                .await
                .map_err(EventSourceError::EventStore)?
                .ok_or(EventSourceError::Unknown)?;

            let original_event = json_event.get_original_event();
            dbg!(&original_event);

            let model_key: ModelKey = stream_id.into();

            let event = original_event
                .as_json::<S::Event>()
                .map_err(EventSourceError::Serde)?;

            let mut state = self
                .state_db
                .get(&model_key)
                .map_err(EventSourceError::StateDbError)?;
            dbg!(&event);

            state.play_event(&event, Some(original_event.revision));

            dbg!(&state);

            self.state_db
                .set(&model_key, state)
                .map_err(EventSourceError::StateDbError)?;
        }
    }

    fn split_event_id(str: &str) -> Result<(&str, &str), EventSourceError<S>> {
        let mut iter = str.split(|c| c == '@');

        if let (Some(index), Some(stream_id)) = (iter.next(), iter.next()) {
            return Ok((index, stream_id));
        }

        Err(EventSourceError::Position(format!(
            "{} isnt in the format index@stream_id",
            str
        )))
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
            EventWithMetadata::from_command(command, previous_metadata, S::name_prefix())
                .map_err(EventSourceError::Metadata)?;

        let mut events_data = vec![command_metadata.clone()];

        let mut previous_metadata = command_metadata.metadata().to_owned();

        let res_events = events.clone();

        for event in events {
            let event_metadata =
                EventWithMetadata::from_event(event, &previous_metadata, S::name_prefix())
                    .map_err(EventSourceError::Metadata)?;

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
        let mut err = Ok(());
        let events: Vec<EventData> = events_with_data
            .into_iter()
            .filter_map(|e| match e.full_event_data() {
                Ok(event) => Some(event),
                Err(e) => {
                    err = Err(EventSourceError::Metadata(e));
                    None
                }
            })
            .collect();
        err?;

        let appended = self
            .event_db
            .append_to_stream(key.format(), options, events)
            .await;

        match appended {
            Ok(_) => Ok(false),
            Err(Error::WrongExpectedVersion { expected, current }) => {
                println!("{current} instead of {expected}");
                Ok(true)
            }
            Err(e) => Err(EventSourceError::EventStore(e)),
        }
    }
}
