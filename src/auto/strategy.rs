/// What the autoprober should do in specific situations
#[derive(Clone, Debug)]
pub enum AutoProberStrategy {
    /// Stop completely. Panics for errors.
    Abort,
    /// Wait this many seconds before proceeding.
    DelaySecs(u32),
    /// Continue instantly
    Continue,
}

pub struct AutoProberCfg {
    pub on_success: AutoProberStrategy,
    pub on_empty: AutoProberStrategy,
}

impl Default for AutoProberCfg {
    fn default() -> Self {
        Self {
            on_success: AutoProberStrategy::Continue,
            on_empty: AutoProberStrategy::Abort,
        }
    }
}
