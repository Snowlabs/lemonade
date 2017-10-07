pub use self::xcbwin::XCB;

use cairo;
mod xcbwin;

pub trait Dock {
    fn create_surface(&self) -> cairo::Surface;
    fn dock(&self);
    fn set_size(&mut self, u16, u16);
    fn flush(&self);
    fn click_cb<F>(&mut self, F)
        where F: Fn(i16, i16, u8) + Send + Sync + 'static;
}
