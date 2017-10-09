use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use window::Dock;

use cairo;
use cairo::XCBSurface;
use cairo_sys;
use xcb;
use xcb::*;


pub struct XCB {

    conn: Arc<Connection>,
    win: u32,
    screen_num: i32,

    screen_size: (u16, u16),

    size: (u16, u16), // (w, h)
    pos: (i16, i16), // (x, y)
    bottom: bool,

    click_fn: Arc<Mutex<Box<Fn(i16, i16, u8) + Sync + Send>>>,
}

impl XCB {
    pub fn new() -> XCB {

        // Create XCB struct to return
        let (conn, screen_num) = {
            let (conn, screen_num) = Connection::connect(None).unwrap();
            (Arc::new(conn), screen_num)
        };
        let win = conn.generate_id();

        let click_fn: Arc<Mutex<Box<Fn(i16, i16, u8) + Sync + Send>>> =
            Arc::new(Mutex::new(Box::new(|_, _, _| {} // Placeholder closure
        )));

        let mut x = XCB {
            conn,
            win,
            screen_num,
            click_fn,
            size:        (1, 1), // default size
            pos:         (0, 0),
            bottom:      false,

            // temporary, these are set later
            screen_size: (0, 0),
        };

        let (w, h);
        {
            let screen = x.get_screen();
            h = screen.height_in_pixels();
            w = screen.width_in_pixels();
        }
        x.screen_size = (h, w);

        // Create event-monitoring thread
        let conn = x.conn.clone();
        let click_fn = x.click_fn.clone();
        thread::spawn(move || {
            while let Some(e) =  conn.wait_for_event() {
                match e.response_type() as u8 {
                    BUTTON_PRESS => {
                        let e: &ButtonPressEvent = unsafe {
                            xcb::cast_event(&e)
                        };

                        let (x, y) = (e.event_x(), e.event_y());
                        let b = e.detail();
                        let f = click_fn.lock().unwrap();

                        f(x, y, b);
                    }

                    EXPOSE => {
                        conn.flush();
                    }

                    _ => {}
                }
            }
        });

        // Create the window
        // Masks to use
        let values = [
            (CW_EVENT_MASK, EVENT_MASK_BUTTON_PRESS | EVENT_MASK_EXPOSURE),
            (CW_BACK_PIXMAP, BACK_PIXMAP_NONE),
        ];

        xcb::create_window(&x.conn,
                           xcb::COPY_FROM_PARENT as u8,
                           x.win,
                           x.get_screen().root(),
                           x.pos.0, x.pos.1,
                           x.size.0, x.size.1,
                           0,
                           xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
                           x.get_screen().root_visual(),
                           &values);

        //x.map_window();

        return x;
    }

    fn map_window(&self) {
        xcb::map_window(&self.conn, self.win);
    }

    fn unmap_window(&self) {
        xcb::unmap_window(&self.conn, self.win);
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
            ypos = self.screen_size.1 as i16 - self.size.1 as i16;

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

        xcb::change_property(&self.conn,
                             xcb::PROP_MODE_REPLACE as u8,
                             self.win,
                             self.get_atom("_NET_WM_STRUT_PARTIAL"),
                             xcb::ATOM_ATOM,
                             16,
                             &data)
                             .request_check()
                             .unwrap();

        self.map_window();
    }

    fn get_atom(&self, name: &str) -> xcb::Atom {
        let atom = xcb::intern_atom(&self.conn, false, name);

        let reply = atom.get_reply().unwrap().atom();
        return reply;
    }

    // TODO somehow store this value in the struct instead of
    // getting it through a function
    fn get_screen(&self) -> xcb::Screen {
        let setup = self.conn.get_setup();
        let screen = setup.roots().nth(self.screen_num as usize).unwrap();

        return screen;
    }

    fn get_visual(&self) -> xcb::Visualtype {
        for root in self.conn.get_setup().roots() {
            for depth in root.allowed_depths() {
                for v in depth.visuals() {
                    if v.visual_id() == self.get_screen().root_visual() {
                        return v;
                    }
                }
            }
        }
        panic!("Failed to find visual type");
    }

    fn set_size(&mut self, w: u16, h: u16) {
        xcb::configure_window(&self.conn, self.win, &[
                (CONFIG_WINDOW_WIDTH as u16, w as u32),
                (CONFIG_WINDOW_HEIGHT as u16, h as u32),
        ]);

        self.size = (w, h);
    }

    /// Set the internal position value.
    ///
    /// Cannot move the window if it is docked. The `reposition_window` method
    /// must be used if it is docked.
    fn set_pos(&mut self, x: u16, y: u16) {
        xcb::configure_window(&self.conn, self.win, &[
                (CONFIG_WINDOW_X as u16, x as u32),
                (CONFIG_WINDOW_Y as u16, y as u32),
        ]);

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

        let cr_draw = cairo::XCBDrawable(self.win);

        let cr_visual = unsafe {
            cairo::XCBVisualType::from_raw_none(
                &mut self.get_visual().base as *mut xcb::ffi::xcb_visualtype_t
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

        xcb::change_property(&self.conn,
                             xcb::PROP_MODE_REPLACE as u8,
                             self.win,
                             self.get_atom("_NET_WM_WINDOW_TYPE"),
                             xcb::ATOM_ATOM,
                             32,
                             &data);
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
            let screen_height = self.screen_size.1;
            self.set_pos(x, screen_height - y);
        } else {
            self.set_pos(x, y);
        }

        self.reposition_window();
    }

    fn get_screen_size(&self) -> (u16, u16) {
        (self.screen_size.0, self.screen_size.1)
    }

    fn flush(&self) {
        self.conn.flush();
    }

    fn click_cb<F>(&mut self, f: F)
        where F: Fn(i16, i16, u8) + Send + Sync + 'static {

        let mut cb = self.click_fn.lock().unwrap();
        *cb = Box::new(f);
    }
}
