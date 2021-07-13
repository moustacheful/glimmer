use super::glimmer_manager::{GlimmerManager, WindowDataMsg};
use crate::gtk_utils;
use actix::prelude::*;
use std::io;
use tokio_i3ipc::{
    event::{Event, Subscribe},
    reply::Node,
    I3,
};
use tokio_stream::StreamExt;

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
            if let Some(found) = n.find_node(id) {
                result = Some(found);
                break;
            }
        }

        result
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
        if let Event::Window(window_evt) = evt? {
            let container = window_evt.container;
            let name = container.name.clone().unwrap_or_default();

            if name == gtk_utils::MAIN_WINDOW_TITLE {
                dbg!(name);
                continue;
            }

            let tree = i32.get_tree().await?;
            let found = tree.find_node(container.id).unwrap_or(container);

            addr.do_send(WindowDataMsg {
                change: window_evt.change,
                container: found,
            })
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
