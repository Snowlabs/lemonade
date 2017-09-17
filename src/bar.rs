use cairo;
use pango;

use format;
use window;
use window::Dock;

use cairo::XCBSurface;
use pango::LayoutExt;
use pangocairo::CairoContextExt;

pub struct Bar<T> {
    pub window: T,

    pub surface: cairo::Surface,
    pub font: pango::FontDescription,
    pub layout: pango::Layout,

    size: (i32, i32),
}

impl Bar<window::XCB> {
    pub fn new_xcb() -> Bar<window::XCB> {
        let window = window::XCB::new();
        window.dock();

        let surface =  window.create_surface();
        let cr = cairo::Context::new(&surface);

        //let font = pango::FontDescription::new(); // TODO: un-hardcode
        let font = pango::FontDescription::from_string("Noto Sans 15");
        let layout = cr.create_pango_layout();
        layout.set_font_description(&font);

        let size = (1, 1);

        let mut r = Bar {
            window,
            surface,
            //cr,
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

    pub fn draw(&self, input: &format::Format) {
        let cr = cairo::Context::new(&self.surface);
        let (bw, bh) = self.size;

        // Format vectors for each area
        let lfmt = &input.left;
        let cfmt = &input.center;
        let rfmt = &input.right;

        // Length for each format vector
        // Used for translating the right amount
        let lpos = self.get_format_length(&lfmt);
        let cpos = self.get_format_length(&cfmt);
        let rpos = self.get_format_length(&rfmt);

        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.paint();

        // Paint from left
        self.format_to_cr(&lfmt, &cr); // Moves the cursor

        cr.translate((bw as f64 / 2.0) - (lpos + cpos / 2.0), 0.0);
        self.format_to_cr(&cfmt, &cr); // Moves the cursor

        cr.translate((bw as f64 / 2.0) - (rpos + cpos / 2.0), 0.0);
        self.format_to_cr(&rfmt, &cr);

        self.window.flush();
    }

    fn format_to_cr(&self, v: &Vec<format::FormatItem>, cr: &cairo::Context) {
        let lay = &self.layout;
        let (_, bh) = self.size; // bh := bar height

        for i in v {
            match *i {
                format::FormatItem::Text(ref t) => {
                    lay.set_text(&t.text);
                    let (w, _) = lay.get_pixel_size();

                    cr.set_source_rgba(t.bg.r, t.bg.g, t.bg.b, t.bg.a);
                    cr.rectangle(0.0, 0.0,
                                 w as f64, bh as f64);
                    cr.fill();

                    cr.set_source_rgba(t.fg.r, t.fg.g, t.fg.b, t.fg.a);
                    cr.show_pango_layout(&lay);

                    cr.translate(w as f64, 0.0);
                }
            }
        }

    }

    fn get_format_length(&self, v: &Vec<format::FormatItem>) -> f64 {
        let mut r: f64 = 0.0;
        let lay = self.layout.clone();
        lay.set_font_description(&self.font); // TODO: this will be replaced

        for i in v.iter() {
            match *i {
                format::FormatItem::Text(ref t) => {
                    lay.set_text(&t.text);
                    let (w, _) = lay.get_pixel_size();
                    r += w as f64;
                }
            }
        }

        return r;
    }
}
