use std::convert::Infallible;

use crate::SentinelStorage;

#[derive(Default)]
pub struct MemorySentinelStorage<Sentinel> {
    sentinel: Option<Sentinel>,
}

impl<Sentinel: Clone> SentinelStorage<Sentinel> for MemorySentinelStorage<Sentinel> {
    type Err = Infallible;

    async fn current(&self) -> Result<Option<Sentinel>, Self::Err> {
        Ok(self.sentinel.clone())
    }

    async fn commit(&mut self, sentinel: Sentinel) -> Result<(), Self::Err> {
        self.sentinel.replace(sentinel);
        Ok(())
    }
}
