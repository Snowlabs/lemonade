extern crate lemonade;
extern crate regex;
#[macro_use]
extern crate clap;

mod lemon;
use lemon::LemonParser;

use std::io;
use std::str::FromStr;
use lemonade::Bar;
use lemonade::format::Color;

fn main() {

    let args = clap_app!(lemonade =>
        (about: "lemonbar replacement with extra features")
        (@arg GEOMETRY: -g +takes_value "Set geometry. Format is WxH+x+y")
        (@arg bott: -b "Dock bar at the bottom")
        //(@arg FORCE: -d "Force docking on unsupported WMs")
        (@arg FONT: -f +takes_value +multiple "Load a font")
        //(@arg CLICK: -a +takes_value "Number of clickable areas")
        // TODO: currently does not render for the first few seconds
        (@arg perm: -p "Don't exit after stdin stops")
        //(@arg NAME: -n +takes_value "Set window name")
        (@arg UL_SIZE: -u +takes_value "Underline width in pixels")
        (@arg BG_COLO: -B +takes_value {is_colo} "Set default background colour")
        (@arg FG_COLO: -F +takes_value {is_colo} "Set default foreground colour")
        (@arg UL_COLO: -U +takes_value {is_colo} "Set default underline colour")
        (@arg OL_OLCO: -O +takes_value {is_colo} "Set default overline colour. \
                                                  Defaults to -U")
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

fn is_colo(s: String) -> Result<(), String> {
    if s.is_empty() {
        return Err("The colour string must not be empty".to_string());
    }

    if s.chars().nth(0).unwrap() != '#' {
        return Err("Format must be either: \
                    #rgb, #argb, #rrggbb, #aarrggbb".to_string());
    }

    match s.len() - 1 {
        3|4|6|8 => {}
        _ => return Err("Invalid number of digits".to_string()),
    }

    if let Some(_) = s.find(|s| {
        match s {
            'a'...'f'|'A'...'F'|'0'...'9' => false,
            _ => true,
        }
    }) { return Err("Digits must be hex".to_string()); }

    Ok(())
}
