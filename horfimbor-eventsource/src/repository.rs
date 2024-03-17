use std::cmp::Ordering;
use std::fmt::Debug;
use std::marker::PhantomData;

use async_trait::async_trait;
use eventstore::{
    AppendToStreamOptions, Client as EventDb, Error, EventData, ExpectedRevision,
    PersistentSubscriptionOptions, ReadStreamOptions, RetryOptions, StreamPosition,
    SubscribeToPersistentSubscriptionOptions, SubscribeToStreamOptions, Subscription,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::cache_db::CacheDb;
use crate::metadata::{CompleteEvent, Metadata};
use crate::model_key::ModelKey;
use crate::{Dto, EventSourceError};
use crate::{State, Stream};

#[derive(Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct DtoRepository<D, C>
where
    D: Dto,
    C: CacheDb<D>,
{
    event_db: EventDb,
    cache_db: C,
    dto: PhantomData<D>,
}

#[derive(Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct StateRepository<S, C>
where
    S: State,
    C: CacheDb<S>,
{
    event_db: EventDb,
    state_db: C,
    state: PhantomData<S>,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct ModelWithPosition<M> {
    position: Option<u64>,
    model: M,
}

impl<M> ModelWithPosition<M>
where
    M: Dto,
{
    pub const fn state(&self) -> &M {
        &self.model
    }

    pub fn play_event(&mut self, event: &M::Event, position: Option<u64>) {
        self.model.play_event(event);

        self.position = position;
    }

    pub const fn position(&self) -> Option<u64> {
        self.position
    }
}

#[async_trait]
pub trait Repository<D, C>: Clone + Send
where
    D: Dto,
    C: CacheDb<D>,
{
    fn new(event_db: EventDb, cache_db: C) -> Self;
    fn event_db(&self) -> &EventDb;
    fn cache_db(&self) -> &C;

    async fn get_model(
        &self,
        key: &ModelKey,
    ) -> Result<ModelWithPosition<D>, EventSourceError<D::Error>>
    where
        D: Dto + DeserializeOwned,
    {
        let value = self
            .cache_db()
            .get(key)
            .map_err(EventSourceError::CacheDbError)?;

        self.complete_from_es(key, &value).await
    }

    async fn complete_from_es(
        &self,
        key: &ModelKey,
        value: &ModelWithPosition<D>,
    ) -> Result<ModelWithPosition<D>, EventSourceError<<D as Dto>::Error>> {
        let mut dto: D = value.model.clone();
        let mut position = value.position;

        let options = ReadStreamOptions::default();
        let options = if let Some(position) = value.position {
            options.position(StreamPosition::Position(position + 1))
        } else {
            options.position(StreamPosition::Start)
        };

        let mut stream = self
            .event_db()
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
                    .as_json::<D::Event>()
                    .map_err(EventSourceError::Serde)?;

                dto.play_event(&event);
            }

            position = Some(original_event.revision);
        }

        let result = ModelWithPosition {
            position,
            model: dto,
        };

        Ok(result)
    }

    async fn create_subscription(
        &self,
        stream: &Stream,
        group_name: &str,
    ) -> Result<(), EventSourceError<D::Error>> {
        let opt = PersistentSubscriptionOptions::default().resolve_link_tos(true);

        let created = self
            .event_db()
            .create_persistent_subscription(stream.to_string(), group_name, &opt)
            .await;

        match created {
            Ok(()) => {}
            Err(e) => match e {
                Error::ResourceAlreadyExists => {}
                _ => return Err(EventSourceError::EventStore(e)),
            },
        }

        Ok(())
    }

    async fn get_subscription(&self, stream: Stream, position: Option<u64>) -> Subscription {
        let mut options =
            SubscribeToStreamOptions::default().retry_options(RetryOptions::default());

        options = match position {
            None => options.start_from(StreamPosition::Start),
            Some(n) => options.start_from(StreamPosition::Position(n)),
        };

        self.event_db()
            .subscribe_to_stream(stream.to_string(), &options)
            .await
    }

    async fn cache_dto(
        &self,
        stream: &Stream,
        group_name: &str,
    ) -> Result<(), EventSourceError<<D as Dto>::Error>> {
        self.create_subscription(stream, group_name).await?;

        let options = SubscribeToPersistentSubscriptionOptions::default().buffer_size(1);

        let mut sub = self
            .event_db()
            .subscribe_to_persistent_subscription(stream.to_string(), group_name, &options)
            .await
            .map_err(EventSourceError::EventStore)?;

        loop {
            let rcv_event = sub.next().await.map_err(EventSourceError::EventStore)?;

            let event = match rcv_event.event.as_ref() {
                None => {
                    continue;
                }
                Some(event) => event,
            };

            let metadata: Metadata = serde_json::from_slice(event.custom_metadata.as_ref())
                .map_err(EventSourceError::Serde)?;

            let model_key: ModelKey = event.stream_id.as_str().into();

            let mut model = self
                .cache_db()
                .get(&model_key)
                .map_err(EventSourceError::CacheDbError)?;

            let ordering = if event.revision == 0 {
                if model.position.is_some() {
                    Ordering::Greater
                } else {
                    Ordering::Equal
                }
            } else {
                model
                    .position
                    .map_or(Ordering::Less, |pos| pos.cmp(&(event.revision - 1)))
            };

            match ordering {
                Ordering::Less => {
                    model = self.complete_from_es(&model_key, &model).await?;
                    dbg!(format!(
                        "cache have been completed from {:?} to {:?}",
                        model.position, event.revision,
                    ));

                    self.cache_db()
                        .set(&model_key, model)
                        .map_err(EventSourceError::CacheDbError)?;
                }
                Ordering::Equal => {
                    if metadata.is_event() {
                        let dto_event = event
                            .as_json::<D::Event>()
                            .map_err(EventSourceError::Serde)?;

                        model.play_event(&dto_event, Some(event.revision));
                    } else {
                        model.position = Some(event.revision);
                    }
                    self.cache_db()
                        .set(&model_key, model)
                        .map_err(EventSourceError::CacheDbError)?;
                }
                Ordering::Greater => {
                    dbg!(format!(
                        "cache should be lower than {} but is : {:?}",
                        event.revision, model.position
                    ));
                }
            }

            sub.ack(rcv_event)
                .await
                .map_err(EventSourceError::EventStore)?;
        }
    }

    /// # Errors
    ///
    /// Will return `Err` if input is not in the format `index@stream_id`
    fn split_event_id(str: &str) -> Result<(&str, &str), EventSourceError<D::Error>> {
        let mut iter = str.split(|c| c == '@');

        if let (Some(index), Some(stream_id)) = (iter.next(), iter.next()) {
            return Ok((index, stream_id));
        }

        Err(EventSourceError::Position(format!(
            "{str} isnt in the format index@stream_id"
        )))
    }
}

