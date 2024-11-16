pub mod preconf;
pub mod proc;
pub mod runtime;
pub mod store;

use std::marker::PhantomData;

use proc::Processor;
use store::SentinelStore;
use thiserror::Error;

pub struct Prober<Store, Sentinel, Proc, ProcErr> {
    store: Store,
    processor: Proc,
    _sentinel: PhantomData<Sentinel>,
    _proc_err: PhantomData<ProcErr>,
}

impl<Store, Sentinel, Proc, ProcErr> Prober<Store, Sentinel, Proc, ProcErr>
where
    Sentinel: Clone,
    Store: SentinelStore<Sentinel>,
    Proc: Processor<Sentinel, ProcErr>,
{
    pub fn new(storage: Store, processor: Proc) -> Self {
        Self {
            store: storage,
            processor,
            _sentinel: PhantomData,
            _proc_err: PhantomData,
        }
    }

    pub async fn probe(&mut self) -> Result<Option<Sentinel>, ProbeError<Store::Err, ProcErr>> {
        let current_sentinel = self.store.current().await.map_err(ProbeError::Store)?;

        let next_sentinel = self
            .processor
            .next(current_sentinel)
            .await
            .map_err(ProbeError::Processor)?;

        if let Some(ref next_sentinel) = next_sentinel {
            self.store
                .commit(next_sentinel.clone())
                .await
                .map_err(ProbeError::Store)?;
        }

        Ok(next_sentinel)
    }

    pub async fn current(&self) -> Result<Option<Sentinel>, Store::Err> {
        self.store.current().await
    }
}

#[derive(Error, Debug)]
pub enum ProbeError<StorageErr, ProcessorError> {
    #[error("store error: {0}")]
    Store(StorageErr),
    #[error("processor error: {0}")]
    Processor(ProcessorError),
}
