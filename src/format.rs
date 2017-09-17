pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl Color {
    pub fn new(r: f64, g: f64, b: f64, a: f64) -> Color {
        Color { r, g, b, a, }
    }
}

pub struct Text {
    pub bg: Color,
    pub fg: Color,
    pub text: String,
}

pub enum FormatItem {
    Text(Text),
}

pub struct Format {
    pub left: Vec<FormatItem>,
    pub center: Vec<FormatItem>,
    pub right: Vec<FormatItem>,
}

impl Format {
    pub fn new() -> Format {
        Format {
            left: Vec::new(),
            center: Vec::new(),
            right: Vec::new(),
        }
    }
}
