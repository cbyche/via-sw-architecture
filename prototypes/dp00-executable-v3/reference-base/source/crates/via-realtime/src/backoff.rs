//! The reconnect ladder.
//!
//! Ported from `server/src/voice/reconnect-backoff.mjs`. Thirty lines upstream,
//! and the numbers in them are a contract with a rate-limited cloud endpoint:
//! `baseMs` 500, `maxMs` 10 000, `jitterRatio` 0.2, and an exponent capped at 8.
//!
//! Two things are worth stating out loud because they are easy to lose in a
//! port:
//!
//! **The cap is on the exponent, not only on the product.** `2 ** min(attempt, 8)`
//! keeps the multiplier from overflowing on a session that has been retrying for
//! hours, and `Math.min(maxMs, …)` then clamps the result. Dropping either one
//! looks harmless and is not: without the exponent cap the multiplier reaches
//! infinity, and `min(maxMs, Infinity)` happens to still be `maxMs`, so the bug
//! is invisible right up until the arithmetic is done in integers instead.
//!
//! **The randomness is injected.** Upstream takes `random = Math.random` as a
//! constructor option so its own test can pass `jitterRatio: 0`. VIA keeps the
//! seam ([`ReconnectBackoff::with_jitter_source`]) because a jittered ladder is
//! otherwise untestable, and defaults it to `getrandom`.

use std::time::Duration;

/// Default first delay. External contract — `reconnect-backoff.mjs:3`.
pub const DEFAULT_BASE_MS: u64 = 500;

/// Default ceiling. External contract — `reconnect-backoff.mjs:4`.
pub const DEFAULT_MAX_MS: u64 = 10_000;

/// Default jitter, as a fraction of the un-jittered delay. External contract —
/// `reconnect-backoff.mjs:5`.
pub const DEFAULT_JITTER_RATIO: f64 = 0.2;

/// The exponent cap. External contract — `reconnect-backoff.mjs:18`.
pub const MAX_ATTEMPT_EXPONENT: u32 = 8;

/// How long a connection must stay up before the ladder is reset.
///
/// External contract — `default-value` / *ReconnectBackoff*: *"Backoff is reset
/// only after a connection stayed up >= 10000 ms"*
/// (`realtime-gateway.mjs:1511-1517`). The rule lives with the ladder so the
/// caller that owns the reconnect loop does not have to restate it; the caller
/// still decides when to apply it.
pub const STABLE_CONNECTION_MS: u64 = 10_000;

/// How the ladder is shaped.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BackoffConfig {
    /// The first delay, and the base of the doubling.
    pub base_ms: u64,
    /// The ceiling the doubling is clamped to.
    pub max_ms: u64,
    /// Jitter as a fraction of the un-jittered delay, applied symmetrically.
    pub jitter_ratio: f64,
}

impl Default for BackoffConfig {
    fn default() -> Self {
        Self {
            base_ms: DEFAULT_BASE_MS,
            max_ms: DEFAULT_MAX_MS,
            jitter_ratio: DEFAULT_JITTER_RATIO,
        }
    }
}

/// A source of uniform values in `[0, 1)` — upstream's `random` option.
pub trait JitterSource: Send + Sync {
    /// The next value in `[0, 1)`.
    fn next_unit(&mut self) -> f64;
}

impl<F> JitterSource for F
where
    F: FnMut() -> f64 + Send + Sync,
{
    fn next_unit(&mut self) -> f64 {
        self()
    }
}

/// `getrandom`-backed uniform values, the stand-in for `Math.random`.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemJitter;

impl JitterSource for SystemJitter {
    fn next_unit(&mut self) -> f64 {
        let mut bytes = [0u8; 8];
        if getrandom::fill(&mut bytes).is_err() {
            // A CSPRNG failure must not turn a reconnect into a panic. The
            // un-jittered ladder is still a correct ladder — it is exactly what
            // `jitterRatio: 0` produces — so the fallback degrades the spread,
            // not the behaviour.
            return 0.5;
        }
        // 53 significant bits is the whole of an f64 mantissa, which is what
        // `Math.random` returns and what keeps the result strictly below 1.
        let value = u64::from_le_bytes(bytes) >> 11;
        (value as f64) / ((1u64 << 53) as f64)
    }
}

