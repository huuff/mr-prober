//! Blanket impls for stores

use std::{future::Future, pin::Pin};

use crate::alias::DynErr;

use super::SentinelStore;

// TODO can't I just async_trait::async_trait?
impl<T> SentinelStore<T> for Box<dyn SentinelStore<T> + Send + Sync + 'static> {
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    fn current<'life0, 'async_trait>(
        &'life0 self,
    ) -> Pin<Box<dyn Future<Output = Result<Option<T>, DynErr>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        (**self).current()
    }

    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    fn commit<'life0, 'async_trait>(
        &'life0 mut self,
        sentinel: T,
    ) -> Pin<Box<dyn Future<Output = Result<(), DynErr>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        (**self).commit(sentinel)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn box_dyn_send_sync_impls_trait() {
        assert!(
            impls::impls!(Box<dyn SentinelStore<String> + Send + Sync + 'static>: SentinelStore<String>)
        )
    }
}
