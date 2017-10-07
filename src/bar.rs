use cairo;
use pango;

use format;
use window;
use window::Dock;

use std::sync::{Arc, Mutex};
use cairo::XCBSurface;
use pango::LayoutExt;
use pangocairo::CairoContextExt;

pub struct Bar<T> {
    window: T,
    surface: cairo::Surface,
    fmt: Vec<format::FormatItem>,
    cmds: Arc<Mutex<Vec<(u8, String, i16, i16)>>>, // (mbutton, cmd, minx, maxx)
    size: (i32, i32),
}

impl Bar<window::XCB> {
    pub fn with_xcb() -> Bar<window::XCB> {
        let window = window::XCB::new();
        window.dock();

        let surface =  window.create_surface();
        let fmt = Vec::new();
        let size = (1, 1);
        let cmds: Arc<Mutex<Vec<(u8, String, i16, i16)>>> =
            Arc::new(Mutex::new(Vec::new()));

        let mut r = Bar {
            window,
            surface,
            fmt,
            cmds,
            size,
        };

        let cmds = r.cmds.clone();
        r.window.click_cb(move |x, _, b| {
            let cmds = cmds.lock().unwrap();

            for &(mb, ref s, xl, xr) in cmds.iter() {
                if mb == b && x >= xl && x <= xr {
                    println!("{}", s);
                }
            }
        });

        return r;
    }
}

impl<T: Dock> Bar<T> {
    pub fn set_size(&mut self, w: i32, h: i32) {
        assert!(w > 0);
        assert!(h > 0);

        self.size = (w, h);
        self.window.set_size(w as u16, h as u16);
        self.surface.set_size(w, h);
    }

    // TODO: remove the insane repetition
    pub fn draw(&mut self, f: Vec<format::FormatItem>) {
        // TODO: remove extra fillers
        //loop {
            //if let Some(&format::FormatItem::Filler(_)) = v.last() {
                //v.pop();
            //} else {
                //break;
            //}
        //}

        self.fmt = f;
        let mut cmds = self.cmds.lock().unwrap();
        *cmds = Vec::new();

        let cr = cairo::Context::new(&self.surface);
        let (bw, bh) = self.size;

        let count = self.filler_count();
        let inter = bw as f64 / count as f64;
        let lengths = self.get_lengths();
        let mut n = 0;
        let mut pos = 0.0;

        for v in &self.fmt {
            match *v {
                format::FormatItem::Text(ref t) => {
                    // Set the font and text
                    let font = pango::FontDescription::from_string(&t.font);
                    let layout = cr.create_pango_layout();
                    layout.set_font_description(&font);
                    layout.set_text(&t.text);

                    let (w, h) = layout.get_pixel_size();

                    // Push commands
                    for &(b, ref s) in &t.cmd {
                        cmds.push((b, s.clone(),
                            pos as i16, pos as i16 + w as i16));
                    }

                    // Text background
                    cr.set_source_rgba(t.bg.r, t.bg.g, t.bg.b, t.bg.a);
                    cr.rectangle(0.0, 0.0, w as f64, bh as f64);
                    cr.fill();

                    // Overline
                    if let Some(ref ol) = t.ol {
                        cr.set_source_rgba(ol.r, ol.g, ol.b, ol.a);
                        cr.rectangle(0.0, 0.0, w as f64, t.ol_size);
                        cr.fill();
                    }

                    // Underline
                    if let Some(ref ul) = t.ul {
                        cr.set_source_rgba(ul.r, ul.g, ul.b, ul.a);
                        cr.rectangle(0.0, bh as f64 - t.ol_size,
                                     w as f64, bh as f64);
                        cr.fill();
                    }

                    // Text foreground
                    cr.save(); {
                        cr.set_source_rgba(t.fg.r, t.fg.g, t.fg.b, t.fg.a);
                        cr.translate(0.0, (bh - h) as f64 / 2.0);
                        cr.show_pango_layout(&layout);
                    } cr.restore();

                    // Move to next position
                    cr.translate(w as f64, 0.0);
                    pos += w as f64;
                }

                format::FormatItem::Filler(ref f) => {
                    n += 1;
                    let mut pnext = (inter * n as f64) - pos;

                    if count == n {
                        pnext -= lengths[n as usize];
                    } else {
                        pnext -= lengths[n as usize] / 2.0;
                    }

                    // Push commands
                    for &(b, ref s) in &f.cmd {
                        cmds.push((b, s.clone(),
                            pos as i16, pos as i16 + pnext as i16));
                    }

                    cr.set_source_rgba(f.bg.r, f.bg.g, f.bg.b, f.bg.a);
                    cr.rectangle(0.0, 0.0, pnext, bh as f64);
                    cr.fill();

                    // Overline
                    if let Some(ref ol) = f.ol {
                        cr.set_source_rgba(ol.r, ol.g, ol.b, ol.a);
                        cr.rectangle(0.0, 0.0, pnext as f64, f.ol_size);
                        cr.fill();
                    }

                    // Underline
                    if let Some(ref ul) = f.ul {
                        cr.set_source_rgba(ul.r, ul.g, ul.b, ul.a);
                        cr.rectangle(0.0, bh as f64 - f.ol_size,
                                     pnext as f64, bh as f64);
                        cr.fill();
                    }

                    cr.translate(pnext, 0.0);
                    pos += pnext;
                }
            }
        }

        self.window.flush();
    }

    fn filler_count(&self) -> i32 {
        let mut r = 0;

        for i in &self.fmt {
            if let &format::FormatItem::Filler(_) = i {
                r += 1;
            }
        }

        return r;
    }

    fn get_lengths(&self) -> Vec<f64> {
        let mut r: Vec<f64> = Vec::new(); // return val

        let lay = cairo::Context::new(&self.surface);
        let lay = lay.create_pango_layout();

        let mut n = 0.0;
        for i in &self.fmt {
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
