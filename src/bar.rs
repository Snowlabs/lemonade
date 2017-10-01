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
    size: (i32, i32),
}

impl Bar<window::XCB> {
    pub fn new_xcb() -> Bar<window::XCB> {
        let window = window::XCB::new();
        window.dock();

        let surface =  window.create_surface();
        let cr = cairo::Context::new(&surface);

        let size = (1, 1);

        let mut r = Bar {
            window,
            surface,
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

    pub fn draw(&self, v: &Vec<format::FormatItem>) {

        // TODO: remove extra fillers
        //loop {
            //if let Some(&format::FormatItem::Filler(_)) = v.last() {
                //v.pop();
            //} else {
                //break;
            //}
        //}

        let cr = cairo::Context::new(&self.surface);
        let (bw, bh) = self.size;

        let count = self.filler_count(&v);
        let inter = bw as f64 / count as f64;
        let lengths = self.get_lengths(&v);
        let mut n = 0;
        let mut pos = 0.0;

        for (i, v) in v.iter().enumerate() {
            match *v {
                format::FormatItem::Text(ref t) => {
                    let font = pango::FontDescription::from_string(&t.font);
                    let layout = cr.create_pango_layout();
                    layout.set_font_description(&font);
                    layout.set_text(&t.text);

                    let (w, _) = layout.get_pixel_size();

                    cr.set_source_rgba(t.bg.r, t.bg.g, t.bg.b, t.bg.a);
                    cr.rectangle(0.0, 0.0, w as f64, bh as f64);
                    cr.fill();

                    cr.set_source_rgba(t.fg.r, t.fg.g, t.fg.b, t.fg.a);
                    cr.show_pango_layout(&layout);

                    cr.translate(w as f64, 0.0);

                    //pos += w as f64;
                }

                format::FormatItem::Filler(ref f) => {
                    n += 1;
                    let mut pnext = (inter * n as f64) - pos;

                    if count == n {
                        pnext -= lengths[n as usize];
                    } else {
                        pnext -= lengths[n as usize] / 2.0;
                    }

                    cr.set_source_rgba(f.bg.r, f.bg.g, f.bg.b, f.bg.a);
                    cr.rectangle(0.0, 0.0, pnext, bh as f64);
                    cr.fill();

                    cr.translate(pnext, 0.0);
                    pos += pnext;
                }
            }
        }

        self.window.flush();
    }

    fn filler_count(&self, v: &Vec<format::FormatItem>) -> i32 {
        let mut r = 0;

        for i in v {
            if let format::FormatItem::Filler(_) = *i {
                r += 1;
            }
        }

        return r;
    }

    fn get_lengths(&self, v: &Vec<format::FormatItem>) -> Vec<f64> {
        let mut r: Vec<f64> = Vec::new(); // return val

        let lay = cairo::Context::new(&self.surface);
        let lay = lay.create_pango_layout();

        let mut n = 0.0;
        for i in v.iter() {
            match *i {
                format::FormatItem::Text(ref t) => {
                    let font = pango::FontDescription::from_string(&t.font);
                    lay.set_text(&t.text);
                    lay.set_font_description(&font);

                    let (w, _) = lay.get_pixel_size();
                    n += w as f64;
                }

                format::FormatItem::Filler(_) => {
                    r.push(n);
                    n = 0.0;
                }
            }
        }

        r.push(n);
        return r;
    }
}
