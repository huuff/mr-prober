use std::str::FromStr;

use thiserror::Error;

use crate::runtime::{Runtime, RuntimeImpl};
use crate::SentinelStore;

/// A sentinel that can be stored in a file.
///
/// This storage uses [`ToString`] to save and [`FromStr`] to retrieve, so a sentinel has to
/// implement both of these
pub trait FileStorableSentinel:
    ToString + FromStr<Err = <Self as FileStorableSentinel>::ParseErr> + Clone
{
    // HACK a crazy hack from https://github.com/rust-lang/rust/issues/20671#issuecomment-1905186183
    // to make this work
    type ParseErr: std::fmt::Debug + std::fmt::Display;
}

impl<T> FileStorableSentinel for T
where
    T: ToString + FromStr + Clone,
    <T as FromStr>::Err: std::fmt::Debug + std::fmt::Display,
{
    type ParseErr = <Self as FromStr>::Err;
}

pub struct FileSentinelStore {
    file: <RuntimeImpl as Runtime>::File,
}

impl<Sentinel: FileStorableSentinel> SentinelStore<Sentinel> for FileSentinelStore {
    type Err = FileStorageError<Sentinel>;

    async fn commit(&mut self, sentinel: Sentinel) -> Result<(), Self::Err> {
        RuntimeImpl::write_str(&self.file, &sentinel.to_string())
            .await
            .map_err(FileStorageError::Filesystem)
    }

    async fn current(&self) -> Result<Option<Sentinel>, Self::Err> {
        let current_sentinel_string = RuntimeImpl::read_string(&self.file)
            .await
            .map_err(FileStorageError::Filesystem)?;
        // ROYY silly, but I'd love an is_not_empty method
        (!current_sentinel_string.is_empty())
            .then(|| Sentinel::from_str(&current_sentinel_string))
            .transpose()
            .map_err(FileStorageError::Parse)
    }
}

#[derive(Error, Debug)]
pub enum FileStorageError<Sentinel: FileStorableSentinel> {
    #[error("parse error: {0}")]
    Parse(<Sentinel as FileStorableSentinel>::ParseErr),
    #[error("runtime filesystem error: {0}")]
    Filesystem(<RuntimeImpl as Runtime>::Err),
}

impl FileSentinelStore {
    pub async fn open(file_path: &str) -> Result<Self, <RuntimeImpl as Runtime>::Err> {
        Ok(Self {
            file: RuntimeImpl::open_file(file_path).await?,
        })
    }
}
