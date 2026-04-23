# horfimbor-eventsource

[![Crates.io](https://img.shields.io/crates/v/horfimbor-eventsource.svg)](https://crates.io/crates/horfimbor-eventsource)

Core event-sourcing library for the Horfimbor engine, built on top of [KurrentDB](https://www.kurrent.io/) with optional Redis caching.

## Features

- `cache-redis` *(default)* — Redis-backed state cache via `StateDb<S>`

## Quick Start

```toml
[dependencies]
horfimbor-eventsource = "0.4"
```

Infrastructure required: KurrentDB and (optionally) Redis. See the workspace `docker-compose.yaml`.

## Core Concepts

### Traits

Define your domain model by implementing four traits:

```rust
use horfimbor_eventsource::{Command, Event, State, StateNamed, Dto};
use horfimbor_eventsource_derive::{Command, Event, StateNamed};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const COUNTER: &str = "counter";

#[derive(Debug, Clone, Serialize, Deserialize, Event)]
#[state(COUNTER)]
pub enum CounterEvent {
    Incremented,
    Decremented,
    Set { value: i64 },
}

#[derive(Debug, Clone, Serialize, Deserialize, Command)]
#[state(COUNTER)]
pub enum CounterCommand {
    Increment,
    Decrement,
    Set(i64),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, StateNamed)]
#[state(COUNTER)]
pub struct Counter {
    pub value: i64,
}

#[derive(Debug, Error)]
pub enum CounterError {
    #[error("value cannot be negative")]
    NegativeValue,
}

impl Dto for Counter {
    type Event = CounterEvent;
    fn play_event(&mut self, event: &CounterEvent) {
        match event {
            CounterEvent::Incremented   => self.value += 1,
            CounterEvent::Decremented   => self.value -= 1,
            CounterEvent::Set { value } => self.value = *value,
        }
    }
}

impl State for Counter {
    type Command = CounterCommand;
    type Error   = CounterError;

    fn try_command(&self, cmd: CounterCommand) -> Result<Vec<CounterEvent>, CounterError> {
        match cmd {
            CounterCommand::Increment => Ok(vec![CounterEvent::Incremented]),
            CounterCommand::Decrement => {
                if self.value == 0 { return Err(CounterError::NegativeValue); }
                Ok(vec![CounterEvent::Decremented])
            }
            CounterCommand::Set(v) => Ok(vec![CounterEvent::Set { value: v }]),
        }
    }
}
```

### ModelKey

Every entity is identified by a `ModelKey` combining a stream name and a UUID:

```rust
use horfimbor_eventsource::ModelKey;

// New entity with a random time-ordered UUID
let key = ModelKey::new_uuid_v7("counter");

// Deterministic UUID from content (useful for singleton entities)
let key = ModelKey::new_uuid_v8("counter", "kind", "some-unique-data");

// Parse from string format "stream_name-uuid"
let key: ModelKey = "counter-01956b3a-...".try_into()?;
```

### Repository

Use `StateRepository` (read + write) or `DtoRepository` (read-only) to interact with KurrentDB:

```rust
use horfimbor_eventsource::{StateRepository, NoCache, Repository};
use kurrentdb::Client;

let db = Client::new("esdb://localhost:2113?tls=false".parse()?)?;
let cache = NoCache::default();
let repo = StateRepository::new(db, cache);

// Send a command — handles optimistic concurrency automatically
let updated_state = repo.add_command(&key, CounterCommand::Increment, None).await?;
println!("Counter is now: {}", updated_state.value);

// Read current state
let model = repo.get_model(&key).await?;
println!("Counter: {}", model.model.value);
```

With Redis caching:

```rust
use horfimbor_eventsource::cache_db::redis::StateDb;

let redis = redis::Client::open("redis://localhost:6379")?;
let cache = StateDb::<Counter>::new(redis);
let repo = StateRepository::new(db, cache);
```

### Streams and Subscriptions

Events are stored in per-entity streams and projected by KurrentDB into category / event-type streams:

```rust
use horfimbor_eventsource::{Stream, helper};

// Subscribe to all events for a specific entity
let sub = helper::get_subscription(&db, Stream::Model(key), 0).await?;

// Subscribe to all events across an entire category (e.g. all counters)
let sub = helper::get_subscription(&db, Stream::Stream("counter"), 0).await?;

// Subscribe to a specific event type across all streams
let sub = helper::get_subscription(&db, Stream::Event("counter.evt.incremented"), 0).await?;

// Subscribe by correlation ID (trace all events from one originating command)
let sub = helper::get_subscription(&db, Stream::Correlation(correlation_uuid), 0).await?;
```

For durable (persistent) subscriptions that survive restarts and support competing consumers:

```rust
let sub = helper::get_persistent_subscription(&db, Stream::Stream("counter"), "my-group").await?;
```

### Metadata and Event Correlation

Every event written by this library carries `Metadata` that enables KurrentDB's built-in correlation projections:

- `$correlationId` — UUID of the originating command; shared across all events in a causal chain.
- `$causationId` — UUID of the direct parent event or command.

### Cache Warming

Run a background task to keep Redis in sync with KurrentDB via a persistent subscription:

```rust
repo.cache_dto(Stream::Stream("counter"), "cache-warmer-group").await;
// This loops forever — run it in a spawned task
```

## Event and Command Naming

The derive macros generate stable, namespaced string identifiers:

| Type | Format | Example |
|---|---|---|
| Command | `"<STATE>.CMD.<VariantName>"` | `"counter.CMD.Increment"` |
| Event | `"<STATE>.evt.<variant_snake_case>"` | `"counter.evt.incremented"` |

These names are stored in KurrentDB and must remain stable. Renaming variants is a breaking change.

## Error Handling

- `EventSourceError` — database, serialization, and position errors.
- `EventSourceStateError` — wraps `EventSourceError` plus your `State::Error`.

## Integration Tests

The `tests/` directory contains complete examples:

| Test | Demonstrates |
|---|---|
| `state_only_test.rs` | Basic CRUD and concurrent command retry |
| `state_with_cache_test.rs` | Redis cache integration |
| `public_event_test.rs` | Tic-Tac-Toe with public/private event split and persistent subscriptions |

Run them with KurrentDB and Redis running:

```sh
just dc-up
cargo test
```
