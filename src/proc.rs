use std::{future::Future, marker::PhantomData};

use crate::alias::DynErr;

#[async_trait::async_trait]
pub trait Processor {
    type Sentinel;

    async fn next(&self, current: Option<Self::Sentinel>)
        -> Result<Option<Self::Sentinel>, DynErr>;
}

pub struct FnProcessor<F, Sentinel> {
    f: F,
    _sentinel: PhantomData<Sentinel>,
}

// TODO, FUTURE: I think async closures would be great to clean the bounds
impl<F, Fut, Sentinel> From<F> for FnProcessor<F, Sentinel>
where
    F: Fn(Option<Sentinel>) -> Fut,
    Fut: Future<Output = Result<Option<Sentinel>, DynErr>>,
{
    fn from(value: F) -> Self {
        FnProcessor {
            f: value,
            _sentinel: PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<F, Fut, Sentinel> Processor for FnProcessor<F, Sentinel>
where
    Sentinel: Send + Sync + 'static,
    F: Fn(Option<Sentinel>) -> Fut + Send + Sync,
    Fut: Future<Output = Result<Option<Sentinel>, DynErr>> + Send,
{
    type Sentinel = Sentinel;

    async fn next(
        &self,
        current: Option<Self::Sentinel>,
    ) -> Result<Option<Self::Sentinel>, DynErr> {
        (self.f)(current).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fn_processor_works() {
        let proc = FnProcessor::from(|_| async { Ok(Some(42)) });

        assert_eq!(proc.next(None).await.expect("should be ok"), Some(42));
    }
}
