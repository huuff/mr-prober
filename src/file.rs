use std::{future::Future, str::FromStr};

use crate::SentinelStorage;

pub struct FileSentinelStorage {
    file: <RuntimeImpl as Runtime>::File,
}

impl<Sentinel> SentinelStorage<Sentinel> for FileSentinelStorage
where
    Sentinel: ToString + FromStr + Clone,
    <Sentinel as FromStr>::Err: std::fmt::Debug,
{
    async fn commit(&mut self, sentinel: Sentinel) {
        RuntimeImpl::write_str(&self.file, &sentinel.to_string()).await;
    }

    async fn current(&self) -> Option<Sentinel> {
        let current_sentinel_string = RuntimeImpl::read_string(&self.file).await;
        // ROYY silly, but I'd love an is_not_empty method
        (!current_sentinel_string.is_empty())
            .then(|| Sentinel::from_str(&current_sentinel_string).unwrap())
    }
}

// TODO: handle errs

impl FileSentinelStorage {
    async fn new(file_path: &str) -> Self {
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
                use tokio::io::AsyncReadExt;

                let mut output = String::new();
                file
                    .lock()
                    .await
                    .read_to_string(&mut output)
                    .await
                    .unwrap();
                output
            }

            async fn write_str(file: &Self::File, text: &str) {
                use tokio::io::AsyncWriteExt;

                file
                    .lock()
                    .await
                    .write_all(text.as_bytes())
                    .await
                    .unwrap();
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
