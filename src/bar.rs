use cairo;
use pango;
use pangocairo;

use cairo::XCBSurface;
use pango::LayoutExt;
use pangocairo::CairoContextExt;

use window;
use window::Dock;

pub struct Bar<T> {
    pub window: T,

    pub surface: cairo::Surface,
    pub cr: cairo::Context,
    pub font: pango::FontDescription,
    pub layout: pango::Layout,

    size: (i32, i32),
}

impl Bar<window::XCB> {
    pub fn new_xcb() -> Bar<window::XCB> {
        let mut window = window::XCB::new();
        window.dock();

        let surface =  window.create_surface();
        let cr = cairo::Context::new(&surface);

        //let font = pango::FontDescription::new();
        let font = pango::FontDescription::from_string("Noto Sans 15");
        let layout = cr.create_pango_layout();
        layout.set_font_description(&font);

        let size = (1, 1);

        let mut r = Bar {
            window,
            surface,
            cr,
            font,
            layout,
            size,
        };

        return r;
    }
}

impl<T: Dock> Bar<T> {
    pub fn size(&mut self, w: i32, h: i32) {
        assert!(w > 0);
        assert!(h > 0);

        self.size = (w, h);
        self.window.size(w as u16, h as u16);
        self.surface.set_size(w, h);
    }

    // TODO: str will be replaced with a better implementation
    // TODO: most code is here is temporary
    pub fn draw(&self, input: &str) {
        let cr = &self.cr;

        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.paint();

        self.layout.set_text(&input);
        let (_, bound) = self.layout.get_pixel_extents();
        cr.set_source_rgb(0.5, 0.5, 0.5);
        cr.rectangle(bound.x as f64, 0.0,
                     bound.width as f64, self.size.1 as f64);
        cr.fill();

        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.show_pango_layout(&self.layout);

        self.window.flush();
    }
}
