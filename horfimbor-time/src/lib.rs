#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

use chrono::{DateTime, Duration, Utc};
use core::ops::Add;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// `HfTime` can fail to construct.
#[derive(Error, Debug)]
pub enum HfTimeError {
    /// in game time must be slower than real time
    #[error("loop length must be greater than length and non zero")]
    InvalidLength,
}

/// `HfTimeConfiguration` can be invalid.
#[derive(Error, Debug)]
pub enum HfTimeConfigurationError {
    /// in game time must be slower than real time
    #[error("start date is out of bound")]
    InvalidStartDate,
}

/// the in-game time is just a wrapper around an integer
#[derive(Copy, Clone, Debug)]
pub struct HfDuration {
    value: i64,
}

impl HfDuration {
    /// the baseline for a web game is the millisecond
    #[must_use]
    pub const fn milliseconds(value: i64) -> Self {
        Self { value }
    }

    /// can be easier to work with seconds
    #[must_use]
    pub const fn seconds(value: i64) -> Self {
        Self {
            value: value * 1000,
        }
    }
}

impl Add<Self> for HfDuration {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value + rhs.value,
        }
    }
}

/// configuration is shared across all service for the same server
/// it defines how long the game is up and when it started
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HfTimeConfiguration {
    start_time: i64,
    irl_length: i64,
    ig_length: i64,
}

impl HfTimeConfiguration {
    /// # Errors
    ///
    /// Will return `Err` if configuration is invalid
    pub fn new(
        irl_length: Duration,
        ig_length: Duration,
        start_time: DateTime<Utc>,
    ) -> Result<Self, HfTimeError> {
        if irl_length.le(&ig_length) || irl_length.is_zero() || ig_length.is_zero() {
            return Err(HfTimeError::InvalidLength);
        }

        Ok(Self {
            start_time: start_time.timestamp_millis(),
            irl_length: irl_length.num_milliseconds(),
            ig_length: ig_length.num_milliseconds(),
        })
    }

    /// get start date as UTC value
    /// # Errors
    ///
    /// Will return `Err` if the start date cannot be converted to UTC
    pub fn start_time(&self) -> Result<DateTime<Utc>, HfTimeConfigurationError> {
        DateTime::from_timestamp_millis(self.start_time)
            .ok_or(HfTimeConfigurationError::InvalidStartDate)
    }

    /// get irl duration in milliseconds
    #[must_use]
    pub const fn irl_length(&self) -> i64 {
        self.irl_length
    }

    /// get in game duration in milliseconds
    #[must_use]
    pub const fn ig_length(&self) -> i64 {
        self.ig_length
    }
}

/// `HfTime` allow to convert in-game time and irl time based on a config
#[derive(Debug)]
pub struct HfTime {
    time: i64,
    config: HfTimeConfiguration,
}

impl HfTime {
    /// it is possible to create an `HfTime` from anypoint in time
    #[must_use]
    pub const fn new(time: DateTime<Utc>, config: HfTimeConfiguration) -> Self {
        Self {
            time: time.timestamp_millis() - config.start_time,
            config,
        }
    }

    /// reduce the boilerplate
    #[must_use]
    pub fn now(config: HfTimeConfiguration) -> Self {
        let start = Utc::now();
        Self::new(start, config)
    }

    /// return the irl time since the beginning.config
    #[must_use]
    pub const fn as_millis(&self) -> i64 {
        self.time
    }

    /// allow to display when an event will finnish
    #[must_use]
    pub const fn as_datetime(&self) -> Option<DateTime<Utc>> {
        DateTime::from_timestamp_millis(self.time + self.config.start_time)
    }

    /// return the time passed when the game is up since the beginning.config
    #[must_use]
    pub const fn as_hf_duration(&self) -> HfDuration {
        HfDuration {
            value: self.as_hf_millis(),
        }
    }

    const fn as_hf_millis(&self) -> i64 {
        let nb_loop = self.time / self.config.irl_length;
        let rest = self.time % self.config.irl_length;
        if rest > self.config.ig_length {
            return (nb_loop + 1) * self.config.ig_length;
        }
        nb_loop * self.config.ig_length + rest
    }
}

#[cfg(test)]
mod test_new {
    use super::*;

    #[test]
    fn test_first_iterations() {
        let config = HfTimeConfiguration::new(
            Duration::milliseconds(10),
            Duration::milliseconds(3),
            DateTime::default(),
        )
        .expect("cannot create configuration");

        // example of how we want HfTime to pass.
        let vals = vec![
            (1, 1),
            (2, 2),
            (3, 3),
            (4, 3),
            (5, 3),
            (6, 3),
            (7, 3),
            (8, 3),
            (9, 3),
            (10, 3),
            (11, 4),
            (12, 5),
            (13, 6),
            (14, 6),
        ];

        for v in vals.iter() {
            let from_time = HfTime::new(
                DateTime::from_timestamp_millis(v.0).expect("cannot create timestamp"),
                config,
            );
            assert_eq!(from_time.as_hf_millis(), v.1);
        }
    }

