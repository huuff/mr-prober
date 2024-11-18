use crate::{
    runtime::{Runtime as _, RuntimeImpl},
    Prober,
};

pub trait AutoProber<P: Prober> {
    fn spawn(self) -> tokio::task::JoinHandle<()>;
}

pub struct AutoProberImpl<P> {
    prober: P,
    on_empty: OnEmptyStrategy,
}

#[async_trait::async_trait]
impl<P> AutoProber<P> for AutoProberImpl<P>
where
    P: Prober + Send + Sync + 'static,
{
    fn spawn(mut self) -> tokio::task::JoinHandle<()> {
        RuntimeImpl::spawn(async move {
            loop {
                match self.prober.probe().await {
                    Ok(_) => {} // continue
                    Err(err) if err.is_empty() => match self.on_empty {
                        OnEmptyStrategy::Abort => {
                            tracing::info!("probe is empty, aborting");
                            return;
                        }
                        OnEmptyStrategy::DelaySecs(secs) => {
                            tracing::info!(
                                "probe is empty, waiting {secs} seconds before trying again"
                            );
                            RuntimeImpl::sleep(secs.into()).await;
                        }
                    },
                    err @ Err(_) => err.unwrap(),
                }
            }
        })
    }
}

/// What to do when the probe comes out empty
pub enum OnEmptyStrategy {
    /// Stop the auto-prober
    Abort,
    /// Wait this many seconds before trying again
    DelaySecs(u32),
}
