use std::convert::Infallible;

use crate::SentinelStore;

#[derive(Default)]
pub struct MemorySentinelStore<Sentinel> {
    sentinel: Option<Sentinel>,
}

impl<Sentinel: Clone> SentinelStore<Sentinel> for MemorySentinelStore<Sentinel> {
    type Err = Infallible;

    async fn current(&self) -> Result<Option<Sentinel>, Self::Err> {
        Ok(self.sentinel.clone())
    }

    async fn commit(&mut self, sentinel: Sentinel) -> Result<(), Self::Err> {
        self.sentinel.replace(sentinel);
        Ok(())
    }
}
