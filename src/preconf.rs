//! Preconfigured probers

use crate::{proc::Processor, runtime, store, Prober};

impl<Sentinel, Proc> Prober<store::mem::MemorySentinelStore<Sentinel>, Sentinel, Proc>
where
    Sentinel: store::mem::MemoryStorableSentinel,
    Proc: Processor<Sentinel = Sentinel>,
{
    /// Creates a new prober that holds its sentinel value in memory
    pub fn in_memory(processor: Proc) -> Self {
        Self::new(store::mem::MemorySentinelStore::default(), processor)
    }
}

#[cfg(feature = "file")]
impl<Sentinel, Proc> Prober<store::file::FileSentinelStore, Sentinel, Proc>
where
    Sentinel: store::file::FileStorableSentinel,
    Proc: Processor<Sentinel = Sentinel>,
{
    /// Creates a new prober that stores its sentinel value in a file
    pub async fn from_file(
        path: &str,
        proc: Proc,
    ) -> Result<Self, <runtime::RuntimeImpl as runtime::Runtime>::Err> {
        Ok(Self::new(
            store::file::FileSentinelStore::open(path).await?,
            proc,
        ))
    }
}