    #[test]
    fn test_creation_from_time() {
        let config = HfTimeConfiguration::new(
            Duration::seconds(1),
            Duration::milliseconds(500),
            DateTime::default(),
        )
        .expect("cannot create configuration");

        let from_time = HfTime::new(
            DateTime::from_timestamp_millis(1200).expect("cannot create timestamp"),
            config,
        );
        assert_eq!(from_time.as_millis(), 1200);
        assert_eq!(from_time.as_hf_millis(), 700);
    }

    #[test]
    fn test_creation_with_start_time() {
        let config = HfTimeConfiguration::new(
            Duration::seconds(1),
            Duration::milliseconds(500),
            DateTime::default(),
        )
        .expect("cannot create configuration");

        let from_time = HfTime::new(
            DateTime::from_timestamp_millis(1200).expect("cannot create timestamp"),
            config,
        );
        assert_eq!(from_time.as_millis(), 1200);
        assert_eq!(from_time.as_hf_millis(), 700);
    }
}

impl Add<Duration> for HfTime {
    type Output = Self;

    #[allow(clippy::cast_possible_truncation)]
    fn add(self, rhs: Duration) -> Self {
        Self {
            time: self.time + rhs.num_milliseconds(),
            config: self.config,
        }
    }
}

impl Add<HfDuration> for HfTime {
    type Output = Self;

    fn add(self, rhs: HfDuration) -> Self {
        let mut nb_loop = self.time / self.config.irl_length;
        nb_loop += rhs.value / self.config.ig_length;

        let mut current_rest = self.time % self.config.irl_length;

        if current_rest > self.config.ig_length {
            nb_loop += 1;
            current_rest = 0;
        }

        let mut rest = rhs.value % self.config.ig_length;
        if current_rest + rest > self.config.ig_length {
            nb_loop += 1;
            current_rest = 0;
            rest = current_rest + rest - self.config.ig_length;
        }

        let time = nb_loop * self.config.irl_length + current_rest + rest;

        Self {
            time,
            config: self.config,
        }
    }
}
#[cfg(test)]
mod test_add {
    use super::*;

    #[test]
    fn test_add_only_full_loop() {
        let config = HfTimeConfiguration::new(
            Duration::milliseconds(120),
            Duration::milliseconds(60),
            DateTime::default(),
        )
        .expect("cannot create configuration");
        let mut time = HfTime::new(
            DateTime::from_timestamp_millis(0).expect("cannot create timestamp"),
            config,
        );

        time = time + Duration::milliseconds(120 * 5);
        assert_eq!(time.as_hf_millis(), 60 * 5);
        assert_eq!(time.as_millis(), 120 * 5);

        time = time + HfDuration::milliseconds(60 * 3);
        assert_eq!(time.as_hf_millis(), 60 * 8);
        assert_eq!(time.as_millis(), 120 * 8);
    }

    #[test]
    fn test_add_full_loop_during_length() {
        let config = HfTimeConfiguration::new(
            Duration::milliseconds(100),
            Duration::milliseconds(30),
            DateTime::default(),
        )
        .expect("cannot create configuration");
        let mut time = HfTime::new(
            DateTime::from_timestamp_millis(15).expect("cannot create timestamp"),
            config,
        );

        time = time + Duration::milliseconds(100 * 2);
        assert_eq!(time.as_hf_millis(), 75);
        assert_eq!(time.as_millis(), 215);

        time = time + HfDuration::milliseconds(30);
        assert_eq!(time.as_hf_millis(), 105);
        assert_eq!(time.as_millis(), 315);
    }

    #[test]
    fn test_add_full_loop_after_length() {
        let config = HfTimeConfiguration::new(
            Duration::milliseconds(100),
            Duration::milliseconds(30),
            DateTime::default(),
        )
        .expect("cannot create configuration");
        let mut time = HfTime::new(
            DateTime::from_timestamp_millis(50).expect("cannot create timestamp"),
            config,
        );

        time = time + Duration::milliseconds(100);
        assert_eq!(time.as_hf_millis(), 60);
        assert_eq!(time.as_millis(), 150);

        time = time + HfDuration::milliseconds(30);
        assert_eq!(time.as_hf_millis(), 90);
        assert_eq!(time.as_millis(), 300);
    }

    #[test]
    fn test_add_partial_after_length() {
        let config = HfTimeConfiguration::new(
            Duration::milliseconds(1000),
            Duration::milliseconds(100),
            DateTime::default(),
        )
        .expect("cannot create configuration");
        let mut time = HfTime::new(
            DateTime::from_timestamp_millis(500).expect("cannot create timestamp"),
            config,
        );

        time = time + HfDuration::milliseconds(10);
        assert_eq!(time.as_hf_millis(), 110);
        assert_eq!(time.as_millis(), 1010);
    }
}

#[cfg(test)]
mod test_creation_after_epoch {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_create_millennium() {
        let config = HfTimeConfiguration::new(
            Duration::seconds(3600 * 24),
            Duration::seconds(3600 * 2),
            Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap(),
        )
        .expect("cannot create configuration");

        let time = HfTime::new(Utc::now(), config);

        assert!(time.as_millis() < 20 * 365 * 24 * 60 * 60 * 1000)
    }
}
