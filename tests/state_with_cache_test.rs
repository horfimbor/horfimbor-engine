use eventstore::{Client as EventClient, Client};
use gyg_eventsource::cache_db::redis::RedisStateDb;
use rand::distributions::Alphanumeric;
use rand::Rng;
use redis::Commands;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

use gyg_eventsource::model_key::ModelKey;

use gyg_eventsource::repository::Repository;
use gyg_eventsource::repository::StateRepository;

use crate::state_db::{PokeCommand, PokeState};

mod concurrent;
mod simple;
mod state_db;

type EasyRedisCache = RedisStateDb<PokeState>;

#[tokio::test]
async fn with_cache() {
    let name: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();

    let name2 = name.clone();
    tokio::spawn(async move {
        let event_store = get_event_db();
        let redis_client = redis::Client::open("redis://localhost:6379/").unwrap();
        let state_repo =
            StateRepository::new(event_store, EasyRedisCache::new(redis_client.clone()));

        state_repo
            .create_subscription(name2.as_str(), name2.as_str())
            .await
            .unwrap();

        state_repo
            .listen(name2.as_str(), name2.as_str())
            .await
            .unwrap();
    });

    let redis_client = redis::Client::open("redis://localhost:6379/").unwrap();

    let event_store = get_event_db();

    let repo = StateRepository::new(
        event_store.clone(),
        EasyRedisCache::new(redis_client.clone()),
    );

    let key = ModelKey::new(name, Uuid::new_v4().to_string());

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

    sleep(Duration::from_millis(2000)).await;

    let data_es = repo.get_model(&key).await.unwrap();
    dbg!(data_es);

    let data_redis: Option<String> = connection.get(key.format()).unwrap();
    assert_eq!(
        data_redis,
        Some(r#"{"position":3,"model":{"nb":182}}"#.to_string())
    );
}

fn get_event_db() -> Client {
    let settings = "esdb://admin:changeit@localhost:2113?tls=false&tlsVerifyCert=false"
        .to_string()
        .parse()
        .unwrap();
    EventClient::new(settings).unwrap()
}
