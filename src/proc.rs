use std::future::Future;

use crate::alias::DynErr;

#[async_trait::async_trait]
pub trait Processor<Sentinel> {
    async fn next(&self, current: Option<Sentinel>) -> Result<Option<Sentinel>, DynErr>;
}

#[async_trait::async_trait]
impl<Sentinel, F, Fut> Processor<Sentinel> for F
where
    Sentinel: Send + 'static,
    F: Fn(Option<Sentinel>) -> Fut + Send + Sync,
    Fut: Future<Output = Result<Option<Sentinel>, DynErr>> + Send,
{
    async fn next(&self, current: Option<Sentinel>) -> Result<Option<Sentinel>, DynErr> {
        (self)(current).await
    }
}
