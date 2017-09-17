extern crate bar;

use bar::Bar;
use bar::format::{Format, FormatItem, Text, Color};
use std::io;
use std::thread;
use std::time;

fn main() {

    let mut bar = Bar::new_xcb();
    bar.size(1920, 30);

    let mut fmt = Format::new();

    fmt.center.push(FormatItem::Text(Text {
        bg: Color::new(1.0, 1.0, 1.0, 1.0),
        fg: Color::new(0.2, 0.0, 0.0, 1.0),
        text: String::from("some center text, "),
    }));

    fmt.center.push(FormatItem::Text(Text {
        bg: Color::new(1.0, 1.0, 1.0, 1.0),
        fg: Color::new(0.2, 0.0, 0.0, 0.7),
        text: String::from("other center text"),
    }));

    fmt.right.push(FormatItem::Text(Text {
        bg: Color::new(1.0, 8.0, 8.0, 1.0),
        fg: Color::new(0.0, 0.2, 0.0, 0.5),
        text: String::from("right text"),
    }));

    fmt.left.push(FormatItem::Text(Text {
        bg: Color::new(1.0, 8.0, 8.0, 1.0),
        fg: Color::new(0.0, 0.2, 0.0, 1.0),
        text: String::from("left text"),
    }));

    //let mut buf = String::new();
    let ten_s = time::Duration::new(1, 0);
    loop {
        //buf.clear();

        //if let Err(_) = io::stdin().read_line(&mut buf) {
            //continue;
        //}

        bar.draw(&fmt);
        thread::sleep(ten_s);
    }
}
