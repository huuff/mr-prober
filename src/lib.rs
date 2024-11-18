mod alias;
pub mod preconf;
pub mod proc;
pub mod runtime;
pub mod store;

use std::marker::PhantomData;

use alias::DynErr;
use proc::Processor;
use store::SentinelStore;
use thiserror::Error;

// MAYBE we'd need a trait for this? it would be much easier to express for downstream crates than
// as an `impl Prober` instead of this lot of generic params
pub struct Prober<Store, Sentinel, Proc, ProcErr> {
    store: Store,
    processor: Proc,
    _sentinel: PhantomData<Sentinel>,
    _proc_err: PhantomData<ProcErr>,
}

impl<Store, Sentinel, Proc, ProcErr> Prober<Store, Sentinel, Proc, ProcErr>
where
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

    // MAYBE rather than returning the sentinel, which requires a clone, we could return some error variant
    // to tell the downstream user that there is no next sentinel, since sentinels should themselves be an
    // inner concern?
    pub async fn probe(&mut self) -> Result<(), ProbeError<ProcErr>> {
        let current_sentinel = self.store.current().await.map_err(ProbeError::Store)?;

        if let Some(next_sentinel) = self
            .processor
            .next(current_sentinel)
            .await
            .map_err(ProbeError::Processor)?
        {
            self.store
                .commit(next_sentinel)
                .await
                .map_err(ProbeError::Store)?;
        }

        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum ProbeError<ProcessorError> {
    #[error("store error: {0}")]
    Store(DynErr),
    #[error("processor error: {0}")]
    Processor(ProcessorError),
}
