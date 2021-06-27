use actix::prelude::*;
use std::thread;
mod actors;
mod gtk_utils;

fn run_actix() {
    thread::spawn(|| {
        let system = System::new();

        system.block_on(async {
            actors::glint_manager::GlintManager::from_registry();
            actors::i3_ipc::I3Ipc {}.start();
        });

        system.run().unwrap();
    });
}

fn main() {
    gtk::init().expect("Failed to initialize GTK.");
    gtk_utils::setup();

    run_actix();

    gtk::main();
}
