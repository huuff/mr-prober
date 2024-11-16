use std::{convert::Infallible, future::Future};

pub trait Processor<Sentinel, Err = Infallible> {
    fn next(
        &self,
        current: Option<Sentinel>,
    ) -> impl Future<Output = Result<Option<Sentinel>, Err>>;
}

impl<Sentinel, Err, F, Fut> Processor<Sentinel, Err> for F
where
    F: Fn(Option<Sentinel>) -> Fut,
    Fut: Future<Output = Result<Option<Sentinel>, Err>>,
{
    async fn next(&self, current: Option<Sentinel>) -> Result<Option<Sentinel>, Err> {
        (self)(current).await
    }
}
