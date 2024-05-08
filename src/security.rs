use std::{collections::HashMap, net::{IpAddr, SocketAddr}, sync::Arc};

use chrono::{DateTime, Duration, Utc};
use tokio::sync::Mutex;
use warp::{filters::BoxedFilter, reply::Reply};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApiVisitor {
    pub ip: IpAddr,
    pub first_request_time: DateTime<Utc>,
    pub requests_amount: u32,
}

pub type VisitorsData = Arc<Mutex<HashMap<IpAddr, ApiVisitor>>>;

pub async fn ratelimit_filter(visitors: VisitorsData, addr: Option<SocketAddr>, allowed_requests: u32, duration: Duration) -> BoxedFilter<(impl Reply,)> {
    todo!("Create rateliter");
}