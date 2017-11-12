extern crate pango;
extern crate cairo;
extern crate cairo_sys;
extern crate pangocairo;
extern crate xcb;

#[cfg(feature = "image")]
extern crate gdk;

#[cfg(feature = "image")]
extern crate gdk_pixbuf;

pub mod bar;
pub mod format;
pub mod window;

pub use bar::Bar;
