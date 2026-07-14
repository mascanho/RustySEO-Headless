use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant};

/// Shared adaptive rate limiter (AIMD: additive/multiplicative increase-decrease).
///
/// Paces the *start* of outgoing requests to a single global cadence via a
/// mutex-guarded "next available slot", independent of how many requests are
/// concurrently in flight (in-flight concurrency is controlled separately by
/// a semaphore). This means a burst of concurrent tasks can't all fire at
/// once against the same host, regardless of the configured concurrency limit.
///
/// The pace speeds up gradually on successful responses and backs off sharply
/// (and honors a server-provided `Retry-After` when present) on 429/5xx
/// responses, so a crawl starts polite and only goes as fast as the target
/// server tolerates.
#[derive(Debug)]
pub struct AdaptiveRateLimiter {
    delay_ms: AtomicU64,
    min_delay_ms: u64,
    max_delay_ms: u64,
    gate: Mutex<Instant>,
}

impl AdaptiveRateLimiter {
    pub fn new(start_delay_ms: u64, min_delay_ms: u64, max_delay_ms: u64) -> Self {
        Self {
            delay_ms: AtomicU64::new(start_delay_ms.max(min_delay_ms)),
            min_delay_ms,
            max_delay_ms,
            gate: Mutex::new(Instant::now()),
        }
    }

    /// Sensible default for crawling an unknown site: start at ~5 req/s,
    /// allow ramping up to ~20 req/s on a permissive server, and allow
    /// backing off up to 30s between requests on a hostile one.
    pub fn default_polite() -> Self {
        Self::new(200, 50, 30_000)
    }

    /// Blocks until it is this caller's turn to send a request, pacing all
    /// callers to a shared global cadence.
    pub async fn acquire(&self) {
        let mut next = self.gate.lock().await;
        let now = Instant::now();
        let wait_until = if *next > now { *next } else { now };

        let base = self.delay_ms.load(Ordering::Relaxed) as f64;
        // +/- 20% jitter so request timing isn't perfectly periodic.
        let jitter_factor = 1.0 + (rand::random::<f64>() * 0.4 - 0.2);
        let interval_ms = ((base * jitter_factor) as u64).max(self.min_delay_ms);

        *next = wait_until + Duration::from_millis(interval_ms);
        let sleep_for = wait_until.saturating_duration_since(now);
        drop(next);

        if sleep_for > Duration::ZERO {
            tokio::time::sleep(sleep_for).await;
        }
    }

    /// Call after a clean, non-throttled response: gently speeds the crawl back up.
    pub fn on_success(&self) {
        let _ = self
            .delay_ms
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |d| {
                Some(((d as f64 * 0.95) as u64).max(self.min_delay_ms))
            });
    }

    /// Call after a 429/5xx response: backs off hard, honoring `Retry-After` if given.
    pub fn on_throttled(&self, retry_after: Option<Duration>) {
        let retry_after_ms = retry_after.map(|d| d.as_millis() as u64).unwrap_or(0);
        let _ = self
            .delay_ms
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |d| {
                let doubled = (d.max(self.min_delay_ms) * 2).min(self.max_delay_ms);
                Some(doubled.max(retry_after_ms).min(self.max_delay_ms))
            });
    }

    #[allow(dead_code)]
    pub fn current_delay(&self) -> Duration {
        Duration::from_millis(self.delay_ms.load(Ordering::Relaxed))
    }
}
