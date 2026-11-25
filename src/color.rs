//! The colour type used by [`crate::Mode`].
//!
//! With the `egui` feature this is `egui::Color32` itself, so egui code passes its own
//! colours straight in. Without it, a minimal stand-in with the same constructors is used.

#[cfg(feature = "egui")]
pub use egui::Color32 as Color;

/// An sRGBA colour (stand-in for `egui::Color32` when the `egui` feature is off).
#[cfg(not(feature = "egui"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color(pub [u8; 4]);

#[cfg(not(feature = "egui"))]
impl Color {
    /// Opaque black.
    pub const BLACK: Color = Color([0, 0, 0, 255]);
    /// Opaque white.
    pub const WHITE: Color = Color([255, 255, 255, 255]);
    /// An opaque colour from red, green and blue.
    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Color([r, g, b, 255])
    }
    /// Red channel.
    pub const fn r(&self) -> u8 {
        self.0[0]
    }
    /// Green channel.
    pub const fn g(&self) -> u8 {
        self.0[1]
    }
    /// Blue channel.
    pub const fn b(&self) -> u8 {
        self.0[2]
    }
    /// Alpha channel.
    pub const fn a(&self) -> u8 {
        self.0[3]
    }
}
