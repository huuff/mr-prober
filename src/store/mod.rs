use crate::alias::DynErr;

#[cfg(feature = "file")]
pub mod file;
pub mod mem;

#[async_trait::async_trait]
pub trait SentinelStore<Sentinel> {
    async fn current(&self) -> Result<Option<Sentinel>, DynErr>;
    async fn commit(&mut self, sentinel: Sentinel) -> Result<(), DynErr>;
}
