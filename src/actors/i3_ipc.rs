use actix::prelude::*;
use std::io;
use tokio_i3ipc::{
    event::{Event, Subscribe},
    reply::Node,
    I3,
};
use tokio_stream::StreamExt;

use super::glimmer_manager::{GlimmerManager, WindowDataMsg};
pub struct I3Ipc {}

trait Find<T> {
    fn find_node(self, id: usize) -> Option<T>;
}

impl Find<Node> for Node {
    fn find_node(self, id: usize) -> Option<Node> {
        if self.id == id {
            return Some(self);
        }

        if self.nodes.is_empty() {
            return None;
        }

        let mut result: Option<Node> = None;

        for n in self.nodes {
            match n.find_node(id) {
                Some(found) => {
                    result = Some(found);
                    break;
                }
                _ => {}
            }
        }

        return result;
    }
}

// impl Find for Node {}

async fn i3_subscribe() -> io::Result<()> {
    let addr = GlimmerManager::from_registry();

    let mut i3 = I3::connect().await?;
    // There must be a better way of doing this instead of using two clients?
    let mut i32 = I3::connect().await?;

    i3.subscribe([Subscribe::Window]).await?;

    let mut listener = i3.listen();
    while let Some(evt) = listener.next().await {
        match evt? {
            Event::Window(window_evt) => {
                let tree = i32.get_tree().await?;
                let found = tree
                    .find_node(window_evt.container.id)
                    .unwrap_or(window_evt.container);

                addr.do_send(WindowDataMsg {
                    change: window_evt.change,
                    container: found,
                })
            }
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
