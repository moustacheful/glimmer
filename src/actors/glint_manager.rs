use super::glint_instance::{CreateDecorationMsg, Geometry, GlintInstance};
use super::messages::NodeChangeMsg;
use actix::prelude::*;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Message)]
#[rtype(result = "()")]
pub struct SweepMessage {}

// Actor definition
#[derive(Default)]
pub struct GlintManager {
    instances: HashMap<usize, actix::WeakAddr<GlintInstance>>,
}

impl Supervised for GlintManager {}
impl SystemService for GlintManager {
    fn start_service(_wrk: &ArbiterHandle) -> Addr<Self> {
        Self {
            instances: HashMap::new(),
        }
        .start()
    }
}
impl Actor for GlintManager {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.notify_later(SweepMessage {}, Duration::new(5, 0));
    }
}

impl GlintManager {
    fn sweep(&mut self) {
        for (id, addr) in &self.instances.clone() {
            match addr.upgrade() {
                None => {
                    self.instances.remove(&id);
                }
                _ => { /* NOOP */ }
            }
        }

        println!("performed sweep, size {:?}", self.instances.keys().len());
    }

    pub fn create_instance(&mut self, node: tokio_i3ipc::reply::Node) {
        let id = node.id;
        let addr = GlintInstance::new(id).start();

        addr.do_send(CreateDecorationMsg {
            label: node.name.unwrap(),
            geometry: Geometry {
                x: node.rect.x as i32,
                y: node.rect.y as i32,
                width: node.rect.width as i32,
                height: node.rect.height as i32,
            },
        });

        self.instances.insert(id, addr.downgrade());
    }
}

impl Handler<NodeChangeMsg> for GlintManager {
    type Result = ();

    fn handle(&mut self, msg: NodeChangeMsg, _ctx: &mut Context<Self>) {
        self.create_instance(msg.node);
    }
}

impl Handler<SweepMessage> for GlintManager {
    type Result = ();

    fn handle(&mut self, _msg: SweepMessage, ctx: &mut Context<Self>) {
        self.sweep();

        ctx.notify_later(SweepMessage {}, Duration::new(5, 0));
    }
}
