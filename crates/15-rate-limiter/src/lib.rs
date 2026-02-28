// ============================================================
//  YOUR CHALLENGE — implement a token bucket rate limiter.
//
//  The token bucket algorithm:
//    - A bucket holds up to `capacity` tokens
//    - Tokens refill at `rate` tokens per second
//    - Each request costs 1 token (or N tokens for burst requests)
//    - If there aren't enough tokens: reject the request
//
//  This is the algorithm behind:
//    - AWS API Gateway throttling
//    - Nginx `limit_req` module
//    - Stripe's and GitHub's rate limiting
//
//  Also implement a SlidingWindowLimiter that tracks a fixed
//  number of requests in a rolling time window.
//
//  Hint: use std::time::Instant for timestamps.
//        Do NOT use threads or async — simulate time in tests.
// ============================================================

use std::time::{Duration, Instant};
use std::collections::VecDeque;

// ── Token Bucket ─────────────────────────────────────────────

pub struct TokenBucket {
    capacity: f64,
    tokens: f64,
    rate: f64,          // tokens per second
    last_refill: Instant,
}

impl TokenBucket {
    /// Create a new bucket. Starts full.
    pub fn new(capacity: f64, rate_per_second: f64) -> Self {
        Self { capacity, tokens: capacity, rate: rate_per_second, last_refill: Instant::now() }
    }

    /// Try to consume `cost` tokens. Returns true if allowed.
    pub fn try_acquire(&mut self, cost: f64) -> bool {
        self.refill();
        if self.tokens >= cost { self.tokens -= cost; true } else { false }
    }

    /// Current token count (after refilling based on elapsed time).
    pub fn available_tokens(&mut self) -> f64 {
        self.refill();
        self.tokens
    }

    fn refill(&mut self) {
        let elapsed = self.last_refill.elapsed().as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.rate).min(self.capacity);
        self.last_refill = Instant::now();
    }
}

// ── Sliding Window ───────────────────────────────────────────

pub struct SlidingWindowLimiter {
    max_requests: usize,
    window: Duration,
    pub timestamps: VecDeque<Instant>,
}

impl SlidingWindowLimiter {
    pub fn new(max_requests: usize, window: Duration) -> Self {
        Self { max_requests, window, timestamps: VecDeque::new() }
    }

    /// Record a request attempt. Returns true if within the limit.
    pub fn try_acquire(&mut self) -> bool {
        let now = Instant::now();
        let cutoff = now - self.window;
        self.timestamps.retain(|&t| t > cutoff);
        if self.timestamps.len() < self.max_requests {
            self.timestamps.push_back(now);
            true
        } else {
            false
        }
    }

    /// Number of requests in the current window.
    pub fn current_count(&self) -> usize {
        let cutoff = Instant::now() - self.window;
        self.timestamps.iter().filter(|&&t| t > cutoff).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod token_bucket {
        use super::*;

        #[test]
        fn new_bucket_starts_full() {
            let mut bucket = TokenBucket::new(10.0, 1.0);
            assert!((bucket.available_tokens() - 10.0).abs() < 0.01);
        }

        #[test]
        fn acquiring_within_capacity_succeeds() {
            let mut bucket = TokenBucket::new(10.0, 1.0);
            assert!(bucket.try_acquire(5.0));
            assert!(bucket.try_acquire(5.0));
        }

        #[test]
        fn acquiring_more_than_available_fails() {
            let mut bucket = TokenBucket::new(5.0, 1.0);
            assert!(bucket.try_acquire(5.0));   // empties bucket
            assert!(!bucket.try_acquire(1.0));  // no tokens left
        }

        #[test]
        fn tokens_refill_over_time() {
            let mut bucket = TokenBucket::new(10.0, 10.0); // 10 tokens/sec
            // Drain completely
            assert!(bucket.try_acquire(10.0));
            assert!(!bucket.try_acquire(1.0));

            // Manually advance the last_refill time to simulate 1 second passing
            bucket.last_refill -= Duration::from_secs(1);
            assert!(bucket.try_acquire(1.0), "should have refilled after 1s");
        }

        #[test]
        fn tokens_do_not_exceed_capacity() {
            let mut bucket = TokenBucket::new(5.0, 100.0); // fast refill
            bucket.last_refill -= Duration::from_secs(100); // simulate long wait
            let available = bucket.available_tokens();
            assert!(available <= 5.0 + 0.001, "tokens capped at capacity: {available}");
        }
    }

    mod sliding_window {
        use super::*;

        #[test]
        fn requests_within_limit_are_allowed() {
            let mut limiter = SlidingWindowLimiter::new(5, Duration::from_secs(1));
            for _ in 0..5 {
                assert!(limiter.try_acquire());
            }
        }

        #[test]
        fn requests_exceeding_limit_are_denied() {
            let mut limiter = SlidingWindowLimiter::new(3, Duration::from_secs(1));
            for _ in 0..3 { limiter.try_acquire(); }
            assert!(!limiter.try_acquire());
        }

        #[test]
        fn old_requests_expire_from_window() {
            let mut limiter = SlidingWindowLimiter::new(3, Duration::from_secs(1));
            // Fill the window
            for _ in 0..3 { limiter.try_acquire(); }
            assert!(!limiter.try_acquire());

            // Expire all timestamps by backdating them
            let old_time = Instant::now() - Duration::from_secs(2);
            limiter.timestamps = std::collections::VecDeque::from(vec![old_time; 3]);
            assert!(limiter.try_acquire(), "window should have reset");
        }
    }
}
