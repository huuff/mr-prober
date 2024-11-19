pub mod into;
pub mod strategy;

use strategy::{AutoProberCfg, AutoProberStrategy};

use crate::{
    runtime::{Runtime as _, RuntimeImpl},
    Prober,
};

pub trait AutoProber<P: Prober> {
    fn spawn(self) -> tokio::task::JoinHandle<()>;
}

pub struct AutoProberImpl<P> {
    prober: P,
    cfg: AutoProberCfg,
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
                    Ok(_) => match self.cfg.on_success {
                        AutoProberStrategy::Abort => return,
                        AutoProberStrategy::DelaySecs(secs) => {
                            RuntimeImpl::sleep(secs.into()).await;
                        }
                        AutoProberStrategy::Continue => {}
                        AutoProberStrategy::Backoff(ref mut backoff) => {
                            match backoff.next_sleep() {
                                Some(delay) => RuntimeImpl::sleep(delay.as_secs()).await,
                                None => return, // maybe an err would be better
                            }
                        }
                    },
                    Err(err) if err.is_empty() => match self.cfg.on_empty {
                        AutoProberStrategy::Abort => {
                            tracing::info!("probe is empty, aborting");
                            return;
                        }
                        AutoProberStrategy::DelaySecs(secs) => {
                            tracing::info!(
                                "probe is empty, waiting {secs} seconds before trying again"
                            );
                            RuntimeImpl::sleep(secs.into()).await;
                        }
                        AutoProberStrategy::Continue => {}
                        AutoProberStrategy::Backoff(ref mut backoff) => {
                            // TODO: kinda duplicated
                            match backoff.next_sleep() {
                                Some(delay) => RuntimeImpl::sleep(delay.as_secs()).await,
                                None => return, // maybe an err would be better
                            }
                        }
                    },
                    err @ Err(_) => err.unwrap(),
                }
            }
        })
    }
}
