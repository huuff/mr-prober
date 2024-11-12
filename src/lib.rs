#[cfg(feature = "file")]
pub mod file;
pub mod mem;

use std::{future::Future, marker::PhantomData};

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

pub struct Prober<Storage, Proc, Sentinel> {
    storage: Storage,
    processor: Proc,
    _sentinel: PhantomData<Sentinel>,
}

impl<Sentinel, Storage, Proc> Prober<Storage, Proc, Sentinel>
where
    Storage: SentinelStorage<Sentinel>,
    Proc: Processor<Sentinel>,
{
    pub fn new(storage: Storage, processor: Proc) -> Self {
        Self {
            storage,
            processor,
            _sentinel: PhantomData,
        }
    }

    pub async fn probe(&mut self) {
        let sentinel = self.storage.current().await;

        let next = self.processor.next(sentinel).await;

        self.storage.commit(next).await;
    }

    pub async fn current(&self) -> Option<Sentinel> {
        self.storage.current().await
    }
}

pub trait SentinelStorage<Sentinel> {
    fn current(&self) -> impl Future<Output = Option<Sentinel>>;
    fn commit(&mut self, sentinel: Sentinel) -> impl Future<Output = ()>;
}
