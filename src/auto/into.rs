use crate::Prober;

use super::{strategy::AutoProberCfg, AutoProberImpl};

pub trait IntoAutoProber: Sized {
    fn into_auto(self, cfg: AutoProberCfg) -> AutoProberImpl<Self>;
}

impl<P: Prober> IntoAutoProber for P {
    fn into_auto(self, cfg: AutoProberCfg) -> AutoProberImpl<Self> {
        AutoProberImpl { prober: self, cfg }
    }
}
