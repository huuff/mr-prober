//! Preconfigured probers

use crate::{proc::Processor, store, Prober};

impl<Sentinel, Proc, ProcErr>
    Prober<store::mem::MemorySentinelStorage<Sentinel>, Sentinel, Proc, ProcErr>
where
    Sentinel: Default + Clone,
    Proc: Processor<Sentinel, ProcErr>,
{
    /// Creates a new prober that holds its sentinel value in memory
    pub fn in_memory(processor: Proc) -> Self {
        Self::new(store::mem::MemorySentinelStorage::default(), processor)
    }
}

#[cfg(feature = "file")]
impl<Sentinel, Proc, ProcErr> Prober<store::file::FileSentinelStorage, Sentinel, Proc, ProcErr>
where
    Sentinel: store::file::FileStorableSentinel,
    Proc: Processor<Sentinel, ProcErr>,
{
    /// Creates a new prober that stores its sentinel value in a file
    pub async fn from_file(
        path: &str,
        proc: Proc,
    ) -> Result<Self, <store::file::RuntimeImpl as store::file::Runtime>::Err> {
        Ok(Self::new(
            store::file::FileSentinelStorage::open(path).await?,
            proc,
        ))
    }
}
