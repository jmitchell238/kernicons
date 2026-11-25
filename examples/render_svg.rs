//! KernPage maintenance helper (kept so KernPage can run it from its repo root). Renders the app icon SVGs in packaging/windows/icon:
//!   kernpage@N.svg (hand-hinted 16, 20, 24)  -> png-hinted/N.png
//!   kernpage.svg   (master, 1408 grid)  -> png-master/N.png for 32, 40, 48, 64, 256, 512, 1408
//! With three arguments (`<in.svg> <width_px> <out.png>`) it renders just that one SVG, e.g. the wordmark.
//! packaging/windows/icon/regenerate.ps1 runs this as one step of rebuilding the app icon.
use resvg::{tiny_skia, usvg};

fn render(svg: &std::path::Path, px: u32, out: &std::path::Path) {
    let text = std::fs::read_to_string(svg).unwrap();
    let tree = usvg::Tree::from_str(&text, &usvg::Options::default()).unwrap();
    let mut pm = tiny_skia::Pixmap::new(px, px).unwrap();
    let s = px as f32 / tree.size().width();
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(s, s),
        &mut pm.as_mut(),
    );
    pm.save_png(out).unwrap();
}

/// `render_svg <in.svg> <width_px> <out.png>` renders any one SVG at that width (height follows the aspect).
fn render_width(svg: &str, width: u32, out: &str) {
    let text = std::fs::read_to_string(svg).unwrap();
    let tree = usvg::Tree::from_str(&text, &usvg::Options::default()).unwrap();
    let s = width as f32 / tree.size().width();
    let h = (tree.size().height() * s).ceil() as u32;
    let mut pm = tiny_skia::Pixmap::new(width, h).unwrap();
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(s, s),
        &mut pm.as_mut(),
    );
    pm.save_png(out).unwrap();
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() == 3 {
        render_width(&args[0], args[1].parse().unwrap(), &args[2]);
        return;
    }
    let dir = std::path::Path::new("packaging/windows/icon");
    for sub in ["png-hinted", "png-master"] {
        std::fs::create_dir_all(dir.join(sub)).unwrap();
    }
    for px in [16u32, 20, 24] {
        render(
            &dir.join(format!("kernpage@{px}.svg")),
            px,
            &dir.join(format!("png-hinted/{px}.png")),
        );
    }
    for px in [32u32, 40, 48, 64, 256, 512, 1408] {
        render(
            &dir.join("kernpage.svg"),
            px,
            &dir.join(format!("png-master/{px}.png")),
        );
    }
}
