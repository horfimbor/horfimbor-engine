# horfimbor-callback-recall

[![Crates.io](https://img.shields.io/crates/v/horfimbor-callback-recall.svg)](https://crates.io/crates/horfimbor-callback-recall)

Persistent scheduled callback library. Register named async handlers, schedule them to fire at a future date with an arbitrary binary payload, and let a background poller dispatch them reliably — even across restarts.

## Features

- `sqlx_sqlite` *(default)* — SQLite backend via sqlx

## Quick Start

```toml
[dependencies]
horfimbor-callback-recall = "0.1"
tokio = { version = "1", features = ["full"] }
```

## Usage

### 1. Open a pool and build the scheduler

```rust
use horfimbor_callback_recall::{SchedulerBuilder, database::sqlite};
use std::time::Duration;

let pool = sqlite::open("sqlite://callbacks.db").await?;

let mut builder = SchedulerBuilder::new(
    pool,
    Duration::from_secs(5), // polling interval
).await?;
```

`new` runs database migrations automatically. If the migration fails the call returns an error — the service will not start with a broken database.

### 2. Register handlers

A handler is an async function that receives the raw `Vec<u8>` payload and returns `Result<(), String>`.

```rust
builder.register("send_email", |payload| async move {
    let address = String::from_utf8(payload).map_err(|e| e.to_string())?;
    println!("Sending email to {address}");
    // ... actual sending logic ...
    Ok(())
});

builder.register("expire_session", |payload| async move {
    let session_id = String::from_utf8(payload).map_err(|e| e.to_string())?;
    println!("Expiring session {session_id}");
    Ok(())
});
```

### 3. Start the scheduler

```rust
let (emitter, listener) = builder.start();
```

`start` spawns the background poller and returns two handles:
- `SchedulerEmitter` — schedule new callbacks from anywhere in your app.
- `SchedulerListener` — manage the poller lifecycle.

### 4. Schedule callbacks

```rust
use horfimbor_callback_recall::database::CallBack;
use chrono::{Utc, Duration};

let due = Utc::now() + Duration::minutes(30);
let payload = b"user@example.com".to_vec();

emitter.schedule(CallBack::new(
    "send_email".to_string(),
    payload,
    due,
)).await?;
```

### 5. Shutdown

```rust
// Block until the poller stops (only on abort/panic):
listener.join().await;

// Or stop it explicitly:
listener.shutdown();
```

## How it works

The background poller ticks at the configured `duration` interval. On each tick it queries the database for all callbacks with `status = 'pending'` and `due_date <= now + interval`. Each due callback is dispatched in its own Tokio task:

- If `due_date` is still in the future, the task sleeps until the exact moment before firing.
- On success the row is marked `fired`.
- On handler error or task panic the row is marked `failed` with the error message.

This means the effective precision of scheduling is within one polling interval.

## Custom backends

Implement the `Pool` trait to use a different database backend:

```rust
use horfimbor_callback_recall::database::{Pool, CallBack, CallBackRow};
use horfimbor_callback_recall::error::CallbackError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[derive(Clone)]
struct MyPool { /* ... */ }

#[async_trait]
impl Pool for MyPool {
    async fn migrate(&self) -> Result<(), CallbackError> { todo!() }
    async fn insert_callback(&self, cb: CallBack) -> Result<(), CallbackError> { todo!() }
    async fn fetch_due_soon(&self, due_before: DateTime<Utc>) -> Result<Vec<CallBackRow>, CallbackError> { todo!() }
    async fn mark_fired(&self, id: u32) -> Result<(), CallbackError> { todo!() }
    async fn mark_failed(&self, id: u32, error: &str) -> Result<(), CallbackError> { todo!() }
}
```

Then pass `MyPool` directly to `SchedulerBuilder::new`.

## Database schema

The SQLite backend creates one table with an index on `(status, due_date)`:

```sql
CREATE TABLE IF NOT EXISTS callbacks (
    id          INTEGER     NOT NULL PRIMARY KEY,
    identifier  TEXT        NOT NULL,   -- handler name
    payload     BLOB        NOT NULL,   -- arbitrary bytes
    due_date    TIMESTAMPTZ NOT NULL,
    status      TEXT        NOT NULL DEFAULT 'pending',  -- pending | fired | failed
    created_at  TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    fired_at    TIMESTAMPTZ,
    failed_at   TIMESTAMPTZ,
    error_msg   TEXT
);
```

## Error handling

`CallbackError` has two variants:

- `Database(String)` — a database operation failed.
- `UnknownHandler(String)` — a callback fired but no handler was registered for its identifier; the row is marked `failed`.
