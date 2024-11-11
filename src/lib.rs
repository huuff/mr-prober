#[cfg(feature = "file")]
pub mod file;
pub mod mem;

use std::future::Future;

pub trait Prober {
    type Item;
    type Sentinel;

    fn next(&self) -> impl Future<Output = Option<Self::Item>>;

    fn commit(&mut self, sentinel: Self::Sentinel) -> impl Future<Output = ()>;
}

pub trait ProberRetriever<Item, Sentinel> {
    fn next(&self, sentinel: Option<&Sentinel>) -> impl Future<Output = Option<Item>>;
}

impl<Item, Sentinel, F, Fut> ProberRetriever<Item, Sentinel> for F
where
    F: Fn(Option<&Sentinel>) -> Fut,
    Fut: Future<Output = Option<Item>>,
{
    fn next(&self, sentinel: Option<&Sentinel>) -> impl Future<Output = Option<Item>> {
        (self)(sentinel)
    }
}
