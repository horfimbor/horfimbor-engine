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
    irl_length: u128,
    ig_length: u128,
}

impl HfTimeConfiguration {
    /// # Errors
    ///
    /// Will return `Err` if configuration is invalid
    pub fn new(irl_length: Duration, ig_length: Duration) -> Result<Self, HfTimeError> {
        if irl_length.lt(&ig_length) {
            return Err(HfTimeError::InvalidLength);
        }

        Ok(Self {
            irl_length: irl_length.as_millis(),
            ig_length: ig_length.as_millis(),
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
        let nb_loop = time.value / config.ig_length;
        let rest = time.value % config.ig_length;
        Self {
            time: nb_loop * config.irl_length + rest,
            config,
        }
    }
    #[must_use]
    pub const fn as_millis(&self) -> u128 {
        self.time
    }
    #[must_use]
    pub const fn as_hf_millis(&self) -> u128 {
        let nb_loop = self.time / self.config.irl_length;
        let rest = self.time % self.config.irl_length;
        if rest > self.config.ig_length {
            return (nb_loop + 1) * self.config.ig_length;
        }
        nb_loop * self.config.ig_length + rest
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
    fn test_first_iterations() {
        let config = HfTimeConfiguration::new(Duration::from_millis(10), Duration::from_millis(3))
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
            let from_time = HfTime::new(Duration::from_millis(v.0), config);
            assert_eq!(from_time.as_hf_millis(), v.1);
        }
    }

    #[test]
    fn test_creation_from_time() {
        let config = HfTimeConfiguration::new(Duration::from_secs(1), Duration::from_millis(500))
            .expect("cannot create configuration");

        let from_time = HfTime::new(Duration::from_millis(1200), config);
        assert_eq!(from_time.as_millis(), 1200);
        assert_eq!(from_time.as_hf_millis(), 700);
    }
    #[test]
    fn test_creation_from_hf_time() {
        let config = HfTimeConfiguration::new(Duration::from_secs(1), Duration::from_millis(500))
            .expect("cannot create configuration");

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
        let config =
            HfTimeConfiguration::new(Duration::from_millis(120), Duration::from_millis(60))
                .expect("cannot create configuration");
        let mut time = HfTime::new(Duration::ZERO, config);

        time = time + Duration::from_millis(120 * 5);
        assert_eq!(time.as_hf_millis(), 60 * 5);
        assert_eq!(time.as_millis(), 120 * 5);

        time = time + HfDuration::from_millis(60 * 3);
        assert_eq!(time.as_hf_millis(), 60 * 8);
        assert_eq!(time.as_millis(), 120 * 8);
    }

    #[test]
    fn test_add_full_loop_during_length() {
        let config =
            HfTimeConfiguration::new(Duration::from_millis(100), Duration::from_millis(30))
                .expect("cannot create configuration");
        let mut time = HfTime::new(Duration::from_millis(15), config);

        time = time + Duration::from_millis(100 * 2);
        assert_eq!(time.as_hf_millis(), 75);
        assert_eq!(time.as_millis(), 215);

        time = time + HfDuration::from_millis(30);
        assert_eq!(time.as_hf_millis(), 105);
        assert_eq!(time.as_millis(), 315);
    }

    #[test]
    fn test_add_full_loop_after_length() {
        let config =
            HfTimeConfiguration::new(Duration::from_millis(100), Duration::from_millis(30))
                .expect("cannot create configuration");
        let mut time = HfTime::new(Duration::from_millis(50), config);

        time = time + Duration::from_millis(100);
        assert_eq!(time.as_hf_millis(), 60);
        assert_eq!(time.as_millis(), 150);

        time = time + HfDuration::from_millis(30);
        assert_eq!(time.as_hf_millis(), 90);
        assert_eq!(time.as_millis(), 300);
    }

    #[test]
    fn test_add_partial_after_length() {
        let config =
            HfTimeConfiguration::new(Duration::from_millis(1000), Duration::from_millis(100))
                .expect("cannot create configuration");
        let mut time = HfTime::new(Duration::from_millis(500), config);

        time = time + HfDuration::from_millis(10);
        assert_eq!(time.as_hf_millis(), 110);
        assert_eq!(time.as_millis(), 1010);
    }
}
