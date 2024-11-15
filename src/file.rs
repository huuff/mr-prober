use std::{future::Future, str::FromStr};

use crate::SentinelStorage;

/// A sentinel that can be stored in a file.
///
/// This storage uses [`ToString`] to save and [`FromStr`] to retrieve, so a sentinel has to
/// implement both of these
pub trait FileStorableSentinel:
    ToString + FromStr<Err = <Self as FileStorableSentinel>::Err> + Clone
{
    // HACK a crazy hack from https://github.com/rust-lang/rust/issues/20671#issuecomment-1905186183
    // to make this work
    type Err: std::fmt::Debug;
}

impl<T> FileStorableSentinel for T
where
    T: ToString + FromStr + Clone,
    <T as FromStr>::Err: std::fmt::Debug,
{
    type Err = <Self as FromStr>::Err;
}

pub struct FileSentinelStorage {
    file: <RuntimeImpl as Runtime>::File,
}

impl<Sentinel: FileStorableSentinel> SentinelStorage<Sentinel> for FileSentinelStorage {
    async fn commit(&mut self, sentinel: Sentinel) {
        RuntimeImpl::write_str(&self.file, &sentinel.to_string()).await;
    }

    async fn current(&self) -> Option<Sentinel> {
        let current_sentinel_string = RuntimeImpl::read_string(&self.file).await;
        // ROYY silly, but I'd love an is_not_empty method
        // TODO: no unwrapping!
        (!current_sentinel_string.is_empty())
            .then(|| Sentinel::from_str(&current_sentinel_string).unwrap())
    }
}

impl FileSentinelStorage {
    pub async fn open(file_path: &str) -> Self {
        Self {
            file: RuntimeImpl::open_file(file_path).await,
        }
    }
}

trait Runtime {
    type File;

    fn open_file(path: &str) -> impl Future<Output = Self::File>;

    fn read_string(file: &Self::File) -> impl Future<Output = String>;

    fn write_str(file: &Self::File, text: &str) -> impl Future<Output = ()>;
}

struct RuntimeImpl;
cfg_if::cfg_if! {
    if #[cfg(feature = "runtime-tokio")] {
        impl Runtime for RuntimeImpl {
            type File = tokio::sync::Mutex<tokio::fs::File>;

            async fn open_file(path: &str) -> Self::File {
                tokio::sync::Mutex::new(
                    tokio::fs::File::options()
                        .read(true)
                        .write(true)
                        .create(true)
                        .truncate(false)
                        .open(path)
                        .await
                        .unwrap()
                )
            }


            async fn read_string(file: &Self::File) -> String {
                use tokio::io::AsyncReadExt as _;
                use tokio::io::AsyncSeekExt as _;

                let mut output = String::new();
                let mut file = file.lock().await;
                file.rewind().await.unwrap();
                file.read_to_string(&mut output).await.unwrap();

                output
            }

            async fn write_str(file: &Self::File, text: &str) {
                use tokio::io::AsyncWriteExt as _;
                use tokio::io::AsyncSeekExt as _;

                let mut file = file.lock().await;

                file.rewind().await.unwrap();
                file.set_len(0).await.unwrap();
                file.write_all(text.as_bytes()).await.unwrap();
            }
        }
    } else {
        impl Runtime for RuntimeImpl {
            type File = ();
            type FileWrapper<File> = ();

            async fn open_file(path: &str) -> Self::FileWrapper<File> {
                compile_error!("need to select a runtime feature")
            }


            async fn read_string(file_wrapper: &Self::FileWrapper<File>) -> String {
                compile_error!("need to select a runtime feature")

            }

            async fn write_str(file_wrapper: &Self::FileWrapper<File>, text: &str) {
                compile_error!("need to select a runtime feature")
            }
        }
    }
}
