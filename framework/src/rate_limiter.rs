use governor::{clock, Quota, RateLimiter};
use governor::state::{InMemoryState, NotKeyed};
use governor::middleware::NoOpMiddleware;
use nonzero::nonzero;
use reqwest_middleware::{Middleware, Next};
use reqwest::{Request, Response};
use task_local_extensions::Extensions;

pub struct RateLimitingMiddleware {
    limiter: RateLimiter<NotKeyed, InMemoryState, clock::DefaultClock, NoOpMiddleware>
}

impl RateLimitingMiddleware {
    pub fn new() -> RateLimitingMiddleware {
        RateLimitingMiddleware { limiter: RateLimiter::direct(Quota::per_second(nonzero!(8u32))) }
    }
}

#[async_trait::async_trait]
impl Middleware for RateLimitingMiddleware {
    async fn handle(&self, req: Request, extensions: &mut Extensions, next: Next<'_>) -> reqwest_middleware::Result<Response> {
        self.limiter.until_ready().await;
        next.run(req, extensions).await
    }
}
