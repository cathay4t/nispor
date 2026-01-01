// SPDX-License-Identifier: Apache-2.0

use rtnetlink::{
    packet_route::link::{BridgeVlanInfoFlags, LinkMessage},
    LinkBridgeVlan, LinkMessageBuilder,
};

use crate::{BridgeConf, BridgePortConf, BridgeVlanEntry, Iface};

fn add_bridge_vlan_to_builder<'a, T>(
    mut builder: LinkMessageBuilder<LinkBridgeVlan>,
    vlans: T,
) -> LinkMessageBuilder<LinkBridgeVlan>
where
    T: Iterator<Item = &'a BridgeVlanEntry>,
{
    for vlan in vlans {
        let mut flag = BridgeVlanInfoFlags::empty();
        if vlan.is_pvid {
            flag |= BridgeVlanInfoFlags::Pvid;
        }
        if vlan.is_egress_untagged {
            flag |= BridgeVlanInfoFlags::Untagged;
        }
        if let Some(vid) = vlan.vid {
            builder = builder.vlan(vid, flag);
        } else if let Some((vid_start, vid_end)) = vlan.vid_range.as_ref() {
            builder = builder
                .vlan_range_start(*vid_start, flag)
                .vlan_range_end(*vid_end, flag);
        }
    }
    builder
}

impl BridgePortConf {
    pub(crate) fn gen_add_port_vlan_conf_link_msg(
        &self,
        cur_iface: &Iface,
    ) -> Option<LinkMessage> {
        if let Some(vlans) = self.vlans.as_ref() {
            if vlans.iter().any(|v| !v.remove) {
                let builder = add_bridge_vlan_to_builder(
                    LinkBridgeVlan::new(cur_iface.index),
                    vlans.iter().filter(|vlan| !vlan.remove),
                );
                return Some(builder.build());
            }
        }
        None
    }

    pub(crate) fn gen_del_port_vlan_conf_link_msg(
        &self,
        cur_iface: &Iface,
    ) -> Option<LinkMessage> {
        if let Some(vlans) = self.vlans.as_ref() {
            if vlans.iter().any(|v| v.remove) {
                let builder = add_bridge_vlan_to_builder(
                    LinkBridgeVlan::new(cur_iface.index),
                    vlans.iter().filter(|vlan| vlan.remove),
                );
                return Some(builder.build());
            }
        }
        None
    }
}

impl BridgeConf {
    pub(crate) fn gen_add_vlan_conf_link_msg(
        &self,
        cur_iface: &Iface,
    ) -> Option<LinkMessage> {
        if let Some(vlans) = self.vlans.as_ref() {
            if vlans.iter().any(|v| !v.remove) {
                let builder = add_bridge_vlan_to_builder(
                    LinkBridgeVlan::new(cur_iface.index).bridge_self(),
                    vlans.iter().filter(|vlan| !vlan.remove),
                );
                return Some(builder.build());
            }
        }
        None
    }

    pub(crate) fn gen_del_vlan_conf_link_msg(
        &self,
        cur_iface: &Iface,
    ) -> Option<LinkMessage> {
        if let Some(vlans) = self.vlans.as_ref() {
            if vlans.iter().any(|v| v.remove) {
                let builder = add_bridge_vlan_to_builder(
                    LinkBridgeVlan::new(cur_iface.index).bridge_self(),
                    vlans.iter().filter(|vlan| vlan.remove),
                );
                return Some(builder.build());
            }
        }
        None
    }
}
