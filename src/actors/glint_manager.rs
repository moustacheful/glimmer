use super::glint_instance::{GlintInstance, InstanceKillNotificationMsg};
use actix::{prelude::*, WeakAddr};
use glib::Sender;
use std::collections::HashMap;
use tokio_i3ipc::event::WindowChange;
use tokio_i3ipc::reply::Node;

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct WindowDataMsg {
    pub change: WindowChange,
    pub container: Node,
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct AttachSenderMsg {
    pub sender: Sender<crate::gtk_utils::Messages>,
}

// Actor definition
#[derive(Default)]
pub struct GlintManager {
    instances: HashMap<usize, actix::WeakAddr<GlintInstance>>,
    sender: Option<Sender<crate::gtk_utils::Messages>>,
}

impl Supervised for GlintManager {}
impl SystemService for GlintManager {
    fn start_service(_wrk: &ArbiterHandle) -> Addr<Self> {
        Self {
            instances: HashMap::new(),
            sender: None,
        }
        .start()
    }
}
impl Actor for GlintManager {
    type Context = Context<Self>;
}

impl GlintManager {
    pub fn get_instance(
        &mut self,
        msg: &WindowDataMsg,
        self_address: Addr<Self>,
    ) -> &WeakAddr<GlintInstance> {
        let id = msg.container.id;
        let sender = self.sender.clone().unwrap();
        let instance = self.instances.entry(id).or_insert_with(|| {
            GlintInstance::new(id, sender, self_address)
                .start()
                .downgrade()
        });

        return instance;
    }
}

impl Handler<WindowDataMsg> for GlintManager {
    type Result = ();

    fn handle(&mut self, msg: WindowDataMsg, ctx: &mut Context<Self>) {
        let instance = self.get_instance(&msg, ctx.address());

        match instance.upgrade() {
            Some(addr) => {
                addr.do_send(msg);
            }
            None => { /*noop*/ }
        }
    }
}

impl Handler<InstanceKillNotificationMsg> for GlintManager {
    type Result = ();

    fn handle(
        &mut self,
        msg: InstanceKillNotificationMsg,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.instances.remove(&msg.id);
    }
}

impl Handler<AttachSenderMsg> for GlintManager {
    type Result = ();

    fn handle(&mut self, msg: AttachSenderMsg, _ctx: &mut Self::Context) -> Self::Result {
        self.sender = Some(msg.sender);
    }
}
