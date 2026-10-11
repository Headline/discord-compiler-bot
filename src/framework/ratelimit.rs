use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serenity::model::id::UserId;

/// Per-user limit of `limit` uses per `window`, with at least `delay` between uses
pub struct RateLimiter {
    delay: Duration,
    window: Duration,
    limit: u32,
    // never held across an await, so a std mutex is sufficient
    users: Mutex<HashMap<UserId, Usage>>,
}

struct Usage {
    window_start: Instant,
    uses: u32,
    last_use: Instant,
}

impl RateLimiter {
    pub fn new(delay: Duration, window: Duration, limit: u32) -> Self {
        RateLimiter {
            delay,
            window,
            limit,
            users: Mutex::new(HashMap::new()),
        }
    }

    /// Records a use for the user, returning false if they are rate limited
    pub fn take(&self, user: UserId) -> bool {
        self.take_at(user, Instant::now())
    }

    fn take_at(&self, user: UserId, now: Instant) -> bool {
        let mut users = self.users.lock().unwrap_or_else(|e| e.into_inner());

        // entries past both limits behave like new users, so drop them
        users.retain(|_, usage| {
            now.duration_since(usage.window_start) < self.window
                || now.duration_since(usage.last_use) < self.delay
        });

        let Some(usage) = users.get_mut(&user) else {
            users.insert(
                user,
                Usage {
                    window_start: now,
                    uses: 1,
                    last_use: now,
                },
            );
            return true;
        };

        if now.duration_since(usage.last_use) < self.delay {
            return false;
        }

        if now.duration_since(usage.window_start) >= self.window {
            usage.window_start = now;
            usage.uses = 0;
        }

        if usage.uses >= self.limit {
            return false;
        }

        usage.uses += 1;
        usage.last_use = now;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const USER: UserId = UserId::new(1);

    fn limiter() -> RateLimiter {
        RateLimiter::new(Duration::from_secs(3), Duration::from_secs(10), 3)
    }

    #[test]
    fn enforces_delay_between_uses() {
        let limiter = limiter();
        let start = Instant::now();
        assert!(limiter.take_at(USER, start));
        assert!(!limiter.take_at(USER, start + Duration::from_secs(2)));
        assert!(limiter.take_at(USER, start + Duration::from_secs(3)));
    }

    #[test]
    fn enforces_limit_per_window() {
        let limiter = limiter();
        let start = Instant::now();
        assert!(limiter.take_at(USER, start));
        assert!(limiter.take_at(USER, start + Duration::from_secs(3)));
        assert!(limiter.take_at(USER, start + Duration::from_secs(6)));
        assert!(!limiter.take_at(USER, start + Duration::from_secs(9)));
        assert!(limiter.take_at(USER, start + Duration::from_secs(10)));
    }

    #[test]
    fn tracks_users_independently() {
        let limiter = limiter();
        let start = Instant::now();
        assert!(limiter.take_at(USER, start));
        assert!(limiter.take_at(UserId::new(2), start));
    }

    #[test]
    fn forgets_idle_users() {
        let limiter = limiter();
        let start = Instant::now();
        assert!(limiter.take_at(USER, start));
        assert!(limiter.take_at(UserId::new(2), start + Duration::from_secs(10)));
        assert_eq!(limiter.users.lock().unwrap().len(), 1);
    }
}
