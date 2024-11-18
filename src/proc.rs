use std::{convert::Infallible, future::Future};

#[async_trait::async_trait]
pub trait Processor<Sentinel, Err = Infallible> {
    async fn next(&self, current: Option<Sentinel>) -> Result<Option<Sentinel>, Err>;
}

#[async_trait::async_trait]
impl<Sentinel, Err, F, Fut> Processor<Sentinel, Err> for F
where
    Sentinel: Send + 'static,
    F: Fn(Option<Sentinel>) -> Fut + Send + Sync,
    Fut: Future<Output = Result<Option<Sentinel>, Err>> + Send,
{
    async fn next(&self, current: Option<Sentinel>) -> Result<Option<Sentinel>, Err> {
        (self)(current).await
    }
}
