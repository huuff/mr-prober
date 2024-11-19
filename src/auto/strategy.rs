use std::time::Duration;

/// What the autoprober should do in specific situations
#[derive(Clone, Debug)]
pub enum AutoProberStrategy {
    /// Stop completely. Panics for errors.
    Abort,
    /// Wait this many seconds before proceeding.
    DelaySecs(u32),
    /// Continue instantly
    Continue,
    /// Backoff
    Backoff(BackoffStrategy),
}

pub struct AutoProberCfg {
    pub on_success: AutoProberStrategy,
    pub on_empty: AutoProberStrategy,
    pub on_error: AutoProberStrategy,
}

impl Default for AutoProberCfg {
    fn default() -> Self {
        Self {
            on_success: AutoProberStrategy::Continue,
            on_empty: AutoProberStrategy::Abort,
            on_error: AutoProberStrategy::Abort,
        }
    }
}

#[derive(Clone, Debug)]
pub struct BackoffStrategy {
    backoff_template: exponential_backoff::Backoff,
    current_backoff: exponential_backoff::IntoIter,
}

impl BackoffStrategy {
    pub fn new(attempts: u32, delay_secs: u32) -> Self {
        let backoff = exponential_backoff::Backoff::new(
            attempts,
            Duration::from_secs(delay_secs.into()),
            None,
        );
        Self {
            current_backoff: backoff.clone().into_iter(),
            backoff_template: backoff,
        }
    }

    pub fn next_sleep(&mut self) -> Option<Duration> {
        // I flatten it because I don't like much its Option<Option<>> approach
        self.current_backoff.next().flatten()
    }

    pub fn reset(&mut self) {
        self.current_backoff = self.backoff_template.clone().into_iter()
    }
}
