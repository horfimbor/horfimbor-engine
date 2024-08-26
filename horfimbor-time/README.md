# Horfimbor time

This crate provide a converter between the ingame time and the "real" timestamp.


the typical use case of this library use some configuration,
get the current `HfTime`, add some in-game time ( to construct a building or whatever )
then get the remaining irl time to wait

```
use chrono::{DateTime, TimeZone, Utc};
use std::time::{Duration};
use horfimbor_time::{HfTime, HfTimeConfiguration, HfDuration};

let config = HfTimeConfiguration::new(
    Duration::from_secs(3600 * 24),
    Duration::from_secs(3600),
    Utc.with_ymd_and_hms(2021, 01, 01, 20, 0, 0).unwrap(),
)
.expect("cannot create configuration");

let time = HfTime::now(config);
let building_time = HfDuration::from_secs(4000);
let end = time + building_time;

println!("building will end at : {}", end.as_datetime().unwrap());

let tomorrow = Utc::now() + Duration::from_secs(24 * 3600);
assert!(end
    .as_datetime()
    .expect("cannot convert to datetime")
    .gt(&tomorrow))

```
