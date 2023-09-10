// SPDX-License-Identifier: Apache-2.0

use serde::{Serialize, Deserialize};

use crate::{IfaceType, BridgeConf, VlanConf};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
pub struct IfaceConf {
    pub name: String,
    #[serde(default = "default_iface_state_in_conf")]
    pub state: IfaceState,
    #[serde(rename = "type")]
    pub iface_type: Option<IfaceType>,
    pub controller: Option<String>,
    pub ipv4: Option<IpConf>,
    pub ipv6: Option<IpConf>,
    pub mac_address: Option<String>,
    pub bond: Option<BondConf>,
    pub veth: Option<VethConf>,
    pub bridge: Option<BridgeConf>,
    pub vlan: Option<VlanConf>,
}

impl IfaceConf {
    pub async fn apply(&self, cur_iface: &Iface) -> Result<(), NisporError> {
        log::warn!(
            "WARN: IfaceConf::apply() is deprecated, \
            please use NetConf::apply() instead"
        );
        let ifaces = vec![self];
        let mut cur_ifaces = HashMap::new();
        cur_ifaces.insert(self.name.to_string(), cur_iface.clone());
        change_ifaces(&ifaces, &cur_ifaces).await
    }
}
