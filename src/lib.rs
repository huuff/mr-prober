mod alias;
pub mod auto;
pub mod preconf;
pub mod proc;
pub mod runtime;
pub mod store;

use std::marker::PhantomData;

use alias::DynErr;
use proc::Processor;
use store::SentinelStore;
use thiserror::Error;

pub struct Prober<Store, Sentinel, Proc> {
    store: Store,
    processor: Proc,
    _sentinel: PhantomData<Sentinel>,
}

impl<Store, Sentinel, Proc> Prober<Store, Sentinel, Proc> {
    pub fn new(storage: Store, processor: Proc) -> Self {
        Self {
            store: storage,
            processor,
            _sentinel: PhantomData,
        }
    }
}

impl<Store, Sentinel, Proc> Prober<Store, Sentinel, Proc>
where
    Store: SentinelStore<Sentinel> + Send,
    Proc: Processor<Sentinel = Sentinel> + Send,
    Sentinel: Send,
{
    pub async fn probe(&mut self) -> ProbeResult {
        let current_sentinel = match self.store.current().await {
            Ok(current_sentinel) => current_sentinel,
            Err(store_err) => return ProbeResult::Error(ProbeError::Store(store_err)),
        };

        match self.processor.next(current_sentinel).await {
            Ok(Some(next_sentinel)) => {
                if let Err(store_err) = self.store.commit(next_sentinel).await {
                    return ProbeResult::Error(ProbeError::Store(store_err));
                }

                ProbeResult::Success
            }
            Ok(None) => ProbeResult::Empty,
            Err(proc_err) => ProbeResult::Error(ProbeError::Processor(proc_err)),
        }
    }
}

/// What comes out of a probe attempts
pub enum ProbeResult {
    /// The probe returned something
    Success,
    /// The probe came out empty
    Empty,
    /// The probe returned an error
    Error(ProbeError),
}

impl ProbeResult {
    /// Does nothing if it's a [`ProbeResult::Success`] or a [`ProbeResult::Empty`], but panics
    /// if it's a [`ProbeResult::Error`]
    pub fn expect_ok(self) {
        if let Self::Error(probe_err) = self {
            probe_err.panic();
        }
    }
}

impl From<ProbeError> for ProbeResult {
    fn from(value: ProbeError) -> Self {
        Self::Error(value)
    }
}

#[derive(Error, Debug)]
pub enum ProbeError {
    #[error("store error: {0}")]
    Store(DynErr),
    #[error("processor error: {0}")]
    Processor(DynErr),
}

impl ProbeError {
    /// Panic with the error message
    pub fn panic(&self) {
        panic!("probe error: {self:?}");
    }
}
