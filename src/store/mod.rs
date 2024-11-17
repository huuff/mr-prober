#[cfg(feature = "file")]
pub mod file;
pub mod mem;

#[async_trait::async_trait]
pub trait SentinelStore<Sentinel> {
    type Err;

    async fn current(&self) -> Result<Option<Sentinel>, Self::Err>;
    async fn commit(&mut self, sentinel: Sentinel) -> Result<(), Self::Err>;
}
