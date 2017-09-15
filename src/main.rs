extern crate cairo_sys;
extern crate cairo;
extern crate xcb;

mod window;

use window::Dock;

fn main() {

    let mut xcb = window::XCB::new();
    xcb.dock();

    xcb.size(100, 100);

    let surface = xcb.create_surface();
    let cr = cairo::Context::new(&surface);

    // Set up loop
    // TODO abstract this functionality
    loop {
        xcb.conn.flush();

        let e = xcb.conn.wait_for_event();
        match e {
            None => { break; }
            Some(event) => {
                match event.response_type() {

                    xcb::KEY_PRESS | xcb::EXPOSE => {
                        cr.set_source_rgb(1.0, 0.0, 0.0);
                        cr.paint();

                        cr.set_line_width(2.0);
                        cr.set_source_rgb(0.0, 0.0, 0.0);
                        cr.move_to(0.0, 0.0);
                        cr.line_to(100.0, 100.0);
                        cr.move_to(100.0, 0.0);
                        cr.line_to(0.0, 100.0);

                        cr.stroke();
                    }

                    _ =>  {}
                }
            }
        }
    }
}
