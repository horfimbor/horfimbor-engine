use std::thread;
use std::time::Duration;

use eventstore::{Client as EventClient, Client};
use futures::executor::block_on;

use tokio::time::sleep;
use uuid::Uuid;

use gyg_eventsource::model_key::ModelKey;
use gyg_eventsource::repository::EventRepository;

use crate::concurrent::{ConcurrentCommand, ConcurrentState};
use crate::simple::{SimpleCommand, SimpleState};
use crate::state_db::NoCache;

mod concurrent;
mod simple;
mod state_db;

type EasyNoCache = NoCache<SimpleState>;

#[tokio::test]
async fn easy_case() {
    let repo = EventRepository::new(get_event_db(), EasyNoCache::new());

    let key = ModelKey::new("simple_test".to_string(), Uuid::new_v4().to_string());

    let model = repo.get_model(&key).await.unwrap();

    assert_eq!(model.state(), &SimpleState { nb: 0 });

    let added = repo
        .add_command(&key, SimpleCommand::Add(17), None)
        .await
        .unwrap();

    assert_eq!(added, (SimpleState { nb: 17 }));

    let model = repo.get_model(&key).await.unwrap();

    assert_eq!(model.state(), &SimpleState { nb: 17 });

    let model = repo.get_model(&key).await.unwrap();

    assert_eq!(model.state(), &SimpleState { nb: 17 });

    repo.add_command(&key, SimpleCommand::Set(50), None)
        .await
        .unwrap();

    let model = repo.get_model(&key).await.unwrap();

    assert_eq!(model.state(), &SimpleState { nb: 50 });

    let model = repo.get_model(&key).await.unwrap();

    assert_eq!(model.state(), &SimpleState { nb: 50 });
}

type ConcurrentNoCache = NoCache<ConcurrentState>;

#[tokio::test]
async fn concurrent_case() {
    let repo = EventRepository::new(get_event_db(), ConcurrentNoCache::new());

    let key = ModelKey::new("concurrent_test".to_string(), Uuid::new_v4().to_string());

    let model = repo.get_model(&key).await.unwrap();

    assert_eq!(model.state(), &ConcurrentState { names: Vec::new() });

    {
        let repo = repo.clone();
        let key = key.clone();
        thread::spawn(move || {
            block_on(repo.add_command(
                &key,
                ConcurrentCommand::TakeTime(1, "one".to_string()),
                None,
            ))
            .unwrap();
        });
    }

    {
        let repo = repo.clone();
        let key = key.clone();
        thread::spawn(move || {
            block_on(repo.add_command(
                &key,
                ConcurrentCommand::TakeTime(2, "two".to_string()),
                None,
            ))
            .unwrap();
        });
    }

    let model = repo.get_model(&key).await.unwrap();

    assert_eq!(model.state(), &ConcurrentState { names: vec![] });

    sleep(Duration::from_millis(200)).await;

    let model = repo.get_model(&key).await.unwrap();

    assert_eq!(
        model.state(),
        &ConcurrentState {
            names: vec!["one".to_string()],
        }
    );
    sleep(Duration::from_millis(500)).await;

    let model = repo.get_model(&key).await.unwrap();

    assert_eq!(
        model.state(),
        &ConcurrentState {
            names: vec!["one".to_string(), "two".to_string()],
        }
    );
}

fn get_event_db() -> Client {
    let settings = "esdb://admin:changeit@localhost:2113?tls=false&tlsVerifyCert=false"
        .to_string()
        .parse()
        .unwrap();
    EventClient::new(settings).unwrap()
}
