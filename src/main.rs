use actix::prelude::*;
use actors::glimmer_manager::AttachSenderMsg;
use clap::{AppSettings, Clap};
use std::thread;
mod actors;
mod gtk_utils;

#[derive(Clap)]
#[clap(version = "0.0.1")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(short, long, default_value = "./style.css")]
    styles: String,
}

fn main() {
    let opts = Opts::parse();

    gtk::init().expect("Failed to initialize GTK.");
    gtk_utils::setup(opts.styles);

    let sender = gtk_utils::handle_messages();

    thread::spawn(move || {
        let system = System::new();

        system.block_on(async {
            let manager = actors::glimmer_manager::GlimmerManager::from_registry();
            manager.do_send(AttachSenderMsg { sender });

            actors::i3_ipc::I3Ipc {}.start();
        });

        system.run().unwrap();
    });

    gtk::main();
}
