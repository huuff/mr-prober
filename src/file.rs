use std::{marker::PhantomData, str::FromStr};

use crate::{Prober, ProberRetriever};

cfg_if::cfg_if! {
    if #[cfg(feature = "runtime-tokio")] {
        type File = tokio::sync::Mutex<tokio::fs::File>;
    } else {
        compile_error!("need to select a runtime")
    }
}

pub struct FileBackedProber<Item, Sentinel, Retriever> {
    retriever: Retriever,
    file: File,
    _item: PhantomData<Item>,
    _sentinel: PhantomData<Sentinel>,
}

impl<Item, Sentinel, Retriever> Prober for FileBackedProber<Item, Sentinel, Retriever>
where
    Sentinel: ToString + FromStr,
    <Sentinel as FromStr>::Err: std::fmt::Debug,
    Retriever: ProberRetriever<Item, Sentinel>,
{
    type Item = Item;
    type Sentinel = Sentinel;

    async fn next(&self) -> Option<Self::Item> {
        let last_sentinel_string = self.read_string().await;
        // ROYY silly, but I'd love an is_not_empty method
        let last_sentinel = (!last_sentinel_string.is_empty())
            .then(|| Self::Sentinel::from_str(&last_sentinel_string).unwrap());
        self.retriever.next(last_sentinel.as_ref()).await
    }

    async fn commit(&mut self, sentinel: Self::Sentinel) {
        self.write_string(&sentinel.to_string()).await;
    }
}

// TODO: how can I clean this up so there aren't so many cfg_ifs?
// TODO: handle errs

impl<Item, Sentinel, Retriever> FileBackedProber<Item, Sentinel, Retriever> {
    async fn new(file_path: &str, retriever: Retriever) -> Self {
        Self {
            retriever,
            file: open_file(file_path).await,
            _item: PhantomData,
            _sentinel: PhantomData,
        }
    }

    cfg_if::cfg_if! {
        if #[cfg(feature = "runtime-tokio")] {
            async fn read_string(&self) -> String {
                use tokio::io::AsyncReadExt;

                let mut output = String::new();
                self.file
                    .lock()
                    .await
                    .read_to_string(&mut output)
                    .await
                    .unwrap();
                output

            }
        } else {
            async fn read_string(&self) -> String {
                compile_error!("need to select a runtime")
            }
        }
    }

    cfg_if::cfg_if! {
        if #[cfg(feature = "runtime-tokio")] {
            async fn write_string(&self, sentinel: &str) {
                use tokio::io::AsyncWriteExt;

                self.file
                    .lock()
                    .await
                    .write_all(sentinel.as_bytes())
                    .await
                    .unwrap();

            }
        } else {
            async fn write_string(&self, sentinel: &str) {
                compile_error!("need to select a runtime")
            }
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "runtime-tokio")] {
        async fn open_file(path: &str) -> File {
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

    } else {
        async fn open_file(path: &str) -> File {
            compile_error!("need to select a runtime")
        }
    }
}
