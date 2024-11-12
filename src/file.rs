use std::{future::Future, marker::PhantomData, str::FromStr};

use crate::{Prober, ProberRetriever};


pub struct FileBackedProber<Item, Sentinel, Retriever> {
    retriever: Retriever,
    file: <RuntimeImpl as Runtime>::File,
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
        let last_sentinel_string = RuntimeImpl::read_string(&self.file).await;
        // ROYY silly, but I'd love an is_not_empty method
        let last_sentinel = (!last_sentinel_string.is_empty())
            .then(|| Self::Sentinel::from_str(&last_sentinel_string).unwrap());
        self.retriever.next(last_sentinel.as_ref()).await
    }

    async fn commit(&mut self, sentinel: Self::Sentinel) {
        RuntimeImpl::write_str(&self.file, &sentinel.to_string()).await;
    }
}

// TODO: handle errs

impl<Item, Sentinel, Retriever> FileBackedProber<Item, Sentinel, Retriever> {
    async fn new(file_path: &str, retriever: Retriever) -> Self {
        Self {
            retriever,
            file: RuntimeImpl::open_file(file_path).await,
            _item: PhantomData,
            _sentinel: PhantomData,
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
