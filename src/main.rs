use actix::prelude::*;
use actors::glimmer_manager::AttachSenderMsg;
use clap::Parser;
use std::thread;
mod actors;
mod gtk_utils;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The path of the css file to use
    #[arg(short, long)]
    styles: String,
}

fn main() {
    let opts = Args::parse();

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
