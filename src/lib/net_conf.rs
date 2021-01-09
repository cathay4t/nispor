use crate::error::NisporError;
use crate::ifaces::IfaceConf;
use serde_derive::{Deserialize, Serialize};
use tokio::runtime::Runtime;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct NetConf {
    pub ifaces: Option<Vec<IfaceConf>>,
}

impl NetConf {
    pub fn apply(&self) -> Result<bool, NisporError> {
        let mut changed = false;
        if let Some(iface_confs) = &self.ifaces {
            for iface_conf in iface_confs {
                changed |= Runtime::new()?.block_on(iface_conf.apply())?;
            }
        }
        Ok(changed)
    }
}
