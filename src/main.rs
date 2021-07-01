use actix::prelude::*;
use actors::glint_manager::AttachSenderMsg;
use std::thread;
mod actors;
mod gtk_utils;

fn main() {
    gtk::init().expect("Failed to initialize GTK.");
    gtk_utils::setup();

    let sender = gtk_utils::handle_messages();

    thread::spawn(move || {
        let system = System::new();

        system.block_on(async {
            let manager = actors::glint_manager::GlintManager::from_registry();
            manager.do_send(AttachSenderMsg { sender });

            actors::i3_ipc::I3Ipc {}.start();
        });

        system.run().unwrap();
    });

    gtk::main();
}
