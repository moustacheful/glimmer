use crate::gtk_utils;
use actix::prelude::*;
use gtk::prelude::*;
use gtk::Window;

#[derive(Debug)]
pub struct Geometry {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct CreateDecorationMsg {
    pub label: String,
    pub geometry: Geometry,
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct DismountMsg {}

// Actor definition
pub struct GlintInstance {
    pub id: usize,
    window: Option<Window>,
}

impl Actor for GlintInstance {
    type Context = Context<Self>;

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        match &self.window {
            Some(window) => {
                window.close();
            }
            None => { /* NOOP */ }
        }
    }
}

impl GlintInstance {
    pub fn new(id: usize) -> Self {
        Self { id, window: None }
    }
}

impl Handler<CreateDecorationMsg> for GlintInstance {
    type Result = ();

    fn handle(&mut self, msg: CreateDecorationMsg, ctx: &mut Context<Self>) {
        let window = gtk_utils::build_window();
        let geometry = msg.geometry;

        window.resize(geometry.width, geometry.height);
        window.move_(geometry.x, geometry.y);
        window.show_all();

        self.window = Some(window);
        ctx.notify_later(DismountMsg {}, std::time::Duration::new(2, 0));
    }
}

impl Handler<DismountMsg> for GlintInstance {
    type Result = ();

    fn handle(&mut self, _msg: DismountMsg, ctx: &mut Self::Context) -> Self::Result {
        ctx.stop();
    }
}
