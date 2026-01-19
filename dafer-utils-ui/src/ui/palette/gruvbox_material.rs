/// Gruvbox Material color palette for use in egui UI, with RGBA (opacity) support.
///
/// Provides a set of associated functions for foreground/background and accent colors.
/// All colors are in sRGB hex and mapped to egui's Color32, using RGBA for opacity control.
///
/// Reference: https://github.com/sainnhe/gruvbox-material
use eframe::egui::Color32;

pub struct GruvboxMaterial;

impl GruvboxMaterial {
    // Backgrounds (opaque by default, alpha=255)
    pub fn bg(alpha: u8) -> Color32 {
        Color32::from_rgba_unmultiplied(40, 40, 40, alpha)
    } // #282828
    pub fn bg1(alpha: u8) -> Color32 {
        Color32::from_rgba_unmultiplied(60, 56, 54, alpha)
    } // #3c3836
    pub fn bg2(alpha: u8) -> Color32 {
        Color32::from_rgba_unmultiplied(80, 73, 69, alpha)
    } // #504945
    pub fn bg3(alpha: u8) -> Color32 {
        Color32::from_rgba_unmultiplied(102, 92, 84, alpha)
    } // #665c54
    pub fn bg4(alpha: u8) -> Color32 {
        Color32::from_rgba_unmultiplied(124, 111, 100, alpha)
    } // #7c6f64

    // Foregrounds (opaque by default, alpha=255)
    pub fn fg(alpha: u8) -> Color32 {
        Color32::from_rgba_unmultiplied(235, 219, 178, alpha)
    } // #ebdbb2
    pub fn fg1(alpha: u8) -> Color32 {
        Color32::from_rgba_unmultiplied(213, 196, 161, alpha)
    } // #d5c4a1
    pub fn fg2(alpha: u8) -> Color32 {
        Color32::from_rgba_unmultiplied(189, 174, 147, alpha)
    } // #bdae93
    pub fn fg3(alpha: u8) -> Color32 {
        Color32::from_rgba_unmultiplied(168, 153, 132, alpha)
    } // #a89984

    // Primary accent colors (opaque by default, alpha=255)
    pub fn red(alpha: u8) -> Color32 {
        Color32::from_rgba_unmultiplied(251, 73, 52, alpha)
    } // #fb4934
    pub fn orange(alpha: u8) -> Color32 {
        Color32::from_rgba_unmultiplied(254, 128, 25, alpha)
    } // #fe8019
    pub fn yellow(alpha: u8) -> Color32 {
        Color32::from_rgba_unmultiplied(250, 189, 47, alpha)
    } // #fabd2f
    pub fn green(alpha: u8) -> Color32 {
        Color32::from_rgba_unmultiplied(184, 187, 38, alpha)
    } // #b8bb26
    pub fn aqua(alpha: u8) -> Color32 {
        Color32::from_rgba_unmultiplied(131, 165, 152, alpha)
    } // #83a598
    pub fn blue(alpha: u8) -> Color32 {
        Color32::from_rgba_unmultiplied(131, 155, 255, alpha)
    } // #83adff (custom, original: #83a598)
    pub fn purple(alpha: u8) -> Color32 {
        Color32::from_rgba_unmultiplied(211, 134, 155, alpha)
    } // #d3869b

    // Additional accent colors
    pub fn gray(alpha: u8) -> Color32 {
        Color32::from_rgba_unmultiplied(146, 131, 116, alpha)
    } // #928374

    // Example: Semi-transparent overlays (alpha < 255)
    pub fn overlay_bg() -> Color32 {
        Color32::from_rgba_unmultiplied(40, 40, 40, 200)
    } // 78% opacity
    pub fn overlay_fg() -> Color32 {
        Color32::from_rgba_unmultiplied(235, 219, 178, 200)
    } // 78% opacity

    // Utility: get all as a palette (opaque only)
    pub fn palette() -> [Color32; 15] {
        [
            Self::bg(255),
            Self::bg1(255),
            Self::bg2(255),
            Self::bg3(255),
            Self::bg4(255),
            Self::fg(255),
            Self::fg1(255),
            Self::fg2(255),
            Self::fg3(255),
            Self::red(255),
            Self::orange(255),
            Self::yellow(255),
            Self::green(255),
            Self::aqua(255),
            Self::purple(255),
        ]
    }
}
