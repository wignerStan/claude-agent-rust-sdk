//! Rate limiting for MCP tool calls.
//!
//! This module provides configurable rate limiting to prevent abuse
//! and ensure fair usage of MCP servers.

use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;

use governor::{
    clock::DefaultClock,
    middleware::NoOpMiddleware,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter as GovernorRateLimiter,
};

/// Configuration for rate limiting.
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per second
    pub requests_per_second: u32,
    /// Burst capacity (max requests in a burst)
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_second: 10,
            burst_size: 20,
        }
    }
}

impl RateLimitConfig {
    /// Create a new rate limit configuration.
    pub fn new(requests_per_second: u32, burst_size: u32) -> Self {
        Self {
            requests_per_second,
            burst_size,
        }
    }

    /// Create a permissive rate limit (high throughput).
    pub fn permissive() -> Self {
        Self {
            requests_per_second: 100,
            burst_size: 200,
        }
    }

    /// Create a strict rate limit (low throughput).
    pub fn strict() -> Self {
        Self {
            requests_per_second: 5,
            burst_size: 10,
        }
    }
}

type InnerRateLimiter = GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock, NoOpMiddleware>;

/// Rate limiter for MCP tool calls.
#[derive(Clone)]
pub struct RateLimiter {
    limiter: Arc<InnerRateLimiter>,
    config: RateLimitConfig,
}

impl RateLimiter {
    /// Create a new rate limiter with the given configuration.
    pub fn new(config: RateLimitConfig) -> Self {
        let quota = Quota::per_second(
            NonZeroU32::new(config.requests_per_second).unwrap_or(NonZeroU32::new(1).unwrap()),
        )
        .allow_burst(NonZeroU32::new(config.burst_size).unwrap_or(NonZeroU32::new(1).unwrap()));

        let limiter = Arc::new(GovernorRateLimiter::direct(quota));

        Self { limiter, config }
    }

    /// Create a rate limiter with default settings.
    pub fn default_limiter() -> Self {
        Self::new(RateLimitConfig::default())
    }

    /// Check if a request is allowed (non-blocking).
    ///
    /// Returns `true` if the request is allowed, `false` if rate limited.
    pub fn check(&self) -> bool {
        self.limiter.check().is_ok()
    }

    /// Wait until a request is allowed (blocking).
    ///
    /// This method will block until the rate limiter allows the request.
    pub async fn wait(&self) {
        self.limiter.until_ready().await;
    }

    /// Wait with a timeout.
    ///
    /// Returns `true` if the request was allowed within the timeout,
    /// `false` if the timeout expired.
    pub async fn wait_with_timeout(&self, timeout: Duration) -> bool {
        tokio::time::timeout(timeout, self.limiter.until_ready())
            .await
            .is_ok()
    }

    /// Get the current configuration.
    pub fn config(&self) -> &RateLimitConfig {
        &self.config
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::default_limiter()
    }
}

impl std::fmt::Debug for RateLimiter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RateLimiter")
            .field("config", &self.config)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_config_default() {
        let config = RateLimitConfig::default();
        assert_eq!(config.requests_per_second, 10);
        assert_eq!(config.burst_size, 20);
    }

    #[test]
    fn test_rate_limiter_creation() {
        let limiter = RateLimiter::default();
        assert_eq!(limiter.config().requests_per_second, 10);
    }

    #[test]
    fn test_rate_limiter_check() {
        let limiter = RateLimiter::new(RateLimitConfig::new(100, 10));

        // First 10 requests should succeed (burst)
        for _ in 0..10 {
            assert!(limiter.check());
        }

        // 11th request should fail (rate limited)
        assert!(!limiter.check());
    }

    #[tokio::test]
    async fn test_rate_limiter_wait() {
        let limiter = RateLimiter::new(RateLimitConfig::new(100, 5));

        // Exhaust the burst
        for _ in 0..5 {
            assert!(limiter.check());
        }

        // Wait with timeout (should succeed quickly due to high rate)
        let result = limiter.wait_with_timeout(Duration::from_millis(100)).await;
        assert!(result);
    }
}
