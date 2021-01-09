use crate::ifaces::get_ifaces_with_handle;
use crate::mac::set_mac_address;
use crate::{
    Iface, IfaceState, IfaceType, IpConf, IpFamily, NisporError, VethConf,
};
use rtnetlink::new_connection;
use rtnetlink::Handle;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct IfaceConf {
    pub name: String,
    #[serde(alias = "type")]
    pub iface_type: Option<IfaceType>,
    #[serde(default = "default_iface_state")]
    pub state: IfaceState,
    pub ipv4: Option<IpConf>,
    pub ipv6: Option<IpConf>,
    pub veth: Option<VethConf>,
    pub mac_address: Option<String>,
}

fn default_iface_state() -> IfaceState {
    IfaceState::Up
}

impl IfaceConf {
    pub(crate) fn is_absent(&self) -> bool {
        self.state == IfaceState::Absent
    }

    pub async fn apply(&self) -> Result<bool, NisporError> {
        let mut changed = false;

        let (connection, handle, _) = new_connection()?;
        tokio::spawn(connection);

        let cur_ifaces = get_ifaces_with_handle(&handle).await?;
        let cur_iface = cur_ifaces.get(&self.name);

        if self.is_absent() {
            match cur_iface {
                None => {
                    return Ok(false);
                }
                Some(i) => {
                    remove_iface(&handle, &i).await?;
                    return Ok(true);
                }
            }
        }

        let iface_type = match &self.iface_type {
            Some(i) => i,
            None => match cur_iface {
                Some(i) => &i.iface_type,
                None => {
                    return Err(NisporError::invalid_argument(format!(
                        "Interface {} does not exist \
                         and interface type is not define neither",
                        &self.name
                    )));
                }
            },
        };

        if let Some(ci) = cur_iface {
            if iface_type != &ci.iface_type {
                return Err(NisporError::invalid_argument(format!(
                    "Interface {} exist but with differnt type against \
                     desired interface type: {}",
                    &ci.iface_type, iface_type
                )));
            }
        }

        match iface_type {
            IfaceType::Veth => match &self.veth {
                Some(veth_conf) => {
                    changed |=
                        veth_conf.apply(&handle, &self, cur_iface).await?;
                }
                None => {
                    if cur_iface == None {
                        return Err(NisporError::invalid_argument(format!(
                            "Veth interface {} does not exist and \
                             has no VethConf defined",
                            &self.name
                        )));
                    }
                }
            },
            _ => (),
        };

        let mut cur_iface = get_iface(&handle, &self.name, &iface_type).await?;

        if let Some(mac_address) = &self.mac_address {
            if set_mac_address(&handle, &cur_iface, mac_address).await? {
                cur_iface = get_iface(&handle, &self.name, &iface_type).await?;
            }
        }

        changed |= apply_iface_state(&handle, &cur_iface, &self.state).await?;

        // TODO, indicate whether IP is changed or not
        if let Some(ipv6_conf) = &self.ipv6 {
            ipv6_conf.apply(&handle, &cur_iface, IpFamily::Ipv6).await?;
        } else {
            IpConf {
                addresses: Vec::new(),
            }
            .apply(&handle, &cur_iface, IpFamily::Ipv6)
            .await?;
        }
        if let Some(ipv4_conf) = &self.ipv4 {
            ipv4_conf.apply(&handle, &cur_iface, IpFamily::Ipv4).await?;
        } else {
            IpConf {
                addresses: Vec::new(),
            }
            .apply(&handle, &cur_iface, IpFamily::Ipv4)
            .await?;
        }
        Ok(changed)
    }
}

pub(crate) async fn remove_iface(
    handle: &Handle,
    iface: &Iface,
) -> Result<(), NisporError> {
    match handle.link().del(iface.index).execute().await {
        Ok(_) => Ok(()),
        Err(e) => Err(NisporError::bug(format!(
            "Failed to delete interface {} index {}: {}",
            &iface.name, iface.index, e,
        ))),
    }
}

fn is_virtual_iface(iface_type: &IfaceType) -> bool {
    [
        IfaceType::Bond,
        IfaceType::Bridge,
        IfaceType::Dummy,
        IfaceType::Loopback,
        IfaceType::MacVlan,
        IfaceType::MacVtap,
        IfaceType::OpenvSwitch,
        IfaceType::Tun,
        IfaceType::Veth,
        IfaceType::Vlan,
        IfaceType::Vrf,
        IfaceType::Vxlan,
    ]
    .contains(iface_type)
}

async fn apply_iface_state(
    handle: &Handle,
    cur_iface: &Iface,
    state: &IfaceState,
) -> Result<bool, NisporError> {
    if &cur_iface.state == state {
        Ok(false)
    } else {
        match state {
            IfaceState::Up => {
                match handle.link().set(cur_iface.index).up().execute().await {
                    Ok(_) => Ok(true),
                    Err(e) => Err(NisporError::bug(format!(
                        "Failed to bring interface {} index {} up: {}",
                        &cur_iface.name, cur_iface.index, e,
                    ))),
                }
            }
            IfaceState::Down => {
                match handle.link().set(cur_iface.index).down().execute().await
                {
                    Ok(_) => Ok(true),
                    Err(e) => Err(NisporError::bug(format!(
                        "Failed to bring interface {} index {} down: {}",
                        &cur_iface.name, cur_iface.index, e,
                    ))),
                }
            }
            _ => {
                eprintln!(
                    "Unsupported interface state {} of interface {}",
                    state, &cur_iface.name
                );
                Ok(false)
            }
        }
    }
}

async fn get_iface(
    handle: &Handle,
    iface_name: &str,
    iface_type: &IfaceType,
) -> Result<Iface, NisporError> {
    let mut ifaces = get_ifaces_with_handle(&handle).await?;
    match ifaces.remove(iface_name) {
        Some(i) => Ok(i),
        None => {
            if is_virtual_iface(iface_type) {
                Err(NisporError::bug(format!(
                    "Interface {} not found after creation",
                    iface_name
                )))
            } else {
                Err(NisporError::invalid_argument(format!(
                    "Interface {} not found",
                    iface_name
                )))
            }
        }
    }
}
