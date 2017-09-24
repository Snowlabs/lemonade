extern crate lemonade;
extern crate regex;

use std::io;
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
                "[BF]", "#(?P<colo>(?:[[:xdigit:]]{3,4}){1,2})", //"|",
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

    pub fn parse(&mut self, fmt: &str) -> Vec<Vec<FormatItem>> {
        // Temporary variables for computing string slices
        let mut bpos: usize = 0;
        let mut epos: usize = 0;

        // Index of the current sub-vector being processed
        let mut i: usize = 0;

        // Return value
        let mut v: Vec<Vec<FormatItem>> = Vec::with_capacity(3);
        for _ in 0..3 { v.push(Vec::new()); } // Initialise vector

        // Iterate through every formatting item
        for mat in self.re.find_iter(fmt) {
            let caps = self.re.captures(mat.as_str()).unwrap();
            epos = mat.start();

            self.pusht(&fmt[bpos..epos], &mut v[i]);

            match *&caps["type"].chars().nth(0).unwrap() {
                'l' => {
                    self.pushf(&mut v[i]);
                    i = 0;
                }

                'c' => {
                    self.pushf(&mut v[i]);
                    i = 1;
                }

                'r' => {
                    self.pushf(&mut v[i]);
                    i = 2;
                }

                'F' => {
                    self.fg = Color::from_hex(&caps["colo"]).unwrap();
                }

                'B' => {
                    self.bg = Color::from_hex(&caps["colo"]).unwrap();
                }

                _ => {}
            }

            bpos = mat.end();
        }

        if bpos > epos {
            epos = fmt.len();
        }

        self.pusht(&fmt[bpos..epos], &mut v[i]);

        return v;
    }

    fn pusht(&self, s: &str, v: &mut Vec<FormatItem>) {
        if ! s.is_empty() {

            if let Some(&FormatItem::Filler(_)) = v.last() {
                v.pop();
            }

            v.push(FormatItem::Text(Text {
                bg: self.bg.clone(),
                fg: self.fg.clone(),
                text: String::from(s),
            }))
        }
    }

    fn pushf(&self, v: &mut Vec<FormatItem>) {

        if let Some(&FormatItem::Filler(_)) = v.last() {
            v.pop();
        }

        v.push(FormatItem::Filler(Filler {
            bg: self.bg.clone(),
        }));
    }
}
