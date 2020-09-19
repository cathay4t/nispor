use crate::error::NisporError;
use crate::ifaces::get_ifaces;
use crate::ifaces::Iface;
use crate::route::get_routes;
use crate::route_rule::get_route_rules;
use crate::route::Route;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct NetState {
    pub ifaces: HashMap<String, Iface>,
    pub routes: Vec<Route>,
}

impl NetState {
    pub fn retrieve() -> Result<NetState, NisporError> {
        let ifaces = get_ifaces()?;
        let routes = get_routes(&ifaces)?;
        let route_rules = get_route_rules()?;
        Ok(NetState { ifaces, routes })
    }
}
