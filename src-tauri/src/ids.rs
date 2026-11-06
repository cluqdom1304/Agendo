//! Identifier generation: UUID v7 as text (SPEC §5.1), time-ordered.

use std::sync::{Arc, Mutex, PoisonError};

use uuid::{ContextV7, Timestamp, Uuid};

use crate::scheduler::clock::Clock;

pub trait IdGenerator: Send + Sync {
    fn new_id(&self) -> String;
}

/// UUID v7 built from a [`Clock`] (never the system time directly). A counter keeps ids created
/// within the same millisecond strictly increasing.
pub struct UuidV7Generator {
    clock: Arc<dyn Clock>,
    context: Mutex<ContextV7>,
}

impl UuidV7Generator {
    pub fn new(clock: Arc<dyn Clock>) -> Self {
        Self {
            clock,
            context: Mutex::new(ContextV7::new()),
        }
    }
}

impl IdGenerator for UuidV7Generator {
    fn new_id(&self) -> String {
        let now = self.clock.now();
        let seconds = u64::try_from(now.timestamp()).unwrap_or(0);
        let context = self.context.lock().unwrap_or_else(PoisonError::into_inner);
        let timestamp = Timestamp::from_unix(&*context, seconds, now.timestamp_subsec_nanos());
        Uuid::new_v7(timestamp).hyphenated().to_string()
    }
}

/// Predictable ids (`id-1`, `id-2`, …) for tests.
#[cfg(test)]
#[derive(Debug, Default)]
pub struct SequentialIds(std::sync::atomic::AtomicU64);

#[cfg(test)]
impl IdGenerator for SequentialIds {
    fn new_id(&self) -> String {
        let next = self.0.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
        format!("id-{next}")
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use chrono::Duration;

    use super::*;
    use crate::{domain::dates, scheduler::clock::ManualClock};

    fn clock_at(text: &str) -> Arc<ManualClock> {
        Arc::new(ManualClock::new(
            dates::parse_instant(text).expect("instant"),
        ))
    }

    #[test]
    fn ids_are_version_7_uuids_in_hyphenated_text() {
        let generator = UuidV7Generator::new(clock_at("2026-03-10T10:00:00Z"));
        let id = generator.new_id();

        assert_eq!(id.len(), 36);
        let parsed = Uuid::parse_str(&id).expect("valid uuid");
        assert_eq!(parsed.get_version_num(), 7);
        assert_eq!(parsed.hyphenated().to_string(), id, "lowercase hyphenated");
    }

    #[test]
    fn embedded_timestamp_comes_from_the_clock() {
        let clock = clock_at("2026-03-10T10:00:00Z");
        let generator = UuidV7Generator::new(clock.clone());
        let parsed = Uuid::parse_str(&generator.new_id()).expect("uuid");

        let (seconds, _) = parsed.get_timestamp().expect("v7 timestamp").to_unix();
        assert_eq!(i64::try_from(seconds).ok(), Some(clock.now().timestamp()));
    }

    #[test]
    fn ids_created_at_the_same_instant_are_unique_and_increasing() {
        let generator = UuidV7Generator::new(clock_at("2026-03-10T10:00:00Z"));
        let ids: Vec<String> = (0..1000).map(|_| generator.new_id()).collect();

        assert_eq!(ids.iter().collect::<HashSet<_>>().len(), ids.len());
        assert!(ids.windows(2).all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn later_instants_sort_after_earlier_ones() {
        let clock = clock_at("2026-03-10T10:00:00Z");
        let generator = UuidV7Generator::new(clock.clone());
        let first = generator.new_id();
        clock.advance(Duration::seconds(1));
        let second = generator.new_id();
        clock.advance(Duration::days(400));
        let third = generator.new_id();

        assert!(first < second && second < third);
    }

    #[test]
    fn sequential_ids_count_from_one() {
        let ids = SequentialIds::default();
        assert_eq!(ids.new_id(), "id-1");
        assert_eq!(ids.new_id(), "id-2");
    }
}
