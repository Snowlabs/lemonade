extern crate lemonade;
extern crate regex;

use std::cell::RefCell;
use std::io;
use std::str::FromStr;
use regex::Regex;
use lemonade::Bar;
use lemonade::format::{FormatItem, Text, Filler, Color};

fn main() {

    let mut bar = Bar::with_xcb();
    bar.set_size(1920, 30);

    let mut buf = String::new();
    let mut lem = LemonParser::new();

    loop {
        buf.clear();
        if let Err(_) = io::stdin().read_line(&mut buf) {
            continue;
        }

        let f = lem.parse(&buf);

        bar.draw(f);
    }
}

pub struct LemonParser {
    bg: Color,
    fg: Color,
    ol: Color,
    ul: Color,
    re: Regex,
}

impl LemonParser {
    pub fn new() -> Self {
        let re = Regex::new(concat!(
            r"%\{(?P<type>",
                "[lcr]", "|",
                "[BFUu]", "#(?P<colo>-|(?:[[:xdigit:]]{3,4}){1,2})", "|",
                "[!-+\\-]", "(?P<attr>[uo])", "|",
                "T", "(?P<index>-|[1-9])", "|",
                "A", "(?:(?P<butt>[1-9])?:(?P<cmd>(?:[^:]|\\\\:)+?):)?", "|",
                "R",
            r")\}",
            )
        ).unwrap();

        let bg = Color::new(1.0, 1.0, 1.0, 1.0);
        let fg = Color::new(0.0, 0.0, 0.0, 1.0);
        let ol = bg.clone();
        let ul = bg.clone();

        Self {
            bg,
            fg,
            ol,
            ul,
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
        let ol = RefCell::new(self.ol.clone());
        let ul = RefCell::new(self.ul.clone());

        // Stack for holding buttons
        let butts: RefCell<Vec<(u8, String)>> = RefCell::new(Vec::new());

        // Attributes
        let oline = RefCell::new(false); // overline
        let uline = RefCell::new(false); // underline

        // TODO: tmp
        let ol_size = 2.0;
        let ul_size = 2.0;

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
                ol: if *oline.borrow() { Some(ol.borrow().clone()) }
                    else { None },
                ul: if *uline.borrow() { Some(ul.borrow().clone()) }
                    else { None },
                ol_size,
                ul_size,
                cmd: butts.borrow().clone(),
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
                    ol: if *oline.borrow() { Some(ol.borrow().clone()) }
                        else { None },
                    ul: if *uline.borrow() { Some(ul.borrow().clone()) }
                        else { None },
                    ol_size,
                    ul_size,
                    cmd: butts.borrow().clone(),
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

            let t = *&caps["type"].chars().nth(0).unwrap();
            match t {
                'l'|'c'|'r' => {
                    let n = match t {
                        'l' => 0,
                        'c' => 1,
                        'r' => 2,
                        _ => 0,
                    };

                    if i != n {
                        pushf(&mut v[i]);
                        i = n;
                    }
                }

                'R' => {
                    swapc();
                }

                'F'|'B'|'U'|'u' => {
                    let def;
                    let mut c = match t {
                        'F' => { def = &self.fg; fg.borrow_mut() }
                        'B' => { def = &self.bg; bg.borrow_mut() }
                        'U' => { def = &self.ul; ul.borrow_mut() }
                        'u' => { def = &self.ol; ol.borrow_mut() }
                        _   => { panic!("") /* PLACEHOLDER */ }
                    };

                    if &caps["colo"] == "-" {
                        *c = def.clone();
                    } else {
                        *c = Color::from_hex(&caps["colo"]).unwrap();
                    }
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

                'A' => {
                    // This is to differentiate between
                    // %{A} and %{Abut:cmd}. If caps is Some
                    // then there is a command
                    match caps.name("cmd") {

                        // Push button on to the stack
                        Some(_) => {
                            // If no button is specified, chose 1
                            let b = match caps.name("butt") {
                                None    => 1,
                                Some(s) => u8::from_str(s.as_str()).unwrap(),
                            };
                            // The command itself
                            let c = String::from(&caps["cmd"]);
                            butts.borrow_mut().push((b, c));
                        }

                        // Pop off a button
                        None => {
                            if let None = butts.borrow_mut().pop() {
                                eprintln!("Unassociated %{{A}}!");
                            }
                        }
                    }
                }

                '!'|'+'|'-' => {
                    let mut a = match &caps["attr"] {
                        "o" => { oline.borrow_mut() }
                        "u" => { uline.borrow_mut() }
                        _   => { panic!("") }
                    };

                    *a = match t {
                        '!' => { ! *a  }
                        '+' => { true  }
                        '-' => { false }
                        _   => { panic!("") }
                    };
                }

                _ => {}
            }

            bpos = mat.end();
        }

        if bpos > epos {
            epos = fmt.len();
        }

        // TODO: remove - 1 for newline
        pusht(&mut v[i], &fmt[bpos..epos - 1]);

        let mut r: Vec<FormatItem> = Vec::new();
        for i in 0..3 {
            r.extend_from_slice(&v[i]);
        }
        return r;
    }
}
