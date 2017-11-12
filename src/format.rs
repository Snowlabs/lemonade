#[cfg(feature = "image")]
use gdk_pixbuf::Pixbuf;

// Fucking fight me
pub type Color = Colour;

#[derive(Clone)]
pub struct Colour {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl Colour {
    pub fn new(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self { r, g, b, a, }
    }

    // Takes either: #rrggbb, #aarrggbb, #rbg, #argb
    // The '#' in front is not necessary
    pub fn from_hex(hex: &str) -> Result<Self, &'static str>{
        let mut s = String::from(hex);
        let mut c = [1.0, 0.0, 0.0, 0.0]; // argb

        // Remove the prepended #
        if s.get(0..1) == Some("#") {
            s.remove(0);
        }

        // Make sure the length is ok, and set the multiplier
        // used when iterating.
        let m: usize;
        let b: usize;
        match s.len() {
            3 => { m = 1; b = 1 }
            4 => { m = 1; b = 0 }
            6 => { m = 2; b = 1 }
            8 => { m = 2; b = 0 }
            _ => return Err("Provided string as invalid length"),
        }

        let mut it = s.chars().peekable();
        let mut n = 0;
        while it.peek().is_some() {
            let h: String = it.by_ref().take(m).collect();
            let v = i32::from_str_radix(&h, 16).unwrap();
            c[b + n] = v as f64 / (16.0_f64.powi(m as i32) - 1.0);
            n += 1;
        }

        Ok(Self {
            a: c[0],
            r: c[1],
            g: c[2],
            b: c[3],
        })
    }
}

#[derive(Clone)]
pub struct BG {
    pub bg: Colour,
    pub ol: Option<Colour>,
    pub ul: Option<Colour>,
    pub ol_size: f64,
    pub ul_size: f64,
    pub cmd: Vec<(u8, String)>,
}

#[derive(Clone)]
pub struct Text {
    pub fg: Colour,
    pub text: String,
    pub font: String,
}

#[cfg(feature = "image")]
#[derive(Clone)]
pub struct Image {
    pub path: String,
    pub width: i32,
    pub height: i32,
    img: Pixbuf,
}

//#[cfg(feature = "image")]
//static ref IMG_CACHE = {
    //let m: HashMap<String, Pixbuf> = HashMap::new();
    //RwLock::new(m)
//}

#[cfg(feature = "image")]
impl Image {

    /// Create `Image` from path, with width and height.
    ///
    /// If the width is negative, then the width is automatically adjusted
    /// according to height to preserver aspect ratio, and vice versa for
    /// the height.
    ///
    /// If the file fails to load, an error is printed to stderr and an
    /// emtpy image with a size of 0 is returned.
    pub fn from_file(path: &str, w: i32, h: i32) -> Result<Self, ()> {
        let img = match Pixbuf::new_from_file_at_size(path, w, h) {
            Ok(img) => img,
            Err(e)  => {
                eprintln!("{}", e);
                return Err(());
            }
        };

        let path   = String::from(path);
        let width  = img.get_width();
        let height = img.get_height();

        Ok(Self {
            path,
            width,
            height,
            img,
        })
    }

    pub fn pixbuf(&self) -> &Pixbuf {
        &self.img
    }
}

#[derive(Clone)]
pub enum FormatItem {
    Text(Text, BG),
    Filler(BG),

    #[cfg(feature = "image")]
    Image(Image, BG),
}
