use super::glimmer_manager::{GlimmerManager, WindowDataMsgTrait, WindowDataMsgWrapper};
use crate::gtk_utils::Messages;
use actix::prelude::*;
use glib::Sender;
use tokio_i3ipc::event::WindowChange;
use tokio_i3ipc::reply::Rect;

#[derive(Debug)]
pub struct Geometry {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct DismountMsg {}

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct InstanceKillNotificationMsg {
    pub id: usize,
}

// Actor definition
pub struct GlimmerInstance {
    pub id: usize,
    sender: Sender<Messages>,
    manager: Addr<GlimmerManager>,
    kill_handle: Option<SpawnHandle>,
}

impl Actor for GlimmerInstance {
    type Context = Context<Self>;

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        // Let the manager know the instance died for cleanup
        self.manager
            .do_send(InstanceKillNotificationMsg { id: self.id });
        // Also destroy the window
        self.sender.send(Messages::Destroy(self.id)).unwrap();
    }
}

impl GlimmerInstance {
    pub fn new(id: usize, sender: Sender<Messages>, manager: Addr<GlimmerManager>) -> Self {
        Self {
            id,
            sender,
            kill_handle: None,
            manager,
        }
    }
}

impl From<Rect> for Geometry {
    fn from(rect: Rect) -> Geometry {
        Geometry {
            x: rect.x as i32,
            y: rect.y as i32,
            width: rect.width as i32,
            height: rect.height as i32,
        }
    }
}

impl Handler<WindowDataMsgWrapper> for GlimmerInstance {
    type Result = ();

    fn handle(&mut self, msg: WindowDataMsgWrapper, ctx: &mut Context<Self>) {
        let message: Messages = match msg.get_change() {
            WindowChange::Focus => Messages::Create(msg.get_window_shim()),
            WindowChange::Close => Messages::Destroy(msg.get_container_id()),
            WindowChange::Move => Messages::Update(msg.get_window_shim()),
            WindowChange::FullscreenMode => Messages::Update(msg.get_window_shim()),

            _m => Messages::None,
        };

        self.sender.send(message).unwrap();

        if let Some(handle) = self.kill_handle {
            ctx.cancel_future(handle);
        }

        self.kill_handle = Some(ctx.notify_later(DismountMsg {}, std::time::Duration::new(2, 0)));
    }
}

impl Handler<DismountMsg> for GlimmerInstance {
    type Result = ();

    fn handle(&mut self, _msg: DismountMsg, ctx: &mut Self::Context) -> Self::Result {
        ctx.stop();
    }
}
