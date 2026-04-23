# horfimbor-time

[![Crates.io](https://img.shields.io/crates/v/horfimbor-time.svg)](https://crates.io/crates/horfimbor-time)

Game-time calculator for Horfimbor. Converts between real-world UTC timestamps and compressed in-game time, where the game only runs during a fixed window of each real-time cycle.

## Concept

Real time is divided into repeating cycles of `irl_length` milliseconds. Within each cycle, only the first `ig_length` milliseconds count as active in-game time. During the remainder of the cycle, the game is paused and no in-game time passes.

```
IRL cycle (e.g. 24h):
|████░░░░░░░░░░░░░░░░░░░░░|████░░░░░░░░░░░░░░░░░░░░░|
 ↑                         ↑
 active (ig_length: 1h)    next cycle starts
 ↓                         ↓
 In-game time advances     In-game time frozen
```

## Quick Start

```toml
[dependencies]
horfimbor-time = "0.3"
chrono = "0.4"
```

```rust
use chrono::{Duration, TimeZone, Utc};
use horfimbor_time::{HfDuration, HfTime, HfTimeConfiguration};

// Game runs 1 hour out of every 24 hours, starting 2021-01-01 20:00 UTC
let config = HfTimeConfiguration::new(
    Duration::seconds(3600 * 24), // IRL cycle: 24 hours
    Duration::seconds(3600),      // Active window: 1 hour per cycle
    Utc.with_ymd_and_hms(2021, 1, 1, 20, 0, 0).unwrap(),
)
.expect("invalid configuration");

let now = HfTime::now(config.clone());

// How long until a building finishes (4000 in-game seconds from now)?
let build_time = HfDuration::from_seconds(4000);
let completion = now + build_time;

println!("Building finishes at: {}", completion.as_datetime().unwrap());
```

## API

### `HfTimeConfiguration`

```rust
HfTimeConfiguration::new(
    irl_length: Duration,      // real-time cycle length
    ig_length: Duration,       // active in-game window per cycle
    start_time: DateTime<Utc>, // when the game clock started
) -> Result<Self, HfTimeError>
```

Validation: `irl_length > ig_length`, both non-zero.

Accessors: `start_time()`, `irl_length() -> i64` (ms), `ig_length() -> i64` (ms).

Compute active in-game milliseconds between two real-world timestamps:
```rust
let ig_ms = config.diff_hf_millis(start_datetime, end_datetime);
```

### `HfTime`

Represents a point in real time, associated with a configuration.

```rust
let t = HfTime::now(config.clone());                       // current time
let t = HfTime::new(some_datetime, config.clone());        // from a DateTime<Utc>

t.as_millis() -> i64            // IRL milliseconds since game start
t.as_datetime() -> Option<DateTime<Utc>>  // convert back to UTC
t.as_hf_duration() -> HfDuration         // in-game time elapsed since game start
```

#### Status

Check whether the game is currently active or paused:

```rust
use horfimbor_time::HfStatus;

let (status, time_until_switch) = t.hf_status();
match status {
    HfStatus::Running => println!("Game active, pauses in {:?}", time_until_switch),
    HfStatus::Paused  => println!("Game paused, resumes in {:?}", time_until_switch),
}
```

#### Arithmetic

```rust
// Add real-world time (chrono Duration)
let later_irl = t + chrono::Duration::hours(2);

// Add in-game time — correctly skips paused windows
let later_ig = t + HfDuration::from_seconds(3600);
```

Adding `HfDuration` advances through the timeline, automatically skipping any paused periods until the full in-game duration has elapsed.

### `HfDuration`

In-game time, measured in milliseconds.

```rust
let d = HfDuration::from_milliseconds(5000);
let d = HfDuration::from_seconds(5);

d.as_milliseconds() -> i64
d.as_seconds() -> i64
```

Supports `+`, `-`, and `*`:

```rust
let doubled = d * 2;
let total   = d + HfDuration::from_seconds(10);
```

## Error Types

- `HfTimeError::InvalidLength` — `irl_length <= ig_length` or either is zero.
- `HfTimeConfigurationError::InvalidStartDate` — start timestamp is outside valid range.
