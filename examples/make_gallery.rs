//! Renders the gallery PNGs in `docs/images/` (run from the repo root):
//!
//!     cargo run --example make_gallery
//!
//! Output, each in a light and a dark version:
//!   * `overview-{light,dark}.png`      every icon at 24 px, 2x magnified (nearest neighbour)
//!   * `sheet-<category>-{light,dark}.png`  labelled sheets: each icon at 16, 20 and 24 px, 3x magnified
//!
//! Magnifying with nearest-neighbour shows the real device pixels, so you can judge crispness.
//! Labels use a system font (Segoe UI / DejaVu Sans / Arial, whichever exists).
use resvg::{tiny_skia, usvg};
use kernicons::{render_rgba, Color, Mode};
use std::sync::Arc;

const CATEGORIES: &[(&str, &str, &[&str])] = &[
    (
        "text",
        "Text formatting",
        &[
            "abc",
            "bold",
            "italic",
            "underline",
            "sub",
            "sup",
            "case",
            "clear-format",
            "font-color",
            "font-color-letter",
            "fonts",
            "grow-font",
            "shrink-font",
            "styles",
            "line-spacing",
            "spacing",
            "pilcrow",
            "text-box",
            "find",
            "replace",
            "hash",
        ],
    ),
    (
        "paragraph",
        "Paragraph and layout",
        &[
            "align-left",
            "align-center",
            "align-right",
            "align-justify",
            "align-objects",
            "bullets",
            "numbering",
            "indent-less",
            "indent-more",
            "columns",
            "margins",
            "orientation",
            "header",
            "footer",
            "guides",
            "wrap",
            "size",
            "object-size",
        ],
    ),
    (
        "objects",
        "Objects, tables and pages",
        &[
            "frame",
            "shapes",
            "picture",
            "blocks",
            "group",
            "ungroup",
            "rotate",
            "background",
            "shading",
            "palette",
            "brush",
            "table",
            "table-above",
            "table-below",
            "table-left",
            "table-right",
            "table-delete",
            "page",
            "pages",
            "single-page",
            "two-page",
            "create-page",
            "page-width",
            "per-sheet",
            "repeat-area",
            "sequence",
            "field",
        ],
    ),
    (
        "app",
        "Commands, files and navigation",
        &[
            "new", "folder", "save", "print", "send", "mail", "exit", "close", "copy", "cut",
            "paste", "undo", "redo", "rename", "trash", "select", "pointer", "move", "window",
            "zoom", "zoom-fit", "fit-page", "eye", "link", "bookmark", "gear", "calendar", "clock",
        ],
    ),
    (
        "arrows",
        "Arrows, status and misc",
        &[
            "arrow",
            "back",
            "forward",
            "backward",
            "chevron",
            "chevron-down",
            "check",
            "info",
            "question",
            "warning",
            "lightbulb",
            "star",
            "globe",
            "people",
        ],
    ),
];

struct Theme {
    name: &'static str,
    bg: (u8, u8, u8),
    card: (u8, u8, u8),
    text: &'static str,
    ink: Color,
}

const THEMES: [Theme; 2] = [
    Theme {
        name: "light",
        bg: (255, 255, 255),
        card: (244, 245, 247),
        text: "#3c4043",
        ink: Color::from_rgb(0x42, 0x42, 0x42),
    },
    Theme {
        name: "dark",
        bg: (30, 31, 34),
        card: (43, 45, 49),
        text: "#d4d6da",
        ink: Color::from_rgb(0xe6, 0xe6, 0xe6),
    },
];

fn fill(
    pm: &mut tiny_skia::Pixmap,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    rgb: (u8, u8, u8),
    radius: f32,
) {
    let mut paint = tiny_skia::Paint::default();
    paint.set_color_rgba8(rgb.0, rgb.1, rgb.2, 255);
    paint.anti_alias = true;
    let mut pb = tiny_skia::PathBuilder::new();
    let (r, x2, y2) = (radius, x + w, y + h);
    pb.move_to(x + r, y);
    pb.line_to(x2 - r, y);
    pb.quad_to(x2, y, x2, y + r);
    pb.line_to(x2, y2 - r);
    pb.quad_to(x2, y2, x2 - r, y2);
    pb.line_to(x + r, y2);
    pb.quad_to(x, y2, x, y2 - r);
    pb.line_to(x, y + r);
    pb.quad_to(x, y, x + r, y);
    pb.close();
    pm.fill_path(
        &pb.finish().unwrap(),
        &paint,
        tiny_skia::FillRule::Winding,
        tiny_skia::Transform::identity(),
        None,
    );
}

/// Draws icon `name` at `px` device pixels, magnified `zoom` times with nearest neighbour, at (x, y).
fn blit(pm: &mut tiny_skia::Pixmap, name: &str, px: u32, zoom: u32, ink: Color, x: i32, y: i32) {
    let img =
        render_rgba(name, px, Mode::ColourInk(ink)).unwrap_or_else(|| panic!("no icon {name}"));
    let mut small = tiny_skia::Pixmap::new(px, px).unwrap();
    for (dst, s) in small.pixels_mut().iter_mut().zip(img.rgba.chunks(4)) {
        *dst = tiny_skia::ColorU8::from_rgba(s[0], s[1], s[2], s[3]).premultiply();
    }
    let paint = tiny_skia::PixmapPaint {
        quality: tiny_skia::FilterQuality::Nearest,
        ..Default::default()
    };
    let z = zoom as f32;
    pm.draw_pixmap(
        0,
        0,
        small.as_ref(),
        &paint,
        tiny_skia::Transform::from_row(z, 0.0, 0.0, z, x as f32, y as f32),
        None,
    );
}

