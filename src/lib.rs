//! Kernicons: the icon set from [KernPage](https://github.com/jmitchell238/kernpage).
//!
//! Every icon is an SVG drawn on a 16 px grid, embedded in the crate and drawn with
//! [resvg](https://docs.rs/resvg) at the exact device-pixel size you ask for. Icons use
//! `currentColor` for their ink, so they take any theme colour (see [`Mode`]).
//!
//! * Core (always available): [`names`], [`svg`], [`design_size`], [`render_rgba`], [`Mode`], [`Color`].
//! * With the default `egui` feature: [`paint`], [`rasterise`], [`rasterise_svg`] and [`INK`];
//!   `Color` is then `egui::Color32`.
//!
//! ```
//! use kernicons::{render_rgba, Mode};
//! let img = render_rgba("bold", 24, Mode::Colour).unwrap();
//! assert_eq!((img.width, img.height), (24, 24));
//! assert_eq!(img.rgba.len(), 24 * 24 * 4);
//! ```

mod color;
mod table;

pub use color::Color;

#[cfg(feature = "egui")]
use egui::{ColorImage, Context, Id, Painter, Pos2, Rect, TextureHandle, TextureOptions, Vec2};
#[cfg(feature = "egui")]
use std::collections::HashMap;
#[cfg(feature = "egui")]
use std::sync::{Arc, Mutex};

/// What `currentColor` renders as in normal colour mode (`#424242`).
pub const INK: Color = Color::from_rgb(0x42, 0x42, 0x42);

/// How to colour an icon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode {
    /// The palette colours as drawn.
    Colour,
    /// As `Colour`, but `currentColor` (the outline ink) is the given colour: lets the app
    /// draw its icons light on a dark background.
    ColourInk(Color),
    /// Every colour mapped to a flat grey ([`grey_of`]).
    Disabled,
    /// One colour: `currentColor` and every dark/mid palette colour become the tint; light
    /// fills (e.g. white paper) become fully transparent so outlines stay outlines.
    Mono(Color),
}

/// A rendered icon: straight (non-premultiplied) RGBA8, row-major, `width * height * 4` bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RgbaImage {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Straight-alpha RGBA bytes.
    pub rgba: Vec<u8>,
}

fn luminance(c: Color) -> f32 {
    (0.3 * c.r() as f32 + 0.59 * c.g() as f32 + 0.11 * c.b() as f32) / 255.0
}

/// The flat colour a palette colour takes when disabled: dark and mid colours become one
/// grey, light fills stay light so shapes keep their structure.
pub fn grey_of(c: Color) -> Color {
    let l = luminance(c);
    let v = if l < 0.7 {
        0xA6
    } else {
        (0xE4 as f32 + (l - 0.7) / 0.3 * 27.0) as u8
    };
    Color::from_rgb(v, v, v)
}

/// All icon names, sorted.
pub fn names() -> impl Iterator<Item = &'static str> {
    table::ICONS.iter().map(|(n, _)| *n)
}

/// The embedded 16 px SVG source for an icon (with `currentColor` unresolved).
pub fn svg(name: &str) -> Option<&'static str> {
    entry(name).map(|e| e.1)
}

/// The hand-tuned size variants of an icon as `(design size, SVG source)`, ascending.
/// Does not include the 16 px base design (see [`svg`]).
pub fn variants(name: &str) -> impl Iterator<Item = (u32, &'static str)> + '_ {
    table::VARIANTS
        .iter()
        .filter(move |v| v.0 == name)
        .map(|&(_, size, src)| (size, src))
}

fn entry(name: &str) -> Option<&'static (&'static str, &'static str)> {
    table::ICONS
        .binary_search_by(|(n, _)| (*n).cmp(name))
        .ok()
        .map(|i| &table::ICONS[i])
}