/// The exponential-with-jitter reconnect ladder.
///
/// ```
/// use via_realtime::{BackoffConfig, ReconnectBackoff};
///
/// // Upstream's own test: doubling with a hard cap, and `reset()` returns to
/// // the base.
/// let mut backoff = ReconnectBackoff::with_jitter_source(
///     BackoffConfig { base_ms: 500, max_ms: 2000, jitter_ratio: 0.0 },
///     || 0.0,
/// );
/// let ladder: Vec<u64> = (0..4).map(|_| backoff.next().as_millis() as u64).collect();
/// assert_eq!(ladder, [500, 1000, 2000, 2000]);
///
/// backoff.reset();
/// assert_eq!(backoff.next().as_millis(), 500);
/// ```
pub struct ReconnectBackoff {
    config: BackoffConfig,
    attempt: u32,
    jitter: Box<dyn JitterSource>,
}

impl core::fmt::Debug for ReconnectBackoff {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ReconnectBackoff")
            .field("config", &self.config)
            .field("attempt", &self.attempt)
            .finish_non_exhaustive()
    }
}

impl Default for ReconnectBackoff {
    fn default() -> Self {
        Self::new(BackoffConfig::default())
    }
}

impl ReconnectBackoff {
    /// A ladder with the system jitter source.
    #[must_use]
    pub fn new(config: BackoffConfig) -> Self {
        Self::with_jitter_source(config, SystemJitter)
    }

    /// A ladder with an injected jitter source.
    ///
    /// This is upstream's `random` constructor option. A closure works:
    /// `with_jitter_source(config, || 0.0)`.
    #[must_use]
    pub fn with_jitter_source(config: BackoffConfig, jitter: impl JitterSource + 'static) -> Self {
        Self {
            config,
            attempt: 0,
            jitter: Box::new(jitter),
        }
    }

    /// How this ladder is shaped.
    #[must_use]
    pub const fn config(&self) -> BackoffConfig {
        self.config
    }

    /// How many delays have been handed out since the last [`reset`](Self::reset).
    #[must_use]
    pub const fn attempt(&self) -> u32 {
        self.attempt
    }

