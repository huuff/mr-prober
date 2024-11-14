#[cfg(feature = "file")]
pub mod file;
pub mod mem;
pub mod proc;

use std::{future::Future, marker::PhantomData};

#[cfg(feature = "file")]
use file::FileSentinelStorage;
use mem::MemorySentinelStorage;
use proc::Processor;

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

impl<Sentinel: Default, Proc> Prober<MemorySentinelStorage<Sentinel>, Proc, Sentinel> {
    pub fn in_memory(processor: Proc) -> Self {
        Self {
            storage: MemorySentinelStorage::default(),
            processor,
            _sentinel: PhantomData,
        }
    }
}

#[cfg(feature = "file")]
impl<Sentinel, Proc> Prober<FileSentinelStorage, Proc, Sentinel> {
    pub async fn from_file(path: &str, proc: Proc) -> Self {
        Self {
            storage: FileSentinelStorage::open(path).await,
            processor: proc,
            _sentinel: PhantomData,
        }
    }
}

pub trait SentinelStorage<Sentinel> {
    fn current(&self) -> impl Future<Output = Option<Sentinel>>;
    fn commit(&mut self, sentinel: Sentinel) -> impl Future<Output = ()>;
}
