use cairo;
use pango;

use format;
use window;
use window::Dock;

use std::num::ParseIntError;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use cairo::XCBSurface;
use pango::LayoutExt;
use pangocairo::CairoContextExt;

pub struct Bar<T: Dock> {
    window: T,
    surface: cairo::Surface,
    fmt: Vec<format::FormatItem>,
    cmds: Arc<Mutex<Vec<(u8, String, i16, i16)>>>, // (mbutton, cmd, minx, maxx)
    size: (i32, i32),
}

impl Bar<window::XCB> {

    /// Create an instance of Bar using XCB.
    ///
    /// By default, the width is set to the width of the screen and
    /// the height is 25. The bar is drawn at the top first, which can
    /// be set after.
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

        let width = r.window.get_screen_size().0 as i32;
        r.set_size(width, 25);

        // Set callbacks
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
        assert!(w >= 0);
        assert!(h >= 0);

        self.size = (w, h);
        self.window.set_size(w as u16, h as u16);
        self.surface.set_size(w, h);
    }

    pub fn set_offset(&mut self, x: i32, y: i32) {
        assert!(x >= 0);
        assert!(y >= 0);

        self.window.set_offset(x as u16, y as u16);
    }

    /// Set the bars geometry with WxH+x+y format.
    ///
    /// This does not validate the string and might return Err. If the
    /// width is omitted, it defaults to the screen width. The height
    /// defaults to 25, for absolutely no reason.
    ///
    /// The position arguments x and y both default to 0.
    pub fn set_geometry(&mut self, g: &str) -> Result<(), ParseIntError> {
        fn stoi(s: &str) -> Result<i32, ParseIntError> {
            if s.len() == 0 { Ok(0) }
            else { i32::from_str(s) }
        }

        let (mut w, mut h) = (self.window.get_screen_size().0 as i32, 0);
        let (mut x, mut y) = (0, 0);

        let mut pl1 = g.len();
        if let Some(i) = g.find('+')  {
            pl1 = i;

            if let Some(i) = g.rfind('+') {
                x = stoi(&g[pl1+1..i])?;
                y = stoi(&g[i+1..g.len()])?;
            } else {
                x = stoi(&g[pl1..g.len()])?;
            }
        }

        if let Some(i) = g.find('x') {
            if let Some(s) = g.get(0..i) {
                w = stoi(s)?;
            }

            if let Some(s) = g.get(i+1..pl1) {
                h = stoi(s)?;
            }
        } else {
            w = stoi(&g[0..pl1])?;
        }

        self.set_size(w, h);
        self.set_offset(x, y);

        Ok(())
    }

    pub fn bottom(&mut self, b: bool) {
        if b { self.window.bottom(); }
        else { self.window.top();    }
    }

    pub fn set_fmt(&mut self, f: Vec<format::FormatItem>) {
        self.fmt = f;
    }

    // TODO: remove the insane repetition
    pub fn draw(&self) {


        let mut cmds = self.cmds.lock().unwrap();
        *cmds = Vec::new();

        let cr = cairo::Context::new(&self.surface);
        cr.set_operator(cairo::Operator::Source);

        let (bw, bh) = self.size;
        let count = self.filler_count();
        let inter = bw as f64 / count as f64;
        let lengths = self.get_lengths();
        let mut n = 0;
        let mut pos = 0.0;

        // Take care of drawing background and adding commands
        let mut draw_bg = |bg: &format::BG, pos: f64, w: f64| {

            // Text background
            cr.set_source_rgba(bg.bg.r, bg.bg.g, bg.bg.b, bg.bg.a);
            cr.rectangle(0.0, 0.0, w, bh as f64);
            cr.fill();

            // Overline
            if let Some(ref ol) = bg.ol {
                cr.set_source_rgba(ol.r, ol.g, ol.b, ol.a);
                cr.rectangle(0.0, 0.0, w, bg.ol_size);
                cr.fill();
            }

            // Underline
            if let Some(ref ul) = bg.ul {
                cr.set_source_rgba(ul.r, ul.g, ul.b, ul.a);
                cr.rectangle(0.0, bh as f64 - bg.ol_size,
                             w, bh as f64);
                cr.fill();
            }

            for &(b, ref s) in &bg.cmd {
                cmds.push((b, s.clone(),
                    pos as i16, pos as i16 + w as i16));
            }
        };

        for v in &self.fmt {
            match *v {
                format::FormatItem::Text(ref t, ref bg) => {

                    // Set the font and text
                    let font = pango::FontDescription::from_string(&t.font);
                    let layout = cr.create_pango_layout();
                    layout.set_font_description(&font);
                    layout.set_text(&t.text);

                    let (w, h) = layout.get_pixel_size();

                    draw_bg(bg, pos, w as f64);

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

                format::FormatItem::Filler(ref bg) => {
                    n += 1;
                    let mut pnext = (inter * n as f64) - pos;

                    if count == n {
                        pnext -= lengths[n as usize];
                    } else {
                        pnext -= lengths[n as usize] / 2.0;
                    }

                    draw_bg(bg, pos, pnext);

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
                format::FormatItem::Text(ref t, _) => {
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
