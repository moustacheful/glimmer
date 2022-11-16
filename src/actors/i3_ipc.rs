use crate::actors::glimmer_manager::{SwayWindowDataMsg, WindowDataMsgWrapper};

use actix::prelude::*;
use std::io;
use swayipc_async::{Connection, EventType, Fallible};
use tokio_i3ipc::{
    event::{Event, Subscribe},
    reply::Node,
    I3,
};
use tokio_stream::StreamExt;

use super::glimmer_manager::{GlimmerManager, I3WindowDataMsg};

pub enum WindowManager {
    I3,
    SWAY,
}

pub struct WmIPC {
    pub wm: WindowManager,
}

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
            let tree = i32.get_tree().await?;
            let found = tree
                .find_node(window_evt.container.id)
                .unwrap_or(window_evt.container);
            addr.do_send(WindowDataMsgWrapper {
                i3wdm: Some(I3WindowDataMsg {
                    change: window_evt.change,
                    container: found,
                }),
                swaywdm: None,
            });
        }
    }

    Ok(())
}

async fn sway_subscribe() -> Fallible<()> {
    let addr = GlimmerManager::from_registry();
    // let mut i = 0;
    let mut swaycon = Connection::new().await?;
    let mut event_stream = Connection::new()
        .await?
        .subscribe([EventType::Window])
        .await?;
    while let Some(event) = event_stream.next().await {
        // For some reason, on sway + weston (wayland), the windows created through
        // the xwayland compatibility package are focused, the window.set_can_focus(false)
        // is ignored...
        //
        // This debug helper avoids to be stuck in an inifinity sequence of Popups
        // that won't let the user terminate the process.
        /* DEBUG HELPER */
        // i += 1;
        // if i > 20 {
        //     panic!("end");
        // }
        /* END DEBUG HELPER */
        match event {
            Ok(swayipc::Event::Window(w)) => {
                let tree = swaycon.get_tree().await?;
                let node = match tree.find(|n: &swayipc::Node| n.id == w.container.id) {
                    Some(node) => node,
                    None => {
                        println!("Couldn't find node...");
                        w.container
                    }
                };
                addr.do_send(WindowDataMsgWrapper {
                    i3wdm: None,
                    swaywdm: Some(SwayWindowDataMsg {
                        change: w.change,
                        container: node,
                    }),
                })
            }
            Err(e) => {
                return Err(e);
            }
            _ => {
                unreachable!()
            }
        }
    }
    Ok(())
}

impl Actor for WmIPC {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Context<Self>) {
        // TODO: handle abort, on actor stop?
        match &self.wm {
            WindowManager::I3 => {
                actix::spawn(async { i3_subscribe().await.unwrap() });
            }
            WindowManager::SWAY => {
                actix::spawn(async { sway_subscribe().await.unwrap() });
            }
        }
    }
}
