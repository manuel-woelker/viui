use crate::result::ViuiResult;
use rgb::RGBA8;

pub struct ScreenSpace;
pub type Float = f32;
pub type Rect = euclid::Rect<Float, ScreenSpace>;
pub type Point = euclid::Point2D<Float, ScreenSpace>;
pub type Size = euclid::Size2D<Float, ScreenSpace>;

#[derive(Debug, Copy, Clone)]
pub struct Color {
    pub rgba: RGBA8,
}

impl Color {
    pub const BLACK: Self = Self::rgb(0, 0, 0);
    pub const WHITE: Self = Self::rgb(255, 255, 255);
    pub const GRAY: Self = Self::rgb(235, 235, 235);

    pub fn rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            rgba: RGBA8::new(red, green, blue, alpha),
        }
    }
    pub const fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self {
            rgba: RGBA8::new(red, green, blue, 255),
        }
    }

    pub fn gray(whiteness: u8) -> Self {
        Self {
            rgba: RGBA8::new(whiteness, whiteness, whiteness, 255),
        }
    }

    /// Constructs a `Color` from hue, saturation, and lightness (HSL) values.
    ///
    /// # Arguments
    ///
    /// * `hue` - A `Float` representing the hue angle in degrees (0 to 360).
    /// * `saturation` - A `Float` representing the saturation percentage (0 to 1).
    /// * `lightness` - A `Float` representing the lightness percentage (0 to 1).
    ///
    /// # Returns
    ///
    /// A `Color` instance representing the color in RGBA format.
    pub fn hsl(hue: Float, saturation: Float, lightness: Float) -> ViuiResult<Self> {
        let hsl = coolor::Hsl::new(hue, saturation, lightness);
        let coolor::Rgb { r, g, b } = hsl.to_rgb();
        Ok(Self::rgb(r, g, b))
    }
}
