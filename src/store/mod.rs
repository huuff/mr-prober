#[cfg(feature = "file")]
pub mod file;
pub mod mem;

use std::future::Future;

pub trait SentinelStore<Sentinel> {
    type Err;

    fn current(&self) -> impl Future<Output = Result<Option<Sentinel>, Self::Err>>;
    fn commit(&mut self, sentinel: Sentinel) -> impl Future<Output = Result<(), Self::Err>>;
}
