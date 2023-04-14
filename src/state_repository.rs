use std::marker::PhantomData;

use crate::model_key::ModelKey;
use eventstore::{Client as EventDb, ReadStreamOptions, StreamPosition};

use crate::state::State;
use crate::state_db::StateDb;

#[derive(Clone)]
pub struct StateRepository<C, S>
where
    S: State,
    C: StateDb<S>,
{
    event_db: EventDb,
    state_db: C,
    state: PhantomData<S>,
}

impl<C, S> StateRepository<C, S>
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

    pub async fn create_subscription(&self, group_name: &str) {
        dbg!(format!("$et-evt.{}", S::name_prefix()));

        self.event_db
            .create_persistent_subscription(
                format!("$et-evt.{}", S::name_prefix()),
                group_name,
                &Default::default(),
            )
            .await
            .unwrap();
    }

    pub async fn listen(&self, group_name: &str) {
        let mut sub = self
            .event_db
            .subscribe_to_persistent_subscription(
                format!("$et-evt.{}", S::name_prefix()),
                group_name,
                &Default::default(),
            )
            .await
            .unwrap();

        loop {
            let event = sub.next().await.unwrap();
            dbg!(&event);

            let or = event.get_original_event().data.clone();

            let str = std::str::from_utf8(or.as_ref()).unwrap();

            let mut iter = str.split(|c| c == '@');

            let index = iter.next().unwrap();
            let stream_id = iter.next().unwrap();
            dbg!(&stream_id);

            let pos: u64 = index.parse().unwrap();

            let options = ReadStreamOptions::default().position(StreamPosition::Position(pos));

            let mut stream = self
                .event_db
                .read_stream(stream_id, &options)
                .await
                .unwrap();

            let json_event = stream.next().await.unwrap().unwrap();

            let original_event = json_event.get_original_event();
            dbg!(&original_event);

            let model_key: ModelKey = stream_id.into();

            let event = original_event.as_json::<S::Event>().unwrap();

            let mut state = self.state_db.get(&model_key).unwrap();
            dbg!(&event);

            state.play_event(&event, Some(original_event.revision));

            dbg!(&state);

            self.state_db.set(&model_key, state).unwrap();
        }
    }
}
