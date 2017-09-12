extern crate cairo_sys;
extern crate cairo;
extern crate xcb;

use cairo::XCBSurface;

fn get_visual_type(conn: &xcb::Connection, scr: &xcb::Screen) -> xcb::Visualtype {
    for root in conn.get_setup().roots() {
        for d in root.allowed_depths() {
            for v in d.visuals() {
                if v.visual_id() == scr.root_visual() {
                    return v;
                }
            }
        }
    }
    panic!("Failed to find visual type");
}

fn main() {

    // Prepare XCB Window
    let (conn, scr_num) = xcb::Connection::connect(None).unwrap();
    let setup = conn.get_setup();
    let screen = setup.roots().nth(scr_num as usize).unwrap();

    let win = conn.generate_id();

    let values = [
        //(xcb::CW_BACK_PIXEL, screen.white_pixel()),
        (xcb::CW_OVERRIDE_REDIRECT, 1),
        (xcb::CW_EVENT_MASK, xcb::EVENT_MASK_EXPOSURE | xcb::EVENT_MASK_KEY_PRESS),
    ];


    xcb::create_window(&conn,
                       xcb::COPY_FROM_PARENT as u8,
                       win,
                       screen.root(),
                       50, 50,
                       100, 100,
                       0,
                       xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
                       screen.root_visual(),
                       &values);

    xcb::map_window(&conn, win);

    // Prepare cairo surface
    let cr_conn = unsafe {
        cairo::XCBConnection::from_raw_none(
            conn.get_raw_conn() as *mut cairo_sys::xcb_connection_t
        )
    };

    let cr_visual = unsafe {
        cairo::XCBVisualType::from_raw_none(
            &mut get_visual_type(&conn, &screen).base
                as *mut xcb::ffi::xcb_visualtype_t
                as *mut cairo_sys::xcb_visualtype_t
        )
    };

    let cr_draw = cairo::XCBDrawable(win);

    // Crate cairo surface
    let surface = cairo::Surface::create(
            &cr_conn, &cr_draw, &cr_visual,
            100, 100);
    let cr = cairo::Context::new(&surface);


    // Set up loop
    conn.flush();
    loop {
        conn.flush();

        let e = conn.wait_for_event();
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
