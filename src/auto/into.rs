use crate::Prober;

use super::{strategy::AutoProberCfg, AutoProber};

impl<Store, Sentinel, Processor> Prober<Store, Sentinel, Processor> {
    pub fn into_auto(self, cfg: AutoProberCfg) -> AutoProber<Store, Sentinel, Processor> {
        AutoProber { prober: self, cfg }
    }
}
