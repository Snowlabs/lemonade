use std::sync::Arc;
use std::sync::Mutex;
use std::ops::Drop;
use std::thread;
use window::Dock;

use cairo;
use cairo::XCBSurface;
use cairo_sys;
use xcb;
use xcb::*;


fn get_visualid_from_depth(scr: Screen, depth: u8) -> (Visualid, u8) {
    for d in scr.allowed_depths() {
        if depth == d.depth() {
            for v in d.visuals() {
                return (v.visual_id(), depth);
            }
        }
    }

    // If no depth matches return root visual
    return (scr.root_visual(), scr.root_depth());
}

pub struct XCB {

    conn:     Arc<Connection>,
    scr_num:  i32,

    win:      Window,
    root:     Window,
    bufpix:   Pixmap,
    gc:       Gcontext,
    colour:   Colormap,
    visual:   Visualid,
    depth:    u8,

    size:     (u16, u16), // (w, h)
    pos:      (i16, i16), // (x, y)
    scr_size: (u16, u16),
    bottom:   bool,

    click_fn: Arc<Mutex<Box<Fn(i16, i16, u8) + Sync + Send>>>,
}

impl XCB {
    pub fn new() -> XCB {

        // Create XCB struct to return
        let (conn, scr_num) = {
            let (conn, scr_num) = Connection::connect(None).unwrap();
            (Arc::new(conn), scr_num)
        };
        let win = conn.generate_id();
        let gc = conn.generate_id(); // The GC is created later
        let colour = conn.generate_id();
        let click_fn: Arc<Mutex<Box<Fn(i16, i16, u8) + Sync + Send>>> =
            Arc::new(Mutex::new(Box::new(|_, _, _| {} // Placeholder closure
        )));
        let bufpix = conn.generate_id(); // Pixmap created later
        let size = (1u16, 1u16); // default size

        let root;
        let visual;
        let depth;
        let mut scr_size = (0u16, 0u16);
        {
            let screen = conn.get_setup()
                             .roots()
                             .nth(scr_num as usize)
                             .unwrap();
            scr_size.0 = screen.width_in_pixels();
            scr_size.1 = screen.height_in_pixels();
            root = screen.root();

            let (v, d) = get_visualid_from_depth(screen, 32);
            visual = v;
            depth = d;
        }

        let x = XCB {
            conn,
            scr_num,
            win,
            root,
            bufpix,
            gc,
            colour,
            visual,
            depth,
            size,
            pos:         (0, 0),
            scr_size,
            bottom:      false,
            click_fn,
        };

        // Create the window
        // Masks to use
        create_colormap(&*x.conn, COLORMAP_ALLOC_NONE as u8,
                        x.colour, x.root,
                        x.visual)
            .request_check().unwrap();

        let values = [
            (CW_EVENT_MASK, EVENT_MASK_BUTTON_PRESS | EVENT_MASK_EXPOSURE),
            (CW_BACK_PIXEL, 0),
            (CW_COLORMAP, x.colour),
            (CW_BORDER_PIXEL, 0),
        ];

        create_window(&*x.conn,
                      x.depth,
                      x.win,
                      x.root,
                      x.pos.0, x.pos.1,
                      x.size.0, x.size.1,
                      0,
                      WINDOW_CLASS_INPUT_OUTPUT as u16,
                      x.visual,
                      &values)
            .request_check().unwrap();

       let title = "lemonade";
        change_property(&*x.conn, xcb::PROP_MODE_REPLACE as u8, x.win, 
            xcb::ATOM_WM_NAME, xcb::ATOM_STRING, 8, title.as_bytes());

        create_gc(&*x.conn, x.gc, x.win, &[]);
        create_pixmap(&*x.conn, x.depth, x.bufpix,
                      x.win, x.size.0, x.size.1);

        // Create event-monitoring thread
        let conn = x.conn.clone();
        let click_fn = x.click_fn.clone();
        let win = x.win;
        let bufpix = x.bufpix;
        let gc = x.gc;
        thread::spawn(move || {
            while let Some(e) = conn.wait_for_event() {
                match e.response_type() as u8 {
                    BUTTON_PRESS => {
                        let e: &ButtonPressEvent = unsafe {
                            cast_event(&e)
                        };

                        let (x, y) = (e.event_x(), e.event_y());
                        let b = e.detail();
                        let f = click_fn.lock().unwrap();

                        f(x, y, b);
                    }

                    EXPOSE => {
                        let e: &ExposeEvent = unsafe {
                            cast_event(&e)
                        };

                        let w = e.width();
                        let h = e.height();
                        let x = e.x() as i16;
                        let y = e.y() as i16;
                        copy_area(&*conn, bufpix, win, gc,
                                  x, y, x, y, w, h);

                        conn.flush();
                    }

                    _ => {}
                }
            }

            println!("ERROR");
        });

        return x;
    }

    fn map_window(&self) {
        map_window(&self.conn, self.win);
    }

    fn unmap_window(&self) {
        unmap_window(&self.conn, self.win);
    }

