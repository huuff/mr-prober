use std::future::Future;

pub trait Processor<Sentinel> {
    fn next(&self, current: Option<Sentinel>) -> impl Future<Output = Sentinel>;
}

impl<Sentinel, F, Fut> Processor<Sentinel> for F
where
    F: Fn(Option<Sentinel>) -> Fut,
    Fut: Future<Output = Sentinel>,
{
    fn next(&self, current: Option<Sentinel>) -> impl Future<Output = Sentinel> {
        (self)(current)
    }
}
