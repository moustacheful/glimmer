use actix::prelude::*;
use actors::{glimmer_manager::AttachSenderMsg, i3_ipc::WindowManager};
use clap::Parser;
use std::thread;
mod actors;
mod gtk_utils;

#[derive(clap::Parser, Debug)]
struct Opts {
    #[arg(short, long, default_value = "./style.css")]
    styles: String,
    #[clap(long)]
    use_sway: bool,
}

fn main() {
    let opts = Opts::parse();

    // x11 backend uses xwayland package
    // it has better results than the wayland backend

    // The following cannot be called before gdk::init()
    // Most probably an oversight (https://github.com/gtk-rs/gtk3-rs/pull/791)
    // gdk::set_allowed_backends("x11");

    // Set env instead:
    std::env::set_var("GDK_BACKEND", "x11");

    gtk::init().expect("Failed to init GTK.");

    gtk_utils::setup(opts.styles);

    let sender = gtk_utils::handle_messages();
    let sway = opts.use_sway;
    thread::spawn(move || {
        let system = System::new();

        system.block_on(async {
            let manager = actors::glimmer_manager::GlimmerManager::from_registry();
            manager.do_send(AttachSenderMsg { sender });

            actors::i3_ipc::WmIPC {
                wm: if sway {
                    WindowManager::SWAY
                } else {
                    WindowManager::I3
                },
            }
            .start();
        });

        system.run().unwrap();
    });

    gtk::main();
}
