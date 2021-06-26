use actix::prelude::*;
use std::io;
use tokio_i3ipc::{
    event::{Event, Subscribe, WindowChange, WindowData},
    I3,
};
use tokio_stream::StreamExt;

use super::{glint_manager::GlintManager, messages::NodeChangeMsg};
pub struct I3Ipc {}

async fn i3_subscribe() -> io::Result<()> {
    let addr = GlintManager::from_registry();

    let mut i3 = I3::connect().await?;
    i3.subscribe([Subscribe::Window]).await?;

    let mut listener = i3.listen();
    while let Some(event) = listener.next().await {
        match event? {
            Event::Window(wevt) => match *wevt {
                WindowData {
                    change: WindowChange::Focus,
                    container,
                } => addr.do_send(NodeChangeMsg { node: container }),
                _ => { /* NOOP: Was not a focus type event. */ }
            },
            _ => { /* NOOP */ }
        }
    }

    Ok(())
}

impl Actor for I3Ipc {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Context<Self>) {
        // TODO: handle abort, on actor stop?
        actix::spawn(async { i3_subscribe().await.unwrap() });
    }
}
