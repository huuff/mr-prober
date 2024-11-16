use std::{future::Future, str::FromStr};

use thiserror::Error;

use crate::SentinelStorage;

/// A sentinel that can be stored in a file.
///
/// This storage uses [`ToString`] to save and [`FromStr`] to retrieve, so a sentinel has to
/// implement both of these
pub trait FileStorableSentinel:
    ToString + FromStr<Err = <Self as FileStorableSentinel>::ParseErr> + Clone
{
    // HACK a crazy hack from https://github.com/rust-lang/rust/issues/20671#issuecomment-1905186183
    // to make this work
    type ParseErr: std::fmt::Debug;
}

impl<T> FileStorableSentinel for T
where
    T: ToString + FromStr + Clone,
    <T as FromStr>::Err: std::fmt::Debug,
{
    type ParseErr = <Self as FromStr>::Err;
}

pub struct FileSentinelStorage {
    file: <RuntimeImpl as Runtime>::File,
}

impl<Sentinel: FileStorableSentinel> SentinelStorage<Sentinel> for FileSentinelStorage {
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
    Parse(<Sentinel as FileStorableSentinel>::ParseErr),
    Filesystem(<RuntimeImpl as Runtime>::Err),
}

impl FileSentinelStorage {
    pub async fn open(file_path: &str) -> Result<Self, <RuntimeImpl as Runtime>::Err> {
        Ok(Self {
            file: RuntimeImpl::open_file(file_path).await?,
        })
    }
}

pub trait Runtime {
    type File;
    type Err;

    fn open_file(path: &str) -> impl Future<Output = Result<Self::File, Self::Err>>;

    fn read_string(file: &Self::File) -> impl Future<Output = Result<String, Self::Err>>;

    fn write_str(file: &Self::File, text: &str) -> impl Future<Output = Result<(), Self::Err>>;
}

pub struct RuntimeImpl;
cfg_if::cfg_if! {
    if #[cfg(feature = "runtime-tokio")] {
        impl Runtime for RuntimeImpl {
            type File = tokio::sync::Mutex<tokio::fs::File>;
            type Err = tokio::io::Error;

            async fn open_file(path: &str) -> Result<Self::File, Self::Err> {
                Ok(
                    tokio::sync::Mutex::new(
                        tokio::fs::File::options()
                        .read(true)
                        .write(true)
                        .create(true)
                        .truncate(false)
                        .open(path)
                        .await?
                    )
                )
            }


            async fn read_string(file: &Self::File) -> Result<String, Self::Err> {
                use tokio::io::AsyncReadExt as _;
                use tokio::io::AsyncSeekExt as _;

                let mut output = String::new();
                let mut file = file.lock().await;
                file.rewind().await?;
                file.read_to_string(&mut output).await?;

                Ok(output)
            }

            async fn write_str(file: &Self::File, text: &str) -> Result<(), Self::Err> {
                use tokio::io::AsyncWriteExt as _;
                use tokio::io::AsyncSeekExt as _;

                let mut file = file.lock().await;

                file.rewind().await?;
                file.set_len(0).await?;
                file.write_all(text.as_bytes()).await?;

                Ok(())
            }
        }
    } else {
        compile_error!("you need to select a runtime")
    }
}
