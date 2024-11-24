pub mod into;
pub mod strategy;

use strategy::{AutoProberCfg, AutoProberStrategy};

use crate::{
    runtime::{Runtime as _, RuntimeImpl},
    ProbeResult, Prober,
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
                    ProbeResult::Success => match self.cfg.on_success {
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
                    ProbeResult::Empty => match self.cfg.on_empty {
                        AutoProberStrategy::Abort => {
                            tracing::info!(event = "probe-empty", "abort");
                            return;
                        }
                        AutoProberStrategy::DelaySecs(secs) => {
                            tracing::info!(event = "probe-empty", "retrying in {secs} seconds");
                            RuntimeImpl::sleep(secs.into()).await;
                        }
                        AutoProberStrategy::Continue => {}
                        AutoProberStrategy::Backoff(ref mut backoff) => {
                            match backoff.next_sleep() {
                                Some(delay) => {
                                    let delay_secs = delay.as_secs();
                                    tracing::error!(
                                        event = "probe-empty",
                                        "trying again in {delay_secs} seconds"
                                    );
                                    RuntimeImpl::sleep(delay_secs).await
                                }
                                None => {
                                    tracing::error!(
                                        event = "probe-empty",
                                        "retries exhausted, aborting"
                                    );
                                    // MAYBE an err would be better
                                    return;
                                }
                            }
                        }
                    },
                    ProbeResult::Error(err) => match self.cfg.on_error {
                        AutoProberStrategy::Abort => {
                            tracing::error!(event = "probe-error", err = ?err, "abort");
                            err.panic();
                        }
                        AutoProberStrategy::DelaySecs(secs) => {
                            tracing::error!(event = "probe-error", err = ?err, "retrying in {secs} seconds");
                            RuntimeImpl::sleep(secs.into()).await;
                        }
                        AutoProberStrategy::Continue => {
                            tracing::error!(event = "probe-error", err = ?err, "trying again");
                        }
                        AutoProberStrategy::Backoff(ref mut backoff) => {
                            match backoff.next_sleep() {
                                Some(delay) => {
                                    let delay_secs = delay.as_secs();
                                    tracing::error!(event = "probe-error", err = ?err, "trying again in {delay_secs} seconds");
                                    RuntimeImpl::sleep(delay_secs).await
                                }
                                None => {
                                    tracing::error!(event = "probe-error", err = ?err, "retries exhausted, aborting");
                                    // MAYBE an err would be better
                                    return;
                                }
                            }
                        }
                    },
                }
            }
        })
    }
}
