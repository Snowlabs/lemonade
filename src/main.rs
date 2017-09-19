extern crate lemonade;

use lemonade::Bar;
use lemonade::format::{FormatItem, Text, Filler, Color};
//use std::io;
use std::thread;
use std::time;

fn main() {

    let mut bar = Bar::new_xcb();
    bar.size(1920, 30);

    //let mut fmt = Format::new();

    let mut fmt: Vec<Vec<FormatItem>> = Vec::new();
    fmt.push(Vec::new());
    fmt.push(Vec::new());
    fmt.push(Vec::new());

    fmt[1].push(FormatItem::Text(Text {
        bg: Color::new(1.0, 1.0, 1.0, 1.0),
        fg: Color::new(0.2, 0.0, 0.0, 1.0),
        text: String::from("some center text, "),
    }));

    fmt[1].push(FormatItem::Text(Text {
        bg: Color::new(1.0, 1.0, 1.0, 1.0),
        fg: Color::new(0.2, 0.0, 0.0, 0.7),
        text: String::from("other center text"),
    }));

    fmt[1].push(FormatItem::Filler(Filler {
        bg: Color::new(0.0, 0.0, 0.0, 1.0),
    }));

    fmt[2].push(FormatItem::Text(Text {
        bg: Color::new(1.0, 8.0, 8.0, 1.0),
        fg: Color::new(0.0, 0.2, 0.0, 0.5),
        text: String::from("right text"),
    }));

    fmt[0].push(FormatItem::Text(Text {
        bg: Color::new(1.0, 8.0, 8.0, 1.0),
        fg: Color::new(0.0, 0.2, 0.0, 1.0),
        text: String::from("left text"),
    }));

    fmt[0].push(FormatItem::Filler(Filler {
        bg: Color::new(0.0, 0.0, 0.0, 1.0),
    }));

    //let mut buf = String::new();
    let one_s = time::Duration::new(1, 0);
    loop {
        //buf.clear();

        //if let Err(_) = io::stdin().read_line(&mut buf) {
            //continue;
        //}

        bar.draw(&fmt);
        thread::sleep(one_s);
    }
}