/// SVG text with `currentColor` and every `#rrggbb` replaced for `mode`.
fn recoloured(src: &str, mode: Mode) -> String {
    // Mono keeps `currentColor` until the end so the tint is never re-mapped as a palette colour.
    let src = match mode {
        Mode::Mono(_) => src.to_string(),
        Mode::ColourInk(ink) => src.replace("currentColor", &hex(ink)),
        _ => src.replace("currentColor", &hex(INK)),
    };
    if matches!(mode, Mode::Colour | Mode::ColourInk(_)) {
        return src;
    }
    let mut out = String::with_capacity(src.len());
    let b = src.as_bytes();
    let mut i = 0;
    while i < src.len() {
        if b[i] == b'#' && i + 7 <= src.len() && src.is_char_boundary(i + 7) {
            if let Ok(v) = u32::from_str_radix(&src[i + 1..i + 7], 16) {
                let c = Color::from_rgb((v >> 16) as u8, (v >> 8) as u8, v as u8);
                match mode {
                    Mode::Mono(t) if luminance(c) < 0.7 => out.push_str(&hex(t)),
                    Mode::Mono(t) => out.push_str(&format!("{}00", hex(t))),
                    _ => out.push_str(&hex(grey_of(c))),
                }
                i += 7;
                continue;
            }
        }
        let ch = src[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    if let Mode::Mono(t) = mode {
        out = out.replace("currentColor", &hex(t));
    }
    out
}

fn hex(c: Color) -> String {
    format!("#{:02x}{:02x}{:02x}", c.r(), c.g(), c.b())
}

/// The design (source SVG, design size) used to draw `name` at `px` device pixels.
///
/// Every icon has a 16 design (`<name>.svg`) and may have hand-tuned variants
/// (`<name>@20.svg`, `<name>@24.svg`, ...). Rule: if `px` is an exact integer multiple of one or
/// more variant sizes, the largest such variant is used (so 24 and 48 use @24, 20 and 40 use
/// @20, 32 and 16 use the 16 design); otherwise the largest variant whose size is <= `px`;
/// otherwise the 16 design.
fn design_for(name: &str, px: u32) -> Option<(&'static str, u32)> {
    let base = entry(name)?;
    let mut best: Option<(&'static str, u32)> = None;
    let mut consider = |svg: &'static str, size: u32| {
        let key = |s: u32| (px % s == 0, s);
        if size <= px.max(1) && best.is_none_or(|(_, b)| key(size) > key(b)) {
            best = Some((svg, size));
        }
    };
    consider(base.1, 16);
    for &(n, size, svg) in table::VARIANTS {
        if n == name {
            consider(svg, size);
        }
    }
    // Below 16 px the 16 design is the only one that fits.
    Some(best.unwrap_or((base.1, 16)))
}

/// The design size (16, 20, 24, ...) that [`render_rgba`] / [`paint`] use for `name` at `px`
/// device pixels. None if the name is unknown.
pub fn design_size(name: &str, px: u32) -> Option<u32> {
    design_for(name, px.max(1)).map(|d| d.1)
}

/// Premultiplied pixmap of an icon at exactly `px` x `px`.
fn pixmap(name: &str, px: u32, mode: Mode) -> Option<resvg::tiny_skia::Pixmap> {
    let px = px.max(1);
    let (src, design) = design_for(name, px)?;
    let text = recoloured(src, mode);
    let tree = resvg::usvg::Tree::from_str(&text, &resvg::usvg::Options::default()).ok()?;
    let mut pm = resvg::tiny_skia::Pixmap::new(px, px)?;
    let s = px as f32 / design as f32;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::from_scale(s, s),
        &mut pm.as_mut(),
    );
    Some(pm)
}

/// Renders an icon to exactly `px` x `px` straight-alpha RGBA8 pixels, picking the best size
/// variant. None if the name is unknown or the SVG is bad.
pub fn render_rgba(name: &str, px: u32, mode: Mode) -> Option<RgbaImage> {
    let pm = pixmap(name, px, mode)?;
    let mut rgba = Vec::with_capacity(pm.data().len());
    for p in pm.pixels() {
        let c = p.demultiply();
        rgba.extend_from_slice(&[c.red(), c.green(), c.blue(), c.alpha()]);
    }
    Some(RgbaImage {
        width: pm.width(),
        height: pm.height(),
        rgba,
    })
}

/// Rasterises an icon to exactly `px` x `px` pixels. None if the name is unknown or the SVG is bad.
#[cfg(feature = "egui")]
pub fn rasterise(name: &str, px: u32, mode: Mode) -> Option<ColorImage> {
    let pm = pixmap(name, px, mode)?;
    Some(ColorImage::from_rgba_premultiplied(
        [pm.width() as usize, pm.height() as usize],
        pm.data(),
    ))
}

/// Rasterises any standalone SVG text to `width_px` wide (height follows its aspect ratio), transparent
/// background. None if the SVG is bad. Used for the logo, whose static SVGs carry their own colours.
#[cfg(feature = "egui")]
pub fn rasterise_svg(svg: &str, width_px: u32) -> Option<ColorImage> {
    let width_px = width_px.max(1);
    let tree = resvg::usvg::Tree::from_str(svg, &resvg::usvg::Options::default()).ok()?;
    let s = width_px as f32 / tree.size().width();
    let h = ((tree.size().height() * s).ceil() as u32).max(1);
    let mut pm = resvg::tiny_skia::Pixmap::new(width_px, h)?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::from_scale(s, s),
        &mut pm.as_mut(),
    );
    Some(ColorImage::from_rgba_premultiplied(
        [width_px as usize, h as usize],
        pm.data(),
    ))
}

