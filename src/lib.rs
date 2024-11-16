pub mod preconf;
pub mod proc;
pub mod store;

use std::marker::PhantomData;

use proc::Processor;
use store::SentinelStorage;
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
