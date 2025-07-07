#[macro_use]
extern crate lazy_static;

use std::time::Duration;

use kurrentdb::Client as EventClient;
use rand::Rng;
use rand::distr::Alphanumeric;
use redis::Commands;
use tokio::time::sleep;
use uuid::Uuid;

use horfimbor_eventsource::Stream;
use horfimbor_eventsource::cache_db::redis::StateDb;
use horfimbor_eventsource::model_key::ModelKey;
use horfimbor_eventsource::repository::Repository;
use horfimbor_eventsource::repository::StateRepository;

use crate::state_db::{PokeCommand, PokeState};

mod concurrent;
mod simple;
mod state_db;

type EasyRedisCache = StateDb<PokeState>;

lazy_static! {
    static ref NAME: &'static str = {
        let name: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();
        Box::leak(name.into_boxed_str())
    };
}

#[tokio::test]
async fn with_cache() {
    let redis_client = redis::Client::open("redis://localhost:6379/").unwrap();
    let event_store = get_event_db();
    let state_repo = StateRepository::new(event_store, EasyRedisCache::new(redis_client.clone()));

    let stream = Stream::Stream(&NAME);

    tokio::spawn(async move {
        let state_repo = state_repo.clone();

        state_repo.cache_dto(&stream, &NAME).await.unwrap();
    });

    let event_store = get_event_db();

    let repo = StateRepository::new(
        event_store.clone(),
        EasyRedisCache::new(redis_client.clone()),
    );

    let key = ModelKey::new(&NAME, Uuid::new_v4());

    let model = repo.get_model(&key).await.unwrap();

    assert_eq!(model.state(), &PokeState { nb: 0 });

    let mut connection = redis_client.get_connection().unwrap();
    let data: Option<String> = connection.get(key.format()).unwrap();

    assert_eq!(data, None);

    repo.add_command(&key, PokeCommand::Poke(80), None)
        .await
        .unwrap();
    let added = repo
        .add_command(&key, PokeCommand::Poke(102), None)
        .await
        .unwrap();

    assert_eq!(added, (PokeState { nb: 182 }));

    sleep(Duration::from_millis(1000)).await;

    let data_redis: Option<String> = connection.get(key.format()).unwrap();
    assert_eq!(
        data_redis,
        Some(r#"{"position":3,"model":{"nb":182}}"#.to_string())
    );

    let data_es = repo.get_model(&key).await.unwrap();
    dbg!(data_es.clone());

    assert_eq!(
        serde_json::to_string(&data_es).unwrap(),
        r#"{"position":3,"model":{"nb":182}}"#.to_string()
    );
}

fn get_event_db() -> EventClient {
    let settings = "esdb://admin:changeit@localhost:2113?tls=false&tlsVerifyCert=false"
        .to_string()
        .parse()
        .unwrap();
    EventClient::new(settings).unwrap()
}