    /// The next delay, and advance the ladder.
    ///
    /// External contract — `default-value` / *ReconnectBackoff*:
    /// `round(min(maxMs, base * 2^min(attempt, 8)) + exponential * jitterRatio *
    /// (random() * 2 - 1))`, floored at 0.
    ///
    /// The floor matters when `jitter_ratio > 1`, which nothing ships but which
    /// a host extension may configure; the arithmetic is done in `f64` for the
    /// same reason upstream does it there — the jitter term is fractional.
    // Upstream's own method name, and the ladder really is a sequence — but it
    // is an infinite one, so `Iterator` would force every caller through an
    // `Option` that is never `None`. The inherent name stays.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Duration {
        let exponent = self.attempt.min(MAX_ATTEMPT_EXPONENT);
        let doubled = (self.config.base_ms as f64) * f64::from(2u32.pow(exponent));
        let exponential = (self.config.max_ms as f64).min(doubled);
        self.attempt = self.attempt.saturating_add(1);
        let jitter =
            exponential * self.config.jitter_ratio * ((self.jitter.next_unit() * 2.0) - 1.0);
        let millis = (exponential + jitter).round().max(0.0);
        Duration::from_millis(millis as u64)
    }

    /// Return to the first rung.
    ///
    /// Called after a connection stayed up for at least
    /// [`STABLE_CONNECTION_MS`]; a flapping connection keeps climbing.
    pub fn reset(&mut self) {
        self.attempt = 0;
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn ladder(config: BackoffConfig, unit: f64, rungs: usize) -> Vec<u64> {
        let mut backoff = ReconnectBackoff::with_jitter_source(config, move || unit);
        (0..rungs)
            .map(|_| backoff.next().as_millis() as u64)
            .collect()
    }

    #[test]
    fn the_default_ladder_is_the_catalogued_one() {
        let config = BackoffConfig::default();
        assert_eq!(config.base_ms, 500);
        assert_eq!(config.max_ms, 10_000);
        assert!((config.jitter_ratio - 0.2).abs() < f64::EPSILON);
        assert_eq!(
            ladder(config, 0.5, 8),
            [500, 1000, 2000, 4000, 8000, 10_000, 10_000, 10_000]
        );
    }

    #[test]
    fn upstreams_own_ladder_test() {
        assert_eq!(
            ladder(
                BackoffConfig {
                    base_ms: 500,
                    max_ms: 2000,
                    jitter_ratio: 0.0
                },
                0.0,
                4
            ),
            [500, 1000, 2000, 2000]
        );
    }

    #[test]
    fn reset_returns_to_the_base() {
        let mut backoff = ReconnectBackoff::with_jitter_source(
            BackoffConfig {
                base_ms: 250,
                max_ms: 2000,
                jitter_ratio: 0.0,
            },
            || 0.0,
        );
        backoff.next();
        backoff.next();
        assert_eq!(backoff.attempt(), 2);
        backoff.reset();
        assert_eq!(backoff.attempt(), 0);
        assert_eq!(backoff.next().as_millis(), 250);
    }

    #[test]
    fn jitter_is_symmetric_about_the_un_jittered_delay() {
        let config = BackoffConfig {
            base_ms: 1000,
            max_ms: 10_000,
            jitter_ratio: 0.2,
        };
        // random() == 0 is the low end, 1 the high end, 0.5 the middle.
        assert_eq!(ladder(config, 0.0, 1), [800]);
        assert_eq!(ladder(config, 0.5, 1), [1000]);
        assert_eq!(ladder(config, 1.0, 1), [1200]);
    }

    #[test]
    fn the_jitter_is_a_fraction_of_the_clamped_delay_not_the_raw_one() {
        // Rung 4 would be 8000 un-clamped; the ceiling is 2000, so the ±20% is
        // ±400 rather than ±1600.
        let config = BackoffConfig {
            base_ms: 1000,
            max_ms: 2000,
            jitter_ratio: 0.2,
        };
        assert_eq!(ladder(config, 0.0, 4), [800, 1600, 1600, 1600]);
    }

    #[test]
    fn the_exponent_is_capped_at_eight() {
        // 2^8 = 256, so with base 1 and no ceiling the ladder tops out at 256
        // rather than growing without bound.
        let config = BackoffConfig {
            base_ms: 1,
            max_ms: u64::MAX,
            jitter_ratio: 0.0,
        };
        let rungs = ladder(config, 0.0, 11);
        assert_eq!(rungs[8], 256);
        assert_eq!(rungs[9], 256);
        assert_eq!(rungs[10], 256);
    }

    #[test]
    fn a_ratio_over_one_is_floored_at_zero_rather_than_going_negative() {
        let config = BackoffConfig {
            base_ms: 1000,
            max_ms: 10_000,
            jitter_ratio: 2.0,
        };
        assert_eq!(ladder(config, 0.0, 1), [0]);
    }

    #[test]
    fn the_attempt_counter_saturates_instead_of_wrapping() {
        let mut backoff = ReconnectBackoff::with_jitter_source(
            BackoffConfig {
                base_ms: 500,
                max_ms: 10_000,
                jitter_ratio: 0.0,
            },
            || 0.0,
        );
        backoff.attempt = u32::MAX;
        assert_eq!(backoff.next().as_millis(), 10_000);
        assert_eq!(backoff.attempt(), u32::MAX);
    }

    #[test]
    fn the_system_jitter_source_stays_inside_the_unit_interval() {
        let mut source = SystemJitter;
        for _ in 0..256 {
            let value = source.next_unit();
            assert!((0.0..1.0).contains(&value), "{value}");
        }
    }

    #[tokio::test(start_paused = true)]
    async fn the_ladder_is_a_real_sleep_schedule() {
        // The whole point of the ladder is elapsed time, so assert on the clock
        // rather than only on the returned numbers. Paused time makes the eight
        // rungs — 25.5 seconds of wall clock — run in microseconds.
        let mut backoff = ReconnectBackoff::with_jitter_source(
            BackoffConfig {
                base_ms: 500,
                max_ms: 10_000,
                jitter_ratio: 0.0,
            },
            || 0.0,
        );
        let start = tokio::time::Instant::now();
        for _ in 0..8 {
            tokio::time::sleep(backoff.next()).await;
        }
        assert_eq!(
            start.elapsed(),
            Duration::from_millis(500 + 1000 + 2000 + 4000 + 8000 + 10_000 + 10_000 + 10_000)
        );
    }
}
