use governor::middleware::NoOpMiddleware;
use governor::state::{InMemoryState, NotKeyed};
use governor::{clock, Quota, RateLimiter};
use nonzero::nonzero;
use reqwest::{Request, Response};
use reqwest_middleware::{Middleware, Next};
use task_local_extensions::Extensions;

pub struct RateLimitingMiddleware {
    limiter: RateLimiter<NotKeyed, InMemoryState, clock::DefaultClock, NoOpMiddleware>,
}

impl RateLimitingMiddleware {
    pub fn new() -> RateLimitingMiddleware {
        RateLimitingMiddleware {
            limiter: RateLimiter::direct(Quota::per_second(nonzero!(8u32))),
        }
    }
}

impl Default for RateLimitingMiddleware {
    fn default() -> Self {
        RateLimitingMiddleware::new()
    }
}

#[async_trait::async_trait]
impl Middleware for RateLimitingMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> reqwest_middleware::Result<Response> {
        self.limiter.until_ready().await;
        next.run(req, extensions).await
    }
}
