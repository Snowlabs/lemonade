extern crate lemonade;
extern crate regex;

use std::cell::RefCell;
use std::io;
use std::str::FromStr;
use regex::Regex;
use lemonade::Bar;
use lemonade::format::{FormatItem, Text, Filler, Color};

fn main() {

    let mut bar = Bar::new_xcb();
    bar.size(1920, 30);

    let mut buf = String::new();
    let mut lem = LemonParser::new();

    loop {
        buf.clear();
        if let Err(_) = io::stdin().read_line(&mut buf) {
            continue;
        }

        let f = lem.parse(&buf);

        bar.draw(&f);
    }
}

pub struct LemonParser {
    bg: Color,
    fg: Color,
    re: Regex,
}

impl LemonParser {
    pub fn new() -> Self {
        let re = Regex::new(concat!(
            r"%\{(?P<type>",
                "[lcr]", "|",
                "[BF]", "#(?P<colo>(?:[[:xdigit:]]{3,4}){1,2})", "|",
                "T", "(?P<index>-|[1-9])", "|",
                "R",
            r")\}",
            )
        ).unwrap();

        let bg = Color::new(1.0, 1.0, 1.0, 1.0);
        let fg = Color::new(0.0, 0.0, 0.0, 1.0);

        Self {
            bg,
            fg,
            re,
        }
    }

    pub fn parse(&mut self, fmt: &str) -> Vec<FormatItem> {
        // Temporary variables for computing string slices
        let mut bpos: usize = 0;
        let mut epos: usize = 0;

        // Colour storage
        let bg = RefCell::new(self.bg.clone());
        let fg = RefCell::new(self.fg.clone());

        // List of fonts and the current font
        // TODO: tmp
        let font_list = vec![String::from("Noto Sans 15"),
                             String::from("Noto Serif 15"),
                             String::from("Source Code Pro 15"),
                             String::from("Terminus 15")];
        let font = RefCell::new(font_list[..].join(", "));

        // Return vector
        let mut v: Vec<Vec<FormatItem>> = Vec::with_capacity(3);
        for _ in 0..3 { v.push(Vec::new()); }

        // Index of currently processed vector
        let mut i: usize = 0;

        // Push s into the vector
        let pusht = |v: &mut Vec<FormatItem>, s: &str| {
            if ! s.is_empty() {
                if let Some(&FormatItem::Filler(_)) = v.last() {
                    v.pop();
                }
            } else {
                return
            }

            v.push(FormatItem::Text(Text {
                bg: bg.borrow().clone(),
                fg: fg.borrow().clone(),
                text: String::from(s),
                font: font.borrow().clone(),
            }));
        };

        // Push a filler into the vector
        let pushf = |v: &mut Vec<FormatItem>| {
            if let Some(&FormatItem::Filler(_)) = v.last() {
                v.pop();
            } else {
                v.push(FormatItem::Filler(Filler {
                    bg: bg.borrow().clone(),
                }));
            }
        };

        let swapc = || {
            // TODO: somehow get std::mem::swap to work
            let t = bg.borrow().clone();
            *bg.borrow_mut() = fg.borrow().clone();
            *fg.borrow_mut() = t;
        };

        // Iterate through every formatting item
        for mat in self.re.find_iter(fmt) {
            let caps = self.re.captures(mat.as_str()).unwrap();
            epos = mat.start();

            pusht(&mut v[i], &fmt[bpos..epos]);

            match *&caps["type"].chars().nth(0).unwrap() {
                'l' => {
                    if i != 0 {
                        pushf(&mut v[i]);
                        i = 0;
                    }
                }

                'c' => {
                    if i != 1 {
                        pushf(&mut v[i]);
                        i = 1;
                    }
                }

                'r' => {
                    if i != 2 {
                        pushf(&mut v[i]);
                        i = 2;
                    }
                }

                'F' => {
                    *fg.borrow_mut() = Color::from_hex(&caps["colo"]).unwrap();
                }

                'B' => {
                    *bg.borrow_mut() = Color::from_hex(&caps["colo"]).unwrap();
                }

                'T' => {
                    if &caps["index"] == "-" {
                        *font.borrow_mut() = font_list[..].join(", ");
                    } else {

                        // 1-based indexing
                        let i = usize::from_str(&caps["index"]).unwrap() - 1;

                        if i > font_list.len() {
                            eprintln!("Font index {} is too high", i);
                            *font.borrow_mut() = font_list[..].join(", ");
                        } else {
                            *font.borrow_mut() = font_list.get(i).unwrap().clone();
                        }
                    }
                }

                'R' => {
                    swapc();
                }

                _ => {}
            }

            bpos = mat.end();
        }

        if bpos > epos {
            epos = fmt.len();
        }

        pusht(&mut v[i], &fmt[bpos..epos]);

        // TODO: clean this up
        let mut r: Vec<FormatItem> = Vec::new();
        for i in 0..3 {
            r.extend_from_slice(&v[i]);
        }
        return r;
    }
}
