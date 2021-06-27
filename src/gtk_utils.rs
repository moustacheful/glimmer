use cairo::{RectangleInt, Region};
use gtk::prelude::*;
use gtk::{Window, WindowType};

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

pub fn build_window() -> Window {
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

    return window;
}