    fn reposition_window(&mut self) {
        self.unmap_window();

        let mut data: [i16; 12] = [
            0, 0, 0, 0, // left, right, top, bottom
            0, 0, // left offset
            0, 0, // right offset
            0, 0, // top offset
            0, 0, // bottom offset
        ];

        let curr_x = self.pos.0;
        let (xb, xe) = (curr_x, curr_x + self.size.0 as i16);

        let ypos;
        if self.bottom {
            ypos = self.scr_size.1 as i16 - self.size.1 as i16;

            data[2]  = 0; // top offset
            data[3]  = self.size.1 as i16;
            data[8]  = 0;  data[9]  = 0;
            data[10] = xb; data[11] = xe;
        } else {
            ypos = 0;

            data[2]  = self.size.1 as i16;
            data[3]  = 0; // bottom offset
            data[8]  = xb; data[9]  = xe;
            data[10] = 0;  data[11] = 0;
        }

        self.set_pos(curr_x as u16, ypos as u16);

        change_property(&self.conn,
                        PROP_MODE_REPLACE as u8,
                        self.win,
                        self.get_atom("_NET_WM_STRUT_PARTIAL"),
                        ATOM_ATOM,
                        16,
                        &data);

        self.map_window();
    }

    fn get_atom(&self, name: &str) -> Atom {
        let atom = intern_atom(&self.conn, false, name);

        atom.get_reply().unwrap().atom()
    }

    fn get_screen(&self) -> Screen {
        let setup = self.conn.get_setup();
        let screen = setup.roots().nth(self.scr_num as usize).unwrap();

        return screen;
    }

    fn get_visual(&self) -> Visualtype {
        for d in self.get_screen().allowed_depths() {
            for v in d.visuals() {
                if v.visual_id() == self.visual {
                    return v;
                }
            }
        }

        panic!("Failed to find visual type");
    }

    /// Set a new size for the window.
    ///
    /// Note: This clears the buffer, so make sure to draw
    /// after setting the size and not before. Else, the
    /// drawn image is lost.
    fn set_size(&mut self, w: u16, h: u16) {

        // Update the pixmap to match new size
        free_pixmap(&self.conn, self.bufpix);
        create_pixmap(&self.conn, self.depth, self.bufpix,
                      self.win, w, h);

        // Clear the new pixmap
        change_gc(&*self.conn, self.gc, &[(GC_FUNCTION, GX_CLEAR)]);
        copy_area(&*self.conn, self.bufpix, self.bufpix, self.gc,
                  0, 0, 0, 0, w, h);
        change_gc(&*self.conn, self.gc, &[(GC_FUNCTION, GX_COPY)]);

        // Set the size
        configure_window(&*self.conn, self.win, &[
                (CONFIG_WINDOW_WIDTH as u16, w as u32),
                (CONFIG_WINDOW_HEIGHT as u16, h as u32),
        ]).request_check()
          .unwrap();

        self.size = (w, h);
    }

    /// Set the internal position value.
    ///
    /// Cannot move the window if it is docked. The `reposition_window` method
    /// must be used if it is docked.
    fn set_pos(&mut self, x: u16, y: u16) {
        configure_window(&self.conn, self.win, &[
                (CONFIG_WINDOW_X as u16, x as u32),
                (CONFIG_WINDOW_Y as u16, y as u32),
        ]).request_check()
          .unwrap();

        self.pos = (x as i16, y as i16);
    }
}


impl Dock for XCB {
    fn create_surface(&self) -> cairo::Surface {

        // Prepare cairo variables
        let cr_conn = unsafe {
            cairo::XCBConnection::from_raw_none(
                self.conn.get_raw_conn() as *mut cairo_sys::xcb_connection_t)
        };

        let cr_draw = cairo::XCBDrawable(self.bufpix);

        let cr_visual = unsafe {
            cairo::XCBVisualType::from_raw_none(
                &mut self.get_visual().base as *mut ffi::xcb_visualtype_t
                                            as *mut cairo_sys::xcb_visualtype_t)
        };

        // Create the surface using previous variables
        return cairo::Surface::create(
            &cr_conn, &cr_draw, &cr_visual,
            self.size.0 as i32, self.size.1 as i32);
    }

    fn dock(&self) {
        let data = [
            self.get_atom("_NET_WM_WINDOW_TYPE_DOCK"),
        ];

        change_property(&self.conn,
                             PROP_MODE_REPLACE as u8,
                             self.win,
                             self.get_atom("_NET_WM_WINDOW_TYPE"),
                             xcb::ATOM_ATOM,
                             32,
                             &data)
            .request_check()
            .expect("Failed to dock window");
    }

    fn top(&mut self) {
        self.bottom = false;
        self.reposition_window();
    }

    fn bottom(&mut self) {
        self.bottom = true;
        self.reposition_window();
    }

    fn set_size(&mut self, w: u16, h: u16) {
        self.set_size(w, h);
    }

    fn set_offset(&mut self, x: u16, y: u16) {
        if self.bottom {
            let screen_height = self.scr_size.1;
            self.set_pos(x, screen_height - y);
        } else {
            self.set_pos(x, y);
        }

        self.reposition_window();
    }

    fn get_screen_size(&self) -> (u16, u16) {
        (self.scr_size.0, self.scr_size.1)
    }

    fn flush(&self) {
        copy_area(&*self.conn, self.bufpix, self.win, self.gc,
                  0, 0, 0, 0, self.size.0, self.size.1);
        self.conn.flush();
    }

    fn click_cb<F>(&mut self, f: F)
        where F: Fn(i16, i16, u8) + Send + Sync + 'static {

        let mut cb = self.click_fn.lock().unwrap();
        *cb = Box::new(f);
    }
}

impl Drop for XCB {
    fn drop(&mut self) {
        free_pixmap(&*self.conn, self.win);
        free_pixmap(&*self.conn, self.bufpix);
        free_gc(&*self.conn, self.gc);
        free_colormap(&*self.conn, self.colour);
    }
}
