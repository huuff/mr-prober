use std::future::Future;

/// An abstraction over runtimes, so they can be swappable.
pub trait Runtime {
    type File;
    type Err;

    fn open_file(path: &str) -> impl Future<Output = Result<Self::File, Self::Err>>;

    fn read_string(file: &Self::File) -> impl Future<Output = Result<String, Self::Err>>;

    fn write_str(file: &Self::File, text: &str) -> impl Future<Output = Result<(), Self::Err>>;
}

/// A runtime implementation that is selected depending on feature flags
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
