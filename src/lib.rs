#[cfg(feature = "file")]
pub mod file;
pub mod mem;
pub mod proc;

use std::{future::Future, marker::PhantomData};

#[cfg(feature = "file")]
use file::FileSentinelStorage;
use file::FileStorableSentinel;
use mem::MemorySentinelStorage;
use proc::Processor;

pub struct Prober<Storage, Sentinel, Proc, ProcErr> {
    storage: Storage,
    processor: Proc,
    _sentinel: PhantomData<Sentinel>,
    _proc_err: PhantomData<ProcErr>,
}

impl<Storage, Sentinel, Proc, ProcErr> Prober<Storage, Sentinel, Proc, ProcErr>
where
    Storage: SentinelStorage<Sentinel>,
    Proc: Processor<Sentinel, ProcErr>,
{
    pub fn new(storage: Storage, processor: Proc) -> Self {
        Self {
            storage,
            processor,
            _sentinel: PhantomData,
            _proc_err: PhantomData,
        }
    }

    pub async fn probe(&mut self) -> Result<(), ProcErr> {
        let sentinel = self.storage.current().await;

        if let Some(next) = self.processor.next(sentinel).await? {
            self.storage.commit(next).await;
        }

        Ok(())
    }

    pub async fn current(&self) -> Option<Sentinel> {
        self.storage.current().await
    }
}

impl<Sentinel, Proc, ProcErr> Prober<MemorySentinelStorage<Sentinel>, Sentinel, Proc, ProcErr>
where
    Sentinel: Default + Clone,
    Proc: Processor<Sentinel, ProcErr>,
{
    pub fn in_memory(processor: Proc) -> Self {
        Self::new(MemorySentinelStorage::default(), processor)
    }
}

#[cfg(feature = "file")]
impl<Sentinel, Proc, ProcErr> Prober<FileSentinelStorage, Sentinel, Proc, ProcErr>
where
    Sentinel: FileStorableSentinel,
    Proc: Processor<Sentinel, ProcErr>,
{
    pub async fn from_file(path: &str, proc: Proc) -> Self {
        Self::new(FileSentinelStorage::open(path).await, proc)
    }
}

pub trait SentinelStorage<Sentinel> {
    fn current(&self) -> impl Future<Output = Option<Sentinel>>;
    fn commit(&mut self, sentinel: Sentinel) -> impl Future<Output = ()>;
}
