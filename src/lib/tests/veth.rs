use nispor::{IfaceState, NetConf, NetState, VethInfo};
use pretty_assertions::assert_eq;
use serde_yaml;
use std::panic;

const IFACE_NAME: &str = "veth1";

const CREATE_VETH1: &str = r#"---
ifaces:
  - name: veth1
    type: veth
    mac_address: "00:23:45:67:89:1a"
    veth:
      peer: veth1.ep"#;

const DELETE_VETH1: &str = r#"---
ifaces:
  - name: veth1
    state: absent"#;

const DOWN_VETH1: &str = r#"---
ifaces:
  - name: veth1
    state: down"#;

const EXPECTED_IFACE_STATE: &str = r#"---
- name: veth1
  iface_type: veth
  state: up
  mtu: 1500
  flags:
    - broadcast
    - lower_up
    - multicast
    - running
    - up
  ipv6:
    addresses:
      - address: "fe80::223:45ff:fe67:891a"
        prefix_len: 64
        valid_lft: forever
        preferred_lft: forever
  mac_address: "00:23:45:67:89:1a"
  veth:
    peer: veth1.ep"#;

#[test]
fn test_get_veth_iface_yaml() {
    with_veth_iface(|| {
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];
        let iface_type = &iface.iface_type;
        assert_eq!(iface_type, &nispor::IfaceType::Veth);
        assert_eq!(
            serde_yaml::to_string(&vec![iface]).unwrap(),
            EXPECTED_IFACE_STATE
        );
    });
}

fn with_veth_iface<T>(test: T) -> ()
where
    T: FnOnce() -> () + panic::UnwindSafe,
{
    let net_conf: NetConf = serde_yaml::from_str(CREATE_VETH1).unwrap();
    assert!(net_conf.apply().unwrap()); // return ture for state changed
    let result = panic::catch_unwind(|| {
        test();
    });

    let net_conf: NetConf = serde_yaml::from_str(DELETE_VETH1).unwrap();
    assert!(net_conf.apply().unwrap()); // return ture for state changed
    assert!(result.is_ok())
}

const ADD_IP_CONF: &str = r#"---
ifaces:
  - name: veth1
    ipv4:
      addresses:
        - address: "192.0.2.1"
          prefix_len: 24
    ipv6:
      addresses:
        - address: "2001:db8:a::9"
          prefix_len: 64"#;

const EMPTY_IP_CONF: &str = r#"---
ifaces:
  - name: veth1
    ipv4:
      addresses: []
    ipv6:
      addresses: []"#;

const EXPECTED_IFACE_STATE_ADD_IP: &str = r#"---
- name: veth1
  iface_type: veth
  state: up
  mtu: 1500
  flags:
    - broadcast
    - lower_up
    - multicast
    - running
    - up
  ipv4:
    addresses:
      - address: 192.0.2.1
        prefix_len: 24
        valid_lft: forever
        preferred_lft: forever
  ipv6:
    addresses:
      - address: "2001:db8:a::9"
        prefix_len: 64
        valid_lft: forever
        preferred_lft: forever
      - address: "fe80::223:45ff:fe67:891a"
        prefix_len: 64
        valid_lft: forever
        preferred_lft: forever
  mac_address: "00:23:45:67:89:1a"
  veth:
    peer: veth1.ep"#;

#[test]
fn test_veth_add_and_remove_ip() {
    with_veth_iface(|| {
        let conf: NetConf = serde_yaml::from_str(ADD_IP_CONF).unwrap();
        conf.apply().unwrap();
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];
        let iface_type = &iface.iface_type;
        assert_eq!(iface_type, &nispor::IfaceType::Veth);
        assert_eq!(
            serde_yaml::to_string(&vec![iface]).unwrap(),
            EXPECTED_IFACE_STATE_ADD_IP
        );
        let conf: NetConf = serde_yaml::from_str(EMPTY_IP_CONF).unwrap();
        conf.apply().unwrap();
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];
        let iface_type = &iface.iface_type;
        assert_eq!(iface_type, &nispor::IfaceType::Veth);
        assert_eq!(
            serde_yaml::to_string(&vec![iface]).unwrap(),
            EXPECTED_IFACE_STATE
        );
    });
}

#[test]
fn test_veth_bring_iface_down() {
    with_veth_iface(|| {
        let conf: NetConf = serde_yaml::from_str(DOWN_VETH1).unwrap();
        assert!(conf.apply().unwrap()); // true for net state changed
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];
        assert_eq!(iface.state, IfaceState::Down);
    });
}

const CHANGE_PEER_VETH1: &str = r#"---
ifaces:
  - name: veth1
    type: veth
    mac_address: "00:23:45:67:89:1a"
    veth:
      peer: veth1.end"#;

#[test]
fn test_veth_change_peer() {
    with_veth_iface(|| {
        let conf: NetConf = serde_yaml::from_str(CHANGE_PEER_VETH1).unwrap();
        assert!(conf.apply().unwrap()); // true for net state changed
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];
        assert_eq!(iface.state, IfaceState::Up);
        assert_eq!(
            iface.veth,
            Some(VethInfo {
                peer: "veth1.end".into(),
            })
        );
    });
}

#[test]
fn test_veth_apply_identical_conf() {
    with_veth_iface(|| {
        let conf: NetConf = serde_yaml::from_str(CREATE_VETH1).unwrap();
        assert!(!conf.apply().unwrap()); // false for net state unchanged
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];
        let iface_type = &iface.iface_type;
        assert_eq!(iface_type, &nispor::IfaceType::Veth);
        assert_eq!(
            serde_yaml::to_string(&vec![iface]).unwrap(),
            EXPECTED_IFACE_STATE
        );
    });
}