#[cfg(feature = "egui")]
type Cache = Arc<Mutex<HashMap<(&'static str, u32, Mode), TextureHandle>>>;

#[cfg(feature = "egui")]
fn texture(ctx: &Context, name: &'static str, px: u32, mode: Mode) -> Option<TextureHandle> {
    let cache: Cache = ctx.data_mut(|d| {
        d.get_temp_mut_or_insert_with::<Cache>(Id::unique("kernicons-cache"), Default::default)
            .clone()
    });
    let key = (name, px, mode);
    if let Some(t) = cache.lock().unwrap().get(&key) {
        return Some(t.clone());
    }
    let img = rasterise(name, px, mode)?;
    let t = ctx.load_texture(
        format!("pw-icon-{name}-{px}-{mode:?}"),
        img,
        TextureOptions::NEAREST,
    );
    cache.lock().unwrap().insert(key, t.clone());
    Some(t)
}

/// Paints icon `name` in the largest centred square of `rect`, at whole device pixels.
/// Returns false if the icon does not exist.
#[cfg(feature = "egui")]
pub fn paint(p: &Painter, rect: Rect, name: &str, mode: Mode) -> bool {
    let Some(&(name, _)) = entry(name) else {
        return false;
    };
    let ctx = p.ctx();
    let ppp = ctx.pixels_per_point();
    let side = rect.width().min(rect.height());
    let px = ((side * ppp).round() as u32).max(1);
    let o = rect.center() - Vec2::splat(side / 2.0);
    let min = Pos2::new((o.x * ppp).round() / ppp, (o.y * ppp).round() / ppp);
    let Some(tex) = texture(ctx, name, px, mode) else {
        return false;
    };
    let dst = Rect::from_min_size(min, Vec2::splat(px as f32 / ppp));
    p.image(
        tex.id(),
        dst,
        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
        Color::WHITE,
    );
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_is_sorted_and_every_icon_renders() {
        let n: Vec<_> = names().collect();
        assert!(
            n.windows(2).all(|w| w[0] < w[1]),
            "table must be sorted and unique"
        );
        for name in n {
            for px in [16, 20, 24, 32] {
                for mode in [Mode::Colour, Mode::Disabled] {
                    let img = render_rgba(name, px, mode).unwrap_or_else(|| panic!("{name} fails"));
                    assert_eq!((img.width, img.height), (px, px));
                    assert!(
                        img.rgba.chunks(4).any(|p| p[3] > 0),
                        "{name} @{px} is empty"
                    );
                }
            }
        }
    }

    #[test]
    fn disabled_has_no_saturated_pixels() {
        for name in names() {
            let img = render_rgba(name, 32, Mode::Disabled).unwrap();
            for p in img.rgba.chunks(4) {
                let (r, g, b) = (p[0] as i32, p[1] as i32, p[2] as i32);
                assert!(
                    (r - g).abs() <= 2 && (g - b).abs() <= 2,
                    "{name} has colour {p:?}"
                );
            }
        }
    }

    fn ds(name: &str, px: u32) -> u32 {
        design_for(name, px).unwrap().1
    }

    #[test]
    fn variant_rule_picks_expected_design() {
        // `numbering` has @20 and @24 variants.
        assert_eq!(ds("numbering", 16), 16);
        assert_eq!(ds("numbering", 20), 20);
        assert_eq!(ds("numbering", 24), 24);
        assert_eq!(ds("numbering", 32), 16); // 32 is a multiple of 16 only
        assert_eq!(ds("numbering", 48), 24);
        assert_eq!(ds("numbering", 22), 20); // no multiple: largest <= px
        assert_eq!(ds("numbering", 12), 16); // smaller than all: the 16 design
                                             // `trash` has @20 and @24 only (no large variants): 28 is no multiple, so the largest <= px.
        assert_eq!(ds("trash", 24), 24);
        assert_eq!(ds("trash", 28), 24);
        assert_eq!(design_size("trash", 28), Some(24));
        assert_eq!(design_size("no-such-icon", 28), None);
        assert_eq!(
            svg("numbering"),
            Some(table::ICONS.iter().find(|e| e.0 == "numbering").unwrap().1)
        );
    }

    #[test]
    fn variants_are_valid() {
        let n: Vec<_> = names().collect();
        assert!(n.windows(2).all(|w| w[0] < w[1]), "names unique and sorted");
        assert!(table::VARIANTS
            .windows(2)
            .all(|w| (w[0].0, w[0].1) < (w[1].0, w[1].1)));
        for &(name, size, src) in table::VARIANTS {
            assert!(
                entry(name).is_some(),
                "variant {name}@{size} has no base icon"
            );
            assert!(size != 16, "{name}@{size}: the 16 design is the base file");
            assert!(
                src.contains(&format!("viewBox=\"0 0 {size} {size}\"")),
                "{name}@{size} viewBox mismatch"
            );
        }
        assert_eq!(
            variants("numbering").map(|v| v.0).collect::<Vec<_>>(),
            [20, 24]
        );
    }

    #[test]
    fn mono_uses_only_the_tint_hue() {
        for t in [Color::from_rgb(0x12, 0xa0, 0x56), Color::WHITE] {
            for name in names() {
                for px in [21, 32] {
                    let img = render_rgba(name, px, Mode::Mono(t)).unwrap();
                    assert!(
                        img.rgba.chunks(4).any(|p| p[3] > 100),
                        "{name} @{px} has no solid tint pixels"
                    );
                    for p in img.rgba.chunks(4) {
                        if p[3] < 8 {
                            continue;
                        }
                        let near = |a: u8, b: u8| (a as i32 - b as i32).abs() <= 14;
                        assert!(
                            near(p[0], t.r()) && near(p[1], t.g()) && near(p[2], t.b()),
                            "{name} has non-tint pixel {p:?}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn mono_light_fill_is_transparent_and_outline_survives() {
        // `page` has a white paper fill: its interior must be fully transparent, the outline solid.
        for t in [Color::BLACK, Color::WHITE] {
            let img = render_rgba("page", 32, Mode::Mono(t)).unwrap();
            assert_eq!(
                img.rgba[(16 * 32 + 12) * 4 + 3],
                0,
                "paper interior must be transparent for {t:?}"
            );
            assert!(
                img.rgba.chunks(4).filter(|p| p[3] > 200).count() > 40,
                "outline missing for {t:?}"
            );
        }
    }

    #[test]
    fn colour_ink_changes_the_outline() {
        let red = Color::from_rgb(200, 0, 0);
        let img = render_rgba("bold", 32, Mode::ColourInk(red)).unwrap();
        assert!(img
            .rgba
            .chunks(4)
            .any(|p| p[3] == 255 && p[0] == 200 && p[1] == 0));
    }

    #[test]
    fn unknown_name_is_none() {
        assert!(svg("no-such-icon").is_none());
        assert!(render_rgba("no-such-icon", 16, Mode::Colour).is_none());
        #[cfg(feature = "egui")]
        assert!(rasterise("no-such-icon", 16, Mode::Colour).is_none());
    }
}
