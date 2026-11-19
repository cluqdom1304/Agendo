//! Waiting longer after each consecutive failure of a background loop, so a broken disk or a
//! locked file is not hammered every few seconds (A4-05).

use std::time::Duration;

/// How long to wait after `failures` failures in a row: nothing for 0, `first` for 1, doubling
/// for each further one, never more than `cap`.
pub fn delay(failures: u32, first: Duration, cap: Duration) -> Duration {
    if failures == 0 {
        return Duration::ZERO;
    }
    let factor = 1_u32.checked_shl(failures - 1).unwrap_or(u32::MAX);
    first.saturating_mul(factor).min(cap)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_delay_doubles_until_the_cap() {
        let first = Duration::from_secs(5);
        let cap = Duration::from_secs(60);
        let seen: Vec<u64> = (0..6).map(|n| delay(n, first, cap).as_secs()).collect();
        assert_eq!(seen, [0, 5, 10, 20, 40, 60]);
        assert_eq!(
            delay(1_000, first, cap),
            cap,
            "no overflow with many failures"
        );
    }
}
