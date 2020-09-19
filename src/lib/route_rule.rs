use netlink_packet_route::RuleMessage;
use crate::NisporError;

pub struct RouteRule {
}

pub(crate) fn get_routes() -> Result<Vec<RouteRule>, NisporError> {
    Ok(Runtime::new()?.block_on(_get_routes())?)
}

async fn _get_routes() -> Result<Vec<RouteRule>, NisporError> {
    let mut rules = Vec::new();
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    let mut links = handle.route_rule().get(IpVersion::V6).execute();
    while let Some(rt_msg) = links.try_next().await? {
        routes.push(get_route_rule(rt_msg)?);
    }
    Ok(rules)
}

fn get_route_rule(
    route_msg: RuleMessage,
) -> Result<RouteRule, NisporError> {
    let mut rule = RouteRule::default();
    Ok(rule)
}
