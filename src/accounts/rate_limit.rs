//! A simple limit on how often something may be done: at most `max`
//! times per `window`, per key (here: per
//! user). Kept in memory; a restart forgets it, which is fine for this.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub struct RateLimit {
    max: usize,
    window: Duration,
    hits: Mutex<HashMap<i64, Vec<Instant>>>,
}

impl RateLimit {
    pub fn new(max: usize, window: Duration) -> Self {
        Self {
            max,
            window,
            hits: Mutex::new(HashMap::new()),
        }
    }

    /// Records an attempt; false when the limit is already reached.
    pub fn allow(&self, key: i64) -> bool {
        let now = Instant::now();
        let mut hits = self.hits.lock().expect("rate limit lock");
        let recent = hits.entry(key).or_default();
        recent.retain(|at| now.duration_since(*at) < self.window);
        if recent.len() >= self.max {
            return false;
        }
        recent.push(now);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_up_to_max_per_key() {
        let limit = RateLimit::new(2, Duration::from_secs(60));
        assert!(limit.allow(1) && limit.allow(1));
        assert!(!limit.allow(1));
        assert!(limit.allow(2));
    }
}
