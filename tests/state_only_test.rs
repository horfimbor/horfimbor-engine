use std::thread;
use std::time::Duration;

use eventstore::Client as EventClient;
use futures::executor::block_on;
use tokio::time::sleep;
use uuid::Uuid;

use gyg_eventsource::model_key::ModelKey;
use gyg_eventsource::state_repository::StateRepository;

use crate::concurrent::{ConcurrentCommand, ConcurrentState};
use crate::simple::{SimpleCommand, SimpleState};

mod concurrent;
mod simple;

#[tokio::test]
async fn easy_case() {
    let repo = get_repository();

    let key = ModelKey::new("simple_test".to_string(), Uuid::new_v4().to_string());

    let model = repo.get_model::<SimpleState>(&key).await.unwrap();

    assert_eq!(model.state(), &SimpleState { nb: 0 });

    let added = repo
        .add_command::<SimpleState>(&key, SimpleCommand::Add(17), None)
        .await
        .unwrap();

    assert_eq!(added, (SimpleState { nb: 17 }));

    let model = repo.get_model::<SimpleState>(&key).await.unwrap();

    assert_eq!(model.state(), &SimpleState { nb: 17 });

    let model = repo.get_model::<SimpleState>(&key).await.unwrap();

    assert_eq!(model.state(), &SimpleState { nb: 17 });

    repo.add_command::<SimpleState>(&key, SimpleCommand::Set(50), None)
        .await
        .unwrap();

    let model = repo.get_model::<SimpleState>(&key).await.unwrap();

    assert_eq!(model.state(), &SimpleState { nb: 50 });

    let model = repo.get_model::<SimpleState>(&key).await.unwrap();

    assert_eq!(model.state(), &SimpleState { nb: 50 });
}

#[tokio::test]
async fn concurrent_case() {
    let repo = get_repository();

    let key = ModelKey::new("concurrent_test".to_string(), Uuid::new_v4().to_string());

    let model = repo.get_model::<ConcurrentState>(&key).await.unwrap();

    assert_eq!(model.state(), &ConcurrentState { names: Vec::new() });

    {
        let repo = repo.clone();
        let key = key.clone();
        thread::spawn(move || {
            block_on(repo.add_command::<ConcurrentState>(
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
            block_on(repo.add_command::<ConcurrentState>(
                &key,
                ConcurrentCommand::TakeTime(2, "two".to_string()),
                None,
            ))
            .unwrap();
        });
    }

    let model = repo.get_model::<ConcurrentState>(&key).await.unwrap();

    assert_eq!(model.state(), &ConcurrentState { names: vec![] });

    sleep(Duration::from_millis(200)).await;

    let model = repo.get_model::<ConcurrentState>(&key).await.unwrap();

    assert_eq!(
        model.state(),
        &ConcurrentState {
            names: vec!["one".to_string()],
        }
    );
    sleep(Duration::from_millis(500)).await;

    let model = repo.get_model::<ConcurrentState>(&key).await.unwrap();

    assert_eq!(
        model.state(),
        &ConcurrentState {
            names: vec!["one".to_string(), "two".to_string()],
        }
    );
}

fn get_repository() -> StateRepository {
    let settings = "esdb://admin:changeit@localhost:2113?tls=false&tlsVerifyCert=false"
        .to_string()
        .parse()
        .unwrap();
    let event_db = EventClient::new(settings).unwrap();

    let cache_db = redis::Client::open("redis://localhost:6379/").unwrap();

    let repo = StateRepository::new(event_db, cache_db);

    repo
}
