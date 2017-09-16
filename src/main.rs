extern crate bar;

use bar::Bar;
use std::io;

fn main() {

    let mut bar = Bar::new_xcb();
    bar.size(1920, 30);

    let mut buf = String::new();
    loop {
        buf.clear();

        if let Err(_) = io::stdin().read_line(&mut buf) {
            continue;
        }

        bar.draw(&buf);
    }
}