fn options() -> usvg::Options<'static> {
    let mut opt = usvg::Options::default();
    Arc::make_mut(&mut opt.fontdb).load_system_fonts();
    opt
}

#[allow(clippy::too_many_arguments)]
fn label(
    pm: &mut tiny_skia::Pixmap,
    opt: &usvg::Options,
    text: &str,
    x: f32,
    y: f32,
    size: f32,
    weight: u32,
    colour: &str,
    anchor: &str,
) {
    let svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\"><text x=\"{x}\" y=\"{y}\" \
         font-family=\"Segoe UI, DejaVu Sans, Arial, sans-serif\" font-size=\"{size}\" font-weight=\"{weight}\" \
         fill=\"{colour}\" text-anchor=\"{anchor}\">{text}</text></svg>",
        pm.width(),
        pm.height()
    );
    let tree = usvg::Tree::from_str(&svg, opt).unwrap();
    resvg::render(&tree, tiny_skia::Transform::identity(), &mut pm.as_mut());
}

fn save(pm: &tiny_skia::Pixmap, name: &str) {
    let path = std::path::Path::new("docs/images").join(name);
    std::fs::create_dir_all("docs/images").unwrap();
    pm.save_png(&path).unwrap();
    println!("wrote {}", path.display());
}

fn sheet(opt: &usvg::Options, t: &Theme, id: &str, title: &str, names: &[&str]) {
    const COLS: usize = 4;
    const PAD: u32 = 24;
    const GAP: u32 = 12;
    const CW: u32 = 232;
    const CH: u32 = 136;
    const HEAD: u32 = 56;
    let rows = names.len().div_ceil(COLS) as u32;
    let w = PAD * 2 + COLS as u32 * CW + (COLS as u32 - 1) * GAP;
    let h = HEAD + PAD + rows * CH + (rows - 1) * GAP;
    let mut pm = tiny_skia::Pixmap::new(w, h).unwrap();
    fill(&mut pm, 0.0, 0.0, w as f32, h as f32, t.bg, 0.0);
    label(
        &mut pm, opt, title, PAD as f32, 38.0, 24.0, 600, t.text, "start",
    );
    label(
        &mut pm,
        opt,
        "16 / 20 / 24 px, magnified 3x",
        (w - PAD) as f32,
        38.0,
        13.0,
        400,
        t.text,
        "end",
    );
    for (i, name) in names.iter().enumerate() {
        let (col, row) = ((i % COLS) as u32, (i / COLS) as u32);
        let (cx, cy) = (PAD + col * (CW + GAP), HEAD + row * (CH + GAP));
        fill(
            &mut pm, cx as f32, cy as f32, CW as f32, CH as f32, t.card, 8.0,
        );
        // Three renders, bottom-aligned on one baseline, centred in the card.
        let total = 16 * 3 + 20 * 3 + 24 * 3 + 2 * 14;
        let mut x = cx + (CW - total) / 2;
        let base = cy + 14 + 72;
        for px in [16u32, 20, 24] {
            blit(
                &mut pm,
                name,
                px,
                3,
                t.ink,
                x as i32,
                (base - px * 3) as i32,
            );
            x += px * 3 + 14;
        }
        label(
            &mut pm,
            opt,
            name,
            (cx + CW / 2) as f32,
            (cy + CH - 16) as f32,
            14.0,
            500,
            t.text,
            "middle",
        );
    }
    save(&pm, &format!("sheet-{id}-{}.png", t.name));
}

fn overview(t: &Theme) {
    let mut all: Vec<&str> = kernicons::names().collect();
    all.sort_unstable();
    const COLS: usize = 18;
    const CELL: u32 = 64;
    const PAD: u32 = 20;
    let rows = all.len().div_ceil(COLS) as u32;
    let (w, h) = (PAD * 2 + COLS as u32 * CELL, PAD * 2 + rows * CELL);
    let mut pm = tiny_skia::Pixmap::new(w, h).unwrap();
    fill(&mut pm, 0.0, 0.0, w as f32, h as f32, t.bg, 0.0);
    for (i, name) in all.iter().enumerate() {
        let (col, row) = ((i % COLS) as u32, (i / COLS) as u32);
        // 24 px at 2x = 48 px, centred in a 64 px cell.
        blit(
            &mut pm,
            name,
            24,
            2,
            t.ink,
            (PAD + col * CELL + 8) as i32,
            (PAD + row * CELL + 8) as i32,
        );
    }
    save(&pm, &format!("overview-{}.png", t.name));
}

fn main() {
    let opt = options();
    let mut listed: Vec<&str> = CATEGORIES
        .iter()
        .flat_map(|c| c.2.iter().copied())
        .collect();
    listed.sort_unstable();
    let all: Vec<&str> = kernicons::names().collect();
    for n in &listed {
        assert!(all.contains(n), "gallery lists unknown icon {n}");
    }
    let missing: Vec<_> = all.iter().filter(|n| !listed.contains(n)).collect();
    assert!(
        missing.is_empty(),
        "icons not in any gallery category: {missing:?}"
    );
    for t in &THEMES {
        overview(t);
        for (id, title, names) in CATEGORIES {
            sheet(&opt, t, id, title, names);
        }
    }
}
