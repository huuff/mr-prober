pub mod into;
pub mod strategy;

use strategy::{AutoProberCfg, AutoProberStrategy};

#[mockall_double::double]
use crate::Prober;
use crate::{
    proc::Processor,
    runtime::{Runtime as _, RuntimeImpl},
    store::SentinelStore,
    ProbeResult,
};

pub struct AutoProber<Store, Sentinel, Proc> {
    prober: Prober<Store, Sentinel, Proc>,
    cfg: AutoProberCfg,
}

impl<Store, Sentinel, Proc> AutoProber<Store, Sentinel, Proc>
where
    Store: SentinelStore<Sentinel> + Send + 'static,
    Proc: Processor<Sentinel = Sentinel> + Send + 'static,
    Sentinel: Send + 'static,
{
    pub fn spawn(mut self) -> tokio::task::JoinHandle<()> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proc::MockProcessor;
    use crate::store::MockSentinelStore;
    use crate::MockProber;

    #[tokio::test]
    async fn actually_probes() {
        let mut prober = MockProber::<MockSentinelStore<_>, (), MockProcessor>::new();
        prober
            .expect_probe()
            .times(1..)
            .returning(|| ProbeResult::Success);

        let auto = AutoProber {
            prober,
            cfg: AutoProberCfg::default(),
        };

        auto.spawn().await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }
}
