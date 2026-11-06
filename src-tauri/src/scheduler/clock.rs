//! The only source of "now" in the application (D-04).
//!
//! Nothing else may call `Utc::now()` or `Local::now()`: everything that needs the time takes a
//! [`Clock`], so tests advance a [`ManualClock`] instead of sleeping.

use std::sync::{Mutex, PoisonError};

use chrono::{DateTime, Duration, Utc};

use crate::domain::dates;

pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;

    /// `now` as the stored text form, `YYYY-MM-DDTHH:MM:SSZ`.
    fn now_string(&self) -> String {
        dates::format_instant(self.now())
    }
}

/// The real clock.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// A clock that only moves when told to.
#[derive(Debug)]
pub struct ManualClock {
    current: Mutex<DateTime<Utc>>,
}

impl ManualClock {
    pub fn new(start: DateTime<Utc>) -> Self {
        Self {
            current: Mutex::new(start),
        }
    }

    pub fn set(&self, instant: DateTime<Utc>) {
        *self.current.lock().unwrap_or_else(PoisonError::into_inner) = instant;
    }

    pub fn advance(&self, by: Duration) {
        let mut current = self.current.lock().unwrap_or_else(PoisonError::into_inner);
        *current += by;
    }
}

impl Clock for ManualClock {
    fn now(&self) -> DateTime<Utc> {
        *self.current.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    fn instant(text: &str) -> DateTime<Utc> {
        dates::parse_instant(text).expect("instant")
    }

    #[test]
    fn manual_clock_stays_put_until_advanced() {
        let clock = ManualClock::new(instant("2026-03-10T10:00:00Z"));
        assert_eq!(clock.now_string(), "2026-03-10T10:00:00Z");
        assert_eq!(clock.now_string(), "2026-03-10T10:00:00Z");

        clock.advance(Duration::minutes(90));
        assert_eq!(clock.now_string(), "2026-03-10T11:30:00Z");
    }

    #[test]
    fn manual_clock_can_be_set_backwards() {
        let clock = ManualClock::new(instant("2026-03-10T10:00:00Z"));
        clock.set(instant("2026-01-01T00:00:00Z"));
        assert_eq!(clock.now(), instant("2026-01-01T00:00:00Z"));
    }

    #[test]
    fn manual_clock_works_behind_a_trait_object() {
        let clock: Arc<dyn Clock> = Arc::new(ManualClock::new(instant("2026-03-10T10:00:00Z")));
        assert_eq!(clock.now_string(), "2026-03-10T10:00:00Z");
    }

    #[test]
    fn system_clock_is_close_to_the_real_time() {
        let before = Utc::now();
        let reading = SystemClock.now();
        let after = Utc::now();
        assert!(before <= reading && reading <= after);
        assert_eq!(SystemClock.now_string().len(), 20);
    }
}
