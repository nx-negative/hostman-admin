//! In-memory sliding-window rate limiter (per key, e.g. IP).

use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

pub struct RateLimiter {
    max: usize,
    window: Duration,
    hits: std::sync::Mutex<HashMap<String, Vec<Instant>>>,
}

impl RateLimiter {
    pub fn new(max: usize, window: Duration) -> Self {
        Self {
            max,
            window,
            hits: std::sync::Mutex::new(HashMap::new()),
        }
    }

    pub fn per_minute(max: usize) -> Self {
        Self::new(max, Duration::from_secs(60))
    }

    /// Records a hit; false = over limit.
    pub fn check(&self, key: &str) -> bool {
        let mut hits = self.hits.lock().unwrap();
        let now = Instant::now();
        let v = hits.entry(key.to_string()).or_default();
        v.retain(|t| now.duration_since(*t) < self.window);
        if v.len() >= self.max {
            return false;
        }
        v.push(now);
        true
    }
}

/// Login lockout: 5 failures → locked 15 min (Appendix C).
pub struct Lockout {
    max_fails: u32,
    lock_for: Duration,
    state: std::sync::Mutex<HashMap<String, (u32, Option<Instant>)>>,
}

impl Lockout {
    pub fn new(max_fails: u32, lock_for: Duration) -> Self {
        Self {
            max_fails,
            lock_for,
            state: std::sync::Mutex::new(HashMap::new()),
        }
    }

    pub fn appendix_c() -> Self {
        Self::new(5, Duration::from_secs(15 * 60))
    }

    /// None = allowed; Some(remaining secs) = locked.
    pub fn check(&self, key: &str) -> Option<u64> {
        let st = self.state.lock().unwrap();
        match st.get(key) {
            Some((_, Some(until))) if *until > Instant::now() => {
                Some(until.duration_since(Instant::now()).as_secs())
            }
            _ => None,
        }
    }

    pub fn fail(&self, key: &str) -> Option<u64> {
        let mut st = self.state.lock().unwrap();
        let e = st.entry(key.to_string()).or_insert((0, None));
        e.0 += 1;
        if e.0 >= self.max_fails {
            let until = Some(Instant::now() + self.lock_for);
            e.1 = until;
            *e = (0, until);
            Some(self.lock_for.as_secs())
        } else {
            None
        }
    }

    pub fn reset(&self, key: &str) {
        self.state.lock().unwrap().remove(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limiter_blocks_after_max() {
        let l = RateLimiter::new(3, Duration::from_secs(60));
        assert!(l.check("ip1"));
        assert!(l.check("ip1"));
        assert!(l.check("ip1"));
        assert!(!l.check("ip1"));
        assert!(l.check("ip2"));
    }

    #[test]
    fn lockout_after_max_fails() {
        let l = Lockout::appendix_c();
        assert_eq!(l.check("a"), None);
        assert_eq!(l.fail("a"), None);
        assert_eq!(l.fail("a"), None);
        assert_eq!(l.fail("a"), None);
        assert_eq!(l.fail("a"), None);
        assert!(l.fail("a").is_some()); // 5th → locked
        assert!(l.check("a").is_some());
        l.reset("a");
        assert_eq!(l.check("a"), None);
    }
}
