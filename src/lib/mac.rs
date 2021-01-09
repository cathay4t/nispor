use crate::{Iface, NisporError};
use rtnetlink::Handle;

pub(crate) fn parse_as_mac(
    mac_len: usize,
    data: &[u8],
) -> Result<String, NisporError> {
    let mut rt = String::new();
    for i in 0..mac_len {
        rt.push_str(&format!(
            "{:02x}",
            *data
                .get(i)
                .ok_or(NisporError::bug("wrong index at mac parsing".into()))?
        ));
        if i != mac_len - 1 {
            rt.push_str(":");
        }
    }
    Ok(rt)
}

pub(crate) async fn set_mac_address(
    handle: &Handle,
    iface: &Iface,
    mac_address: &str,
) -> Result<bool, NisporError> {
    if iface.mac_address == mac_address {
        Ok(false)
    } else {
        let addr = mac_address_str_to_u8(mac_address)?;
        match handle.link().set(iface.index).address(addr).execute().await {
            Ok(_) => Ok(true),
            Err(e) => Err(NisporError::invalid_argument(format!(
                "Failed to set mac address of interface {} index {}: {}",
                &iface.name, iface.index, e,
            ))),
        }
    }
}

fn mac_address_str_to_u8(mac_addr: &str) -> Result<Vec<u8>, NisporError> {
    let mut data = Vec::new();
    let mac_addrs: Vec<&str> = mac_addr.split(":").collect();
    for hex_str in mac_addrs {
        data.push(match u8::from_str_radix(hex_str, 16) {
            Ok(d) => d,
            Err(e) => {
                return Err(NisporError::invalid_argument(format!(
                    "Invalid MAC address {}: {}",
                    mac_addr, e
                )));
            }
        })
    }
    Ok(data)
}
