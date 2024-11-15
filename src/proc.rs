use std::future::Future;

pub trait Processor<Sentinel> {
    fn next(&self, current: Option<Sentinel>) -> impl Future<Output = Option<Sentinel>>;
}

impl<Sentinel, F, Fut> Processor<Sentinel> for F
where
    F: Fn(Option<Sentinel>) -> Fut,
    Fut: Future<Output = Option<Sentinel>>,
{
    async fn next(&self, current: Option<Sentinel>) -> Option<Sentinel> {
        (self)(current).await
    }
}
