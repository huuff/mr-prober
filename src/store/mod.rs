#[cfg(feature = "file")]
mod blankets;
pub mod file;
pub mod mem;

use crate::alias::DynErr;

#[async_trait::async_trait]
pub trait SentinelStore<Sentinel> {
    async fn current(&self) -> Result<Option<Sentinel>, DynErr>;
    async fn commit(&mut self, sentinel: Sentinel) -> Result<(), DynErr>;
}
