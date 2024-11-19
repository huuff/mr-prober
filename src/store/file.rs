use crate::DynErr;
use std::str::FromStr;

use crate::runtime::{Runtime, RuntimeImpl};
use crate::SentinelStore;

pub struct FileSentinelStore {
    file: <RuntimeImpl as Runtime>::File,
}

#[async_trait::async_trait]
impl<Sentinel: FileStorableSentinel> SentinelStore<Sentinel> for FileSentinelStore {
    async fn commit(&mut self, sentinel: Sentinel) -> Result<(), DynErr> {
        Ok(RuntimeImpl::write_str(&self.file, &sentinel.to_string()).await?)
    }

    async fn current(&self) -> Result<Option<Sentinel>, DynErr> {
        let current_sentinel_string = RuntimeImpl::read_string(&self.file).await?;
        // ROYY silly, but I'd love an is_not_empty method
        Ok((!current_sentinel_string.is_empty())
            .then(|| Sentinel::from_str(&current_sentinel_string))
            .transpose()?)
    }
}

impl FileSentinelStore {
    pub async fn open(file_path: &str) -> Result<Self, <RuntimeImpl as Runtime>::Err> {
        Ok(Self {
            file: RuntimeImpl::open_file(file_path).await?,
        })
    }
}

/// A sentinel that can be stored in a file.
///
/// This storage uses [`ToString`] to save and [`FromStr`] to retrieve, so a sentinel has to
/// implement both of these
pub trait FileStorableSentinel:
    ToString + FromStr<Err = <Self as FileStorableSentinel>::ParseErr> + Clone + Send + 'static
{
    // HACK a crazy hack from https://github.com/rust-lang/rust/issues/20671#issuecomment-1905186183
    // to make this work
    type ParseErr: std::error::Error + Send + Sync;
}

impl<T> FileStorableSentinel for T
where
    T: ToString + FromStr + Clone + Send + 'static,
    <T as FromStr>::Err: std::error::Error + Send + Sync,
{
    type ParseErr = <Self as FromStr>::Err;
}
