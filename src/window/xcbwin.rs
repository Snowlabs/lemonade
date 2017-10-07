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

    pub conn: Arc<Connection>,
    win: u32,
    screen_num: i32,

    size: (u16, u16), // (w, h)
    pos: (i16, i16), // (x, y)

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
        let size: (u16, u16) = (1, 1); // minimum size
        let pos: (i16, i16) = (0, 0);

        let click_fn: Arc<Mutex<Box<Fn(i16, i16, u8) + Sync + Send>>> =
            Arc::new(Mutex::new(Box::new(|_, _, _| {} // Placeholder closure
        )));

        let x = XCB {
            conn,
            win,
            screen_num,
            size,
            pos,
            click_fn,
        };

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

                    }

                    _ => {}
                }
            }
        });

        // Create the window
        x.create_win();

        return x;
    }

    fn create_win(&self) {

        // Masks to use
        let values = [
            (CW_EVENT_MASK, EVENT_MASK_BUTTON_PRESS
                           |EVENT_MASK_EXPOSURE),
        ];

        xcb::create_window(&self.conn,
                           xcb::COPY_FROM_PARENT as u8,
                           self.win,
                           self.get_screen().root(),
                           self.pos.0, self.pos.1,
                           self.size.0, self.size.1,
                           0,
                           xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
                           self.get_screen().root_visual(),
                           &values);

        xcb::map_window(&self.conn, self.win);
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

    fn set_size(&mut self, w: u16, h: u16) {
        xcb::configure_window(&self.conn, self.win, &[
                (xcb::CONFIG_WINDOW_WIDTH as u16, w as u32),
                (xcb::CONFIG_WINDOW_HEIGHT as u16, h as u32),
        ]);

        self.size = (w, h);
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
