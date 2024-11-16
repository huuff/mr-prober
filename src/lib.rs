#[cfg(feature = "file")]
pub mod file;
pub mod mem;
pub mod proc;

use std::{future::Future, marker::PhantomData};

#[cfg(feature = "file")]
use file::{FileSentinelStorage, FileStorableSentinel};
use mem::MemorySentinelStorage;
use proc::Processor;
use thiserror::Error;

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

    pub async fn probe(&mut self) -> Result<(), ProbeError<Storage::Err, ProcErr>> {
        let sentinel = self.storage.current().await.map_err(ProbeError::Store)?;

        if let Some(next) = self
            .processor
            .next(sentinel)
            .await
            .map_err(ProbeError::Processor)?
        {
            self.storage.commit(next).await.map_err(ProbeError::Store)?;
        }

        Ok(())
    }

    pub async fn current(&self) -> Result<Option<Sentinel>, Storage::Err> {
        self.storage.current().await
    }
}

#[derive(Error, Debug)]
pub enum ProbeError<StorageErr, ProcessorError> {
    Store(StorageErr),
    Processor(ProcessorError),
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
    pub async fn from_file(
        path: &str,
        proc: Proc,
    ) -> Result<Self, <file::RuntimeImpl as file::Runtime>::Err> {
        Ok(Self::new(FileSentinelStorage::open(path).await?, proc))
    }
}

pub trait SentinelStorage<Sentinel> {
    type Err;

    fn current(&self) -> impl Future<Output = Result<Option<Sentinel>, Self::Err>>;
    fn commit(&mut self, sentinel: Sentinel) -> impl Future<Output = Result<(), Self::Err>>;
}
