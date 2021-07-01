use std::collections::HashMap;

use cairo::{RectangleInt, Region};
use glib::Sender;
use gtk::prelude::*;
use gtk::{Window, WindowType};

use crate::actors::glint_instance::Geometry;

const MAIN_WINDOW_TITLE: &str = "HIGHLIGHTER";

pub fn setup() {
    let css_provider = gtk::CssProvider::new();
    let data = std::fs::read("./style.css").expect("Could not read css file!");
    css_provider
        .load_from_data(&data)
        .expect("Could not load css into GTK");

    let screen = gdk::Screen::default().unwrap();

    let _visual = screen
        .rgba_visual()
        .expect("No RGBA supported -- this utility makes no sense without it");

    gtk::StyleContext::add_provider_for_screen(&screen, &css_provider, 1);
}

pub fn update_window_position(window: &Window, geometry: Geometry) {
    window.resize(geometry.width, geometry.height);
    window.move_(geometry.x, geometry.y);
}

pub fn build_window(id: usize, geometry: Geometry) -> Window {
    let window = Window::new(WindowType::Popup);
    window.set_title(MAIN_WINDOW_TITLE);
    window.set_default_size(1, 1);
    window.set_can_focus(false);
    window.set_resizable(false);
    window.set_app_paintable(true);
    window.set_keep_above(true);

    window.connect_draw(|w, _c| {
        let rect: RectangleInt = RectangleInt {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        };
        let region: Region = Region::create_rectangle(&rect);

        w.window()
            .unwrap()
            .input_shape_combine_region(&region, 0, 0);

        return gtk::Inhibit(false);
    });

    let b = gtk::BoxBuilder::new().name("box").build();
    b.style_context().add_class("animate");

    let screen = gdk::Screen::default().unwrap();
    let visual = screen
        .rgba_visual()
        .expect("No RGBA supported -- this utility makes no sense without it");

    window.add(&b);
    window.set_visual(Some(&visual));
    update_window_position(&window, geometry);
    window.show_all();

    return window;
}

#[derive(Debug)]
pub struct WindowShim {
    pub id: usize,
    pub label: Option<String>,
    pub geometry: Geometry,
}

#[derive(Debug)]
pub enum Messages {
    Create(WindowShim),
    Update(WindowShim),
    Destroy(usize),
    None,
}

pub fn handle_messages() -> Sender<Messages> {
    let ctx = glib::MainContext::default();
    let _guard = ctx.acquire();

    let (sender, receiver) = glib::MainContext::channel::<Messages>(glib::PRIORITY_DEFAULT);
    let mut windows: HashMap<usize, Window> = HashMap::new();

    receiver.attach(None, move |msg| {
        match msg {
            Messages::Create(w) => match windows.insert(w.id, build_window(w.id, w.geometry)) {
                Some(old_window) => old_window.close(),
                _ => {}
            },
            Messages::Update(w) => match windows.get(&w.id) {
                Some(window) => {
                    update_window_position(&window, w.geometry);
                }
                None => {}
            },
            Messages::Destroy(id) => match windows.remove(&id) {
                Some(w) => {
                    w.close();
                }
                None => {}
            },

            _ => {}
        }

        glib::Continue(true)
    });

    return sender;
}
