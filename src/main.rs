extern crate lemonade;
extern crate regex;
#[macro_use]
extern crate clap;

use std::cell::RefCell;
use std::io;
use std::str::FromStr;
use regex::Regex;
use lemonade::Bar;
use lemonade::format::{FormatItem, Text, Filler, Color};

fn main() {

    let args = clap_app!(lemonade =>
        (about: "lemonbar replacement with extra features")
        (@arg GEOMETRY: -g +takes_value "Set geometry. Format is WxH+x+y")
        (@arg bott: -b "Dock bar at the bottom")
        //(@arg FORCE: -d "Force docking on unsupported WMs")
        (@arg FONT: -f ... +takes_value #{1, 1} "Load a font")
        //(@arg CLICK: -a +takes_value "Number of clickable areas")
        // TODO: currently does not render for the first few seconds
        (@arg perm: -p "Don't exit after stdin stops")
        //(@arg NAME: -n +takes_value "Set window name")
        (@arg UL_SIZE: -u +takes_value "Underline width in pixels")
        (@arg BG_COLO: -B +takes_value "Set default background colour")
        (@arg FG_COLO: -F +takes_value "Set default foreground colour")
        (@arg OL_OLCO: -O +takes_value "Set default overline colour. Defaults to -U")
        (@arg UL_COLO: -U +takes_value "Set default underline colour")
    ).get_matches();


    // bar takes care of drawing the window.
    // lem handles the input.
    let mut bar = Bar::with_xcb();
    let mut lem = LemonParser::new();

    // Whether to exit when stdin ends
    let quit_on_input_end = ! args.is_present("perm");

    bar.bottom(args.is_present("bott"));

    // Set command-line arguments
    if let Some(s) = args.value_of("GEOMETRY") {
        bar.set_geometry(&s).unwrap();
    }

    if let Some(s) = args.values_of("FONT") {
        lem.font_list = s.map(|s| s.to_string()).collect();
    }

    if let Some(s) = args.value_of("UL_SIZE") {
        lem.ul_size = f64::from_str(s).unwrap(); // TODO: validate
        if let None = args.value_of("OL_SIZE") {
            lem.ol_size = f64::from_str(s).unwrap();
        }
    }

    if let Some(s) = args.value_of("BG_COLO") {
        lem.bg = Color::from_hex(&s).unwrap();
    }

    if let Some(s) = args.value_of("FG_COLO") {
        lem.fg = Color::from_hex(&s).unwrap();
    }

    if let Some(s) = args.value_of("OL_COLO") {
        lem.ol = Color::from_hex(&s).unwrap();
    }

    if let Some(s) = args.value_of("UL_COLO") {
        lem.ul = Color::from_hex(&s).unwrap();
    }

    // Loop for reading from stdin and variables
    let mut buf = String::new();
    loop {
        match io::stdin().read_line(&mut buf) {
            Ok(n) if n == 0 => {
                match quit_on_input_end {
                    true  => std::process::exit(0),
                    false => continue,
                }
            }
            Ok(_) => {}
            Err(_) => std::process::exit(1),
        }

        buf.pop(); // Remove newline
        bar.set_fmt(lem.parse(&buf));
        bar.draw();
        buf.clear();
    }
}

pub struct LemonParser {
    pub bg: Color,
    pub fg: Color,
    pub ol: Color,
    pub ul: Color,
    pub ol_size: f64,
    pub ul_size: f64,
    pub font_list: Vec<String>,
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

        let bg = Color::new(0.0, 0.0, 0.0, 0.0);
        let fg = Color::new(1.0, 1.0, 1.0, 1.0);
        let ol = bg.clone();
        let ul = bg.clone();
        let ol_size = 1.0;
        let ul_size = 1.0;
        let font_list = vec![String::new()];

        Self {
            bg,
            fg,
            ol,
            ul,
            ol_size,
            ul_size,
            font_list,
            re,
        }
    }

    pub fn parse(&mut self, fmt: &str) -> Vec<FormatItem> {
        // Temporary variables for computing string slices
        let mut bpos: usize = 0;
        let mut epos: usize = fmt.len();

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

        let ol_size = self.ol_size;
        let ul_size = self.ul_size;

        // List of fonts and the current font
        let font = RefCell::new(self.font_list[..].join(", "));

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
            // Since fg and bg are wrapped in a RefCell, mem::swap
            // cannot be used. This is an alternative.
            unsafe {
                std::ptr::swap(fg.as_ptr(), bg.as_ptr());
            }
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
                        *font.borrow_mut() = self.font_list[..].join(", ");
                    } else {

                        // 1-based indexing
                        let i = usize::from_str(&caps["index"]).unwrap() - 1;

                        if i > self.font_list.len() {
                            eprintln!("Font index {} is too high", i);
                                *font.borrow_mut() = self.font_list[..].join(", ");
                        } else {
                            *font.borrow_mut() = match self.font_list.get(i) {
                                Some(f) => f.clone(),
                                None    => String::new(),
                            }
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

        pusht(&mut v[i], &fmt[bpos..epos]);

        let mut r: Vec<FormatItem> = Vec::new();
        for i in 0..3 {
            r.extend_from_slice(&v[i]);
        }
        return r;
    }
}
