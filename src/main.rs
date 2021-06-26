use actix::prelude::*;
mod actors;
mod event_loop_bridge;
mod gtk_utils;

fn main() {
    gtk::init().expect("Failed to initialize GTK.");
    gtk_utils::setup();

    event_loop_bridge::run_actix_inside_gtk_event_loop().unwrap();
    event_loop_bridge::block_on(async {
        actors::glint_manager::GlintManager::from_registry();
        actors::i3_ipc::I3Ipc {}.start();
    });

    gtk::main();
}
