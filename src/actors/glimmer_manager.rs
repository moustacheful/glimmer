use super::glimmer_instance::{Geometry, GlimmerInstance, InstanceKillNotificationMsg};
use crate::gtk_utils::WindowShim;
use actix::{prelude::*, WeakAddr};
use glib::Sender;
use std::collections::HashMap;
use tokio_i3ipc::event::WindowChange;
use tokio_i3ipc::reply::Node;

pub trait WindowDataMsgTrait {
    fn get_container_id(&self) -> usize;
    fn get_window_shim(&self) -> WindowShim;
    fn get_change(&self) -> WindowChange;
}

#[derive(Debug, Message, Clone)]
#[rtype(result = "()")]
pub struct I3WindowDataMsg {
    pub change: WindowChange,
    pub container: Node,
}

impl WindowDataMsgTrait for I3WindowDataMsg {
    fn get_container_id(&self) -> usize {
        return self.container.id;
    }
    fn get_change(&self) -> WindowChange {
        self.change
    }
    fn get_window_shim(&self) -> WindowShim {
        let rect = &self.container.rect;
        return WindowShim {
            id: self.get_container_id(),
            label: match &self.container.name {
                None => None,
                Some(s) => Some(s.clone().to_owned()),
            },
            geometry: Geometry {
                height: rect.height as i32,
                width: rect.width as i32,
                x: rect.x as i32,
                y: rect.y as i32,
            },
        };
    }
}

#[derive(Debug, Message, Clone)]
#[rtype(result = "()")]
pub struct SwayWindowDataMsg {
    pub change: swayipc::WindowChange,
    pub container: swayipc::Node,
}

impl WindowDataMsgTrait for SwayWindowDataMsg {
    fn get_container_id(&self) -> usize {
        return self.container.id as usize;
    }
    fn get_change(&self) -> WindowChange {
        match self.change {
            swayipc::WindowChange::Close => WindowChange::Close,
            swayipc::WindowChange::Floating => WindowChange::Floating,
            swayipc::WindowChange::Focus => WindowChange::Focus,
            swayipc::WindowChange::FullscreenMode => WindowChange::FullscreenMode,
            swayipc::WindowChange::Mark => WindowChange::Mark,
            swayipc::WindowChange::Move => WindowChange::Move,
            swayipc::WindowChange::New => WindowChange::New,
            swayipc::WindowChange::Title => WindowChange::Title,
            swayipc::WindowChange::Urgent => WindowChange::Urgent,
            _ => unreachable!("WindowChange type not recognized"),
        }
    }
    fn get_window_shim(&self) -> WindowShim {
        let rect = self.container.rect;
        return WindowShim {
            id: self.get_container_id(),
            label: match &self.container.name {
                None => None,
                Some(s) => Some(s.clone().to_owned()),
            },
            geometry: Geometry {
                height: rect.height,
                width: rect.width,
                x: rect.x,
                y: rect.y,
            },
        };
    }
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct WindowDataMsgWrapper {
    pub i3wdm: Option<I3WindowDataMsg>,
    pub swaywdm: Option<SwayWindowDataMsg>,
}

impl WindowDataMsgWrapper {
    // fn get_wdm(&self) -> Box<dyn WDM> {
    fn get_wdm(&self) -> &dyn WindowDataMsgTrait {
        if self.i3wdm.is_none() && self.swaywdm.is_none() {
            panic!("ERROR: WindowDataMsgWrapper failed. Couldn't find internal WIndowDataMsg.")
        }
        if let Some(wdm) = &self.i3wdm {
            return wdm;
        }
        if let Some(wdm) = &self.swaywdm {
            return wdm;
        }
        unreachable!()
    }
}

impl WindowDataMsgTrait for WindowDataMsgWrapper {
    fn get_container_id(&self) -> usize {
        return self.get_wdm().get_container_id();
    }
    fn get_window_shim(&self) -> WindowShim {
        return self.get_wdm().get_window_shim();
    }
    fn get_change(&self) -> WindowChange {
        return self.get_wdm().get_change();
    }
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct AttachSenderMsg {
    pub sender: Sender<crate::gtk_utils::Messages>,
}

// Actor definition
#[derive(Default)]
pub struct GlimmerManager {
    instances: HashMap<usize, actix::WeakAddr<GlimmerInstance>>,
    sender: Option<Sender<crate::gtk_utils::Messages>>,
}

impl Supervised for GlimmerManager {}
impl SystemService for GlimmerManager {
    fn start_service(_wrk: &ArbiterHandle) -> Addr<Self> {
        Self {
            instances: HashMap::new(),
            sender: None,
        }
        .start()
    }
}
impl Actor for GlimmerManager {
    type Context = Context<Self>;
}

impl GlimmerManager {
    pub fn get_instance(
        &mut self,
        msg: &WindowDataMsgWrapper,
        self_address: Addr<Self>,
    ) -> &WeakAddr<GlimmerInstance> {
        let id = msg.get_container_id();
        let sender = self.sender.clone().unwrap();
        let instance = self.instances.entry(id).or_insert_with(|| {
            GlimmerInstance::new(id, sender, self_address)
                .start()
                .downgrade()
        });

        instance
    }
}

impl Handler<WindowDataMsgWrapper> for GlimmerManager {
    type Result = ();

    fn handle(&mut self, msg: WindowDataMsgWrapper, ctx: &mut Context<Self>) {
        let instance = self.get_instance(&msg, ctx.address());

        if let Some(addr) = instance.upgrade() {
            addr.do_send(msg);
        }
    }
}

impl Handler<InstanceKillNotificationMsg> for GlimmerManager {
    type Result = ();

    fn handle(
        &mut self,
        msg: InstanceKillNotificationMsg,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.instances.remove(&msg.id);
    }
}

impl Handler<AttachSenderMsg> for GlimmerManager {
    type Result = ();

    fn handle(&mut self, msg: AttachSenderMsg, _ctx: &mut Self::Context) -> Self::Result {
        self.sender = Some(msg.sender);
    }
}
