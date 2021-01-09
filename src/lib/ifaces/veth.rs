use crate::ifaces::iface_conf::remove_iface;
use crate::ifaces::IfaceConf;
use crate::Iface;
use crate::IfaceType;
use crate::NisporError;
use rtnetlink::Handle;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct VethInfo {
    // Interface name of peer.
    // Use interface index number when peer interface is in other namespace.
    pub peer: String,
}

pub(crate) fn veth_iface_tidy_up(iface_states: &mut HashMap<String, Iface>) {
    let mut index_to_name = HashMap::new();
    for iface in iface_states.values() {
        index_to_name.insert(format!("{}", iface.index), iface.name.clone());
    }

    for iface in iface_states.values_mut() {
        if iface.iface_type != IfaceType::Veth {
            continue;
        }

        if let Some(VethInfo { peer }) = &iface.veth {
            if let Some(peer_iface_name) = index_to_name.get(peer) {
                iface.veth = Some(VethInfo {
                    peer: peer_iface_name.clone(),
                })
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct VethConf {
    pub peer: String,
}

impl VethConf {
    pub(crate) async fn apply(
        &self,
        handle: &rtnetlink::Handle,
        iface_conf: &IfaceConf,
        cur_iface: Option<&Iface>,
    ) -> Result<bool, NisporError> {
        if let Some(cur_iface) = cur_iface {
            if let Some(veth_info) = &cur_iface.veth {
                if &veth_info.peer != &self.peer {
                    remove_iface(handle, cur_iface).await?;
                    create_veth(handle, &iface_conf.name, &self.peer).await?;
                    Ok(true)
                } else {
                    // no change required.
                    Ok(false)
                }
            } else {
                Err(NisporError::bug(format!(
                    "Current veth interface {} has empty veth entry",
                    &cur_iface.name
                )))
            }
        } else {
            create_veth(handle, &iface_conf.name, &self.peer).await?;
            Ok(true)
        }
    }
}

async fn create_veth(
    handle: &Handle,
    name: &str,
    peer: &str,
) -> Result<(), NisporError> {
    match handle
        .link()
        .add()
        .veth(name.to_string(), peer.to_string())
        .execute()
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(NisporError::bug(format!(
            "Failed to create new veth pair '{}' '{}': {}",
            &name, &peer, e
        ))),
    }
}
