use std::thread;
use std::time::Duration;

use eventstore::{Client as EventClient, Client};
use futures::executor::block_on;

use gyg_eventsource::cache_db::NoCache;
use tokio::time::sleep;
use uuid::Uuid;

use gyg_eventsource::model_key::ModelKey;
use gyg_eventsource::repository::Repository;
use gyg_eventsource::repository::{DtoRepository, StateRepository};

use crate::concurrent::{ConcurrentCommand, ConcurrentState};
use crate::simple::{SimpleCommand, SimpleNbAddDto, SimpleState};

mod concurrent;
mod simple;
mod state_db;

type EasyNoCacheState = NoCache<SimpleState>;
type EasyNoCacheDto = NoCache<SimpleNbAddDto>;

#[tokio::test]
async fn easy_case() {
    let repo_state = StateRepository::new(get_event_db(), EasyNoCacheState::new());

    let repo_dto = DtoRepository::new(get_event_db(), EasyNoCacheDto::new());

    let key = ModelKey::new("simple_test".to_string(), Uuid::new_v4().to_string());

    // test empty data :

    let model = repo_state.get_model(&key).await.unwrap();
    assert_eq!(model.state(), &SimpleState { nb: 0 });

    let dto = repo_dto.get_model(&key).await.unwrap();
    assert_eq!(dto.state(), &SimpleNbAddDto { nb: 0 });

    // test by adding 17

    let added = repo_state
        .add_command(&key, SimpleCommand::Add(17), None)
        .await
        .unwrap();

    assert_eq!(added, (SimpleState { nb: 17 }));

    let model = repo_state.get_model(&key).await.unwrap();
    assert_eq!(model.state(), &SimpleState { nb: 17 });

    let dto = repo_dto.get_model(&key).await.unwrap();
    assert_eq!(dto.state(), &SimpleNbAddDto { nb: 1 });

    // test by setting 50

    repo_state
        .add_command(&key, SimpleCommand::Set(50), None)
        .await
        .unwrap();

    let model = repo_state.get_model(&key).await.unwrap();
    assert_eq!(model.state(), &SimpleState { nb: 50 });

    let dto = repo_dto.get_model(&key).await.unwrap();
    assert_eq!(dto.state(), &SimpleNbAddDto { nb: 2 });

    // test by setting 50 another time

    repo_state
        .add_command(&key, SimpleCommand::Set(50), None)
        .await
        .unwrap();

    let model = repo_state.get_model(&key).await.unwrap();
    assert_eq!(model.state(), &SimpleState { nb: 50 });

    let dto = repo_dto.get_model(&key).await.unwrap();
    assert_eq!(dto.state(), &SimpleNbAddDto { nb: 3 });
}

type ConcurrentNoCache = NoCache<ConcurrentState>;

#[tokio::test]
async fn concurrent_case() {
    let repo = StateRepository::new(get_event_db(), ConcurrentNoCache::new());

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
