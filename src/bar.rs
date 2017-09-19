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

    pub fn draw(&self, input: &Vec<Vec<format::FormatItem>>) {
        let cr = cairo::Context::new(&self.surface);
        let (bw, _) = self.size;

        let inter = bw as f64 / (input.len() - 1) as f64;
        let mut pos = 0.0;
        let mut pnext = 0.0;

        for (i, v) in input.iter().enumerate() {
            let l = self.get_format_length(&v);
            // Compute beginning of next vec
            match input.get(i + 2) {
                Some(_) => {
                    pnext = inter * (i as f64 + 1.0);
                    pnext -= l;
                    pnext -= self.get_format_length(&input[i + 1]) / 2.0;
                    pnext -= pos;
                }

                None => {
                    pnext = bw as f64;
                    pnext -= self.get_format_length(&v);
                    if let Some(vn) = input.get(i + 1) {
                        pnext -= self.get_format_length(&vn);
                    }
                    pnext -= pos;
                }
            }

            self.format_to_cr(&v, &cr, pnext);
            pos += l;
            pos += pnext;
            cr.translate(pnext, 0.0);
        }

        self.window.flush();
    }

    fn format_to_cr(&self, v: &Vec<format::FormatItem>,
                    cr: &cairo::Context, to: f64) {
        let lay = &self.layout;
        let (_, bh) = self.size; // bh := bar height
        let cr = cr.clone();

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

                format::FormatItem::Filler(ref t) => {
                    cr.set_source_rgba(t.bg.r, t.bg.g, t.bg.b, t.bg.a);
                    cr.rectangle(0.0, 0.0,
                                 to, bh as f64);
                    cr.fill();
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
                _ => {}
            }
        }

        return r;
    }
}