impl<D, C> Repository<D, C> for DtoRepository<D, C>
where
    D: Dto,
    C: CacheDb<D>,
{
    fn new(event_db: EventDb, cache_db: C) -> Self {
        Self {
            event_db,
            cache_db,
            dto: PhantomData,
        }
    }
    fn event_db(&self) -> &EventDb {
        &self.event_db
    }
    fn cache_db(&self) -> &C {
        &self.cache_db
    }
}

impl<S, C> Repository<S, C> for StateRepository<S, C>
where
    S: State,
    C: CacheDb<S>,
{
    fn new(event_db: EventDb, state_db: C) -> Self {
        Self {
            event_db,
            state_db,
            state: PhantomData,
        }
    }

    fn event_db(&self) -> &EventDb {
        &self.event_db
    }
    fn cache_db(&self) -> &C {
        &self.state_db
    }
}

impl<S, C> StateRepository<S, C>
where
    S: State,
    C: CacheDb<S>,
{
    /// # Errors
    ///
    /// Will return `Err` if events cannot be added to the evenstore
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
        S: State + Sync,
    {
        let model: ModelWithPosition<S> = self.get_model(key).await?;

        let state = model.model;

        let events = state
            .try_command(command.clone())
            .map_err(EventSourceError::State)?;

        let options = model.position.map_or_else(
            || AppendToStreamOptions::default().expected_revision(ExpectedRevision::NoStream),
            |position| {
                AppendToStreamOptions::default()
                    .expected_revision(ExpectedRevision::Exact(position))
            },
        );

        let command_metadata = CompleteEvent::from_command(command, previous_metadata)
            .map_err(EventSourceError::Metadata)?;

        let mut events_data = vec![command_metadata.clone()];

        let mut previous_metadata = command_metadata.metadata().to_owned();

        let res_events = events.clone();

        for event in events {
            let event_metadata = CompleteEvent::from_event(event, &previous_metadata)
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
        events_with_data: Vec<CompleteEvent>,
    ) -> Result<bool, EventSourceError<S::Error>>
    where
        S: State,
    {
        let events: Vec<EventData> = events_with_data
            .into_iter()
            .map(|e| e.event_data().to_owned())
            .collect();

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
