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

// Must be the last item of the vector!
pub struct Filler {
    pub bg: Color,
}

pub enum FormatItem {
    Text(Text),
    Filler(Filler),
}
