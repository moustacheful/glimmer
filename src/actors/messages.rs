use actix::prelude::*;
use tokio_i3ipc::reply::Node;

#[derive(Message)]
#[rtype(result = "()")]
pub struct NodeChangeMsg {
    pub node: Node,
}
