# glimmer

## What

A tool for decorating i3 (_should_ work with Sway too!) windows when they get focused, written in Rust.

https://user-images.githubusercontent.com/4857535/124782646-61e90a80-df12-11eb-8930-a321ecffbee1.mp4

## Why

When using i3-gaps I ran into the following problems.

- Glitches when using regular borders and titlebars that showed up in the background (as described [here](https://github.com/Airblader/i3/issues/190)), which as far as I know haven't been solved yet.
- The above meant usually relying on transparency or dimming to have a 'highlighted' state, which for me at least beats the purpose of having all your windows in a tiled fashion. To get the highlighted effect you have to lower the opacity or dim it to the point it's too hard to read.
- Even if borders and titlebars _did_ work properly. They just don't gel very well with the `a e s t h e t i c` anyway.

Eventually I gave up and continued using vanilla i3 instead, but had this idea floating around. It focuses on those shortcomings. Does it solve them? Not too sure myself, but it was a fun experiment.

## Requirements

- A compositor, since it literally draws windows on top of your windows.
- GTK needs to be on your system to build this application, follow the instructions [here](https://crates.io/crates/gtk)

Since the binaries are not distributed yet, and this hasn't been published as a crate either, the only way is to clone the repo and then run with

```
cargo build && cargo run
```

## Running and customizing

By default this application should use the `style.css` in the root of the repo, but you can provide your own using the `--styles` arg and providing a path to your own styles.

The css file dictates how the window decorations look like, and they have 2 elements to customize, `#box` which represents the boundaries of the window and `#label`, which has the window title. Additionally, there's an `.animate` class applied to the parent which can help triggering animations for both the box and label.

#### Note that the CSS is not full spec, and you can see the supported properties by GTK [here](https://developer.gnome.org/gtk3/stable/chap-css-properties.html)

A simple example for this, animated using transitions:

```css
#box {
  background: rgba(255, 200, 0, 0.2);
  transition: background 2s ease, margin 0.2s ease;
  margin: 10px;
}

.animate #box {
  background: transparent;
  margin: 0px;
}

#label {
  opacity: 0; /* Hide the label */
}
```

This will produce the following

https://user-images.githubusercontent.com/4857535/124782792-8349f680-df12-11eb-8231-4a356d33f066.mp4
