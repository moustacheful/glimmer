use super::glimmer_manager::{GlimmerManager, WindowDataMsg};
use crate::gtk_utils::{Messages, WindowShim};
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

impl From<WindowDataMsg> for WindowShim {
    fn from(window_data: WindowDataMsg) -> WindowShim {
        let rect = window_data.container.rect;
        WindowShim {
            id: window_data.container.id,
            label: window_data.container.name,
            geometry: rect.into(),
        }
    }
}

impl From<WindowDataMsg> for usize {
    fn from(window_data: WindowDataMsg) -> usize {
        window_data.container.id
    }
}

impl Handler<WindowDataMsg> for GlimmerInstance {
    type Result = ();

    fn handle(&mut self, msg: WindowDataMsg, ctx: &mut Context<Self>) {
        let message: Messages = match msg.change {
            WindowChange::Focus => Messages::Create(msg.into()),
            WindowChange::Close => Messages::Destroy(msg.into()),
            WindowChange::Move => Messages::Update(msg.into()),
            WindowChange::FullscreenMode => Messages::Update(msg.into()),

            _m => Messages::None,
        };

        self.sender.send(message).unwrap();

        match self.kill_handle {
            Some(handle) => {
                ctx.cancel_future(handle);
            }
            None => {}
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
