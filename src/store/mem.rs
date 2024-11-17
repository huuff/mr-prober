use std::convert::Infallible;

use crate::SentinelStore;

#[async_trait::async_trait]
impl<Sentinel: MemoryStorableSentinel> SentinelStore<Sentinel> for MemorySentinelStore<Sentinel> {
    type Err = Infallible;

    async fn current(&self) -> Result<Option<Sentinel>, Self::Err> {
        Ok(self.sentinel.clone())
    }

    async fn commit(&mut self, sentinel: Sentinel) -> Result<(), Self::Err> {
        self.sentinel.replace(sentinel);
        Ok(())
    }
}

/// A sentinel that can be stored in a memory.
pub trait MemoryStorableSentinel: Clone + Send + Sync + 'static {}

impl<T> MemoryStorableSentinel for T where T: Clone + Send + Sync + 'static {}

pub struct MemorySentinelStore<Sentinel> {
    sentinel: Option<Sentinel>,
}

impl<T> Default for MemorySentinelStore<T> {
    fn default() -> Self {
        Self { sentinel: None }
    }
}
