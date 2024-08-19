use core::ops::Add;
use std::time::{Duration, SystemTime, SystemTimeError, UNIX_EPOCH};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HfTimeError {
    #[error("loop lengh must be greater than lengh")]
    InvalidLength,
}

#[derive(Copy, Clone, Debug)]
pub struct HfDuration {
    value: u128,
}

impl HfDuration {
    #[must_use]
    pub const fn from_millis(value: u128) -> Self {
        Self { value }
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

#[derive(Copy, Clone, Debug)]
pub struct HfTimeConfiguration {
    loop_length: u128,
    length: u128,
}

impl HfTimeConfiguration {
    /// # Errors
    ///
    /// Will return `Err` if configuration is invalid
    pub const fn new(loop_length: u128, length: u128) -> Result<Self, HfTimeError> {
        if loop_length < length {
            return Err(HfTimeError::InvalidLength);
        }

        Ok(Self {
            loop_length,
            length,
        })
    }
}

#[derive(Debug)]
pub struct HfTime {
    time: u128,
    config: HfTimeConfiguration,
}

impl HfTime {
    /// # Errors
    ///
    /// Will return an `Err` if time goes backward for `duration_since`.
    pub fn now(config: HfTimeConfiguration) -> Result<Self, SystemTimeError> {
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH)?;
        Ok(Self::new(since_the_epoch, config))
    }

    #[must_use]
    pub const fn new(time: Duration, config: HfTimeConfiguration) -> Self {
        Self {
            time: time.as_millis(),
            config,
        }
    }

    #[must_use]
    pub const fn hf_new(time: &HfDuration, config: HfTimeConfiguration) -> Self {
        let nb_loop = time.value / config.length;
        let rest = time.value % config.length;
        Self {
            time: nb_loop * config.loop_length + rest,
            config,
        }
    }
    #[must_use]
    pub const fn as_millis(&self) -> u128 {
        self.time
    }
    #[must_use]
    pub const fn as_hf_millis(&self) -> u128 {
        let nb_loop = self.time / self.config.loop_length;
        let rest = self.time % self.config.loop_length;
        if rest > self.config.length {
            return (nb_loop + 1) * self.config.length;
        }
        nb_loop * self.config.length + rest
    }
    #[must_use]
    pub const fn as_hf_duration(&self) -> HfDuration {
        HfDuration {
            value: self.as_hf_millis(),
        }
    }
}

#[cfg(test)]
mod test_new {
    use super::*;

    #[test]
    fn test_start() {
        let config = HfTimeConfiguration::new(10, 3).expect("cannot create configuration");

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
            let from_time = HfTime::new(Duration::from_millis(v.0), config);
            assert_eq!(from_time.as_hf_millis(), v.1);
        }
    }
    #[test]
    fn test_creation() {
        let config = HfTimeConfiguration::new(1000, 500).expect("cannot create configuration");

        let from_time = HfTime::new(Duration::from_millis(1200), config);
        assert_eq!(from_time.as_millis(), 1200);
        assert_eq!(from_time.as_hf_millis(), 700);

        let from_time = HfTime::hf_new(&HfDuration::from_millis(1200), config);
        assert_eq!(from_time.as_millis(), 2200);
        assert_eq!(from_time.as_hf_millis(), 1200);
    }
}

impl Add<Duration> for HfTime {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self {
        Self {
            time: self.time + rhs.as_millis(),
            config: self.config,
        }
    }
}

impl Add<HfDuration> for HfTime {
    type Output = Self;

    fn add(self, rhs: HfDuration) -> Self {
        let current_loop = self.time / self.config.loop_length;
        let current_rest = self.time % self.config.loop_length;

        let nb_loop = rhs.value / self.config.length;
        let rest = rhs.value % self.config.length;

        let time = (current_loop + nb_loop) * self.config.loop_length + current_rest + rest;

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
    fn test_edge() {
        let config = HfTimeConfiguration::new(1000, 500).expect("cannot create configuration");

        let mut time = HfTime::new(Duration::ZERO, config);

        time = time + HfDuration::from_millis(400);
        assert_eq!(time.as_millis(), 400);
        assert_eq!(time.as_hf_millis(), 400);

        time = time + Duration::from_millis(200);
        assert_eq!(time.as_millis(), 600);
        assert_eq!(time.as_hf_millis(), 500);

        time = time + HfDuration::from_millis(500);
        assert_eq!(time.as_millis(), 1600);
        assert_eq!(time.as_hf_millis(), 1000);
    }

    #[test]
    fn test_add() {
        let config = HfTimeConfiguration::new(1000, 500).expect("cannot create configuration");

        let mut time = HfTime::new(Duration::ZERO, config);

        assert_eq!(time.as_millis(), 0);

        time = time + Duration::from_secs(100);
        assert_eq!(time.as_millis(), 100000);
        assert_eq!(time.as_hf_millis(), 50000);

        time = time + HfDuration::from_millis(1400);
        assert_eq!(time.as_millis(), 102400);
        assert_eq!(time.as_hf_millis(), 51400);

        time = time + Duration::from_millis(500);
        assert_eq!(time.as_millis(), 102900);
        assert_eq!(time.as_hf_millis(), 51500);

        time = time + HfDuration::from_millis(500);
        assert_eq!(time.as_millis(), 103900);
        assert_eq!(time.as_hf_millis(), 52000);
    }
}
