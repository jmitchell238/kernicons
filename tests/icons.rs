//! Integration tests: the `icons/` folder and the embedded table agree, and everything renders.
use kernicons::{names, render_rgba, svg, variants, Mode};
use std::collections::BTreeSet;

fn files() -> Vec<String> {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("icons");
    let mut v: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    v.sort();
    v
}

#[test]
fn every_icon_parses_and_renders_at_16_20_24() {
    let all: Vec<_> = names().collect();
    assert!(all.len() > 100);
    for name in all {
        for px in [16, 20, 24] {
            let img = render_rgba(name, px, Mode::Colour)
                .unwrap_or_else(|| panic!("{name} @{px} failed"));
            assert!(
                img.rgba.chunks(4).any(|p| p[3] > 0),
                "{name} @{px} is blank"
            );
        }
    }
}

#[test]
fn every_size_variant_has_a_base_icon_and_a_matching_view_box() {
    let known: BTreeSet<_> = names().collect();
    for f in files() {
        let stem = f
            .strip_suffix(".svg")
            .unwrap_or_else(|| panic!("{f} is not an .svg"));
        if let Some((base, size)) = stem.split_once('@') {
            assert!(known.contains(base), "{f} has no base icon {base}.svg");
            let size: u32 = size
                .parse()
                .unwrap_or_else(|_| panic!("{f}: bad size suffix"));
            let (_, src) = variants(base)
                .find(|v| v.0 == size)
                .unwrap_or_else(|| panic!("{f} missing from table"));
            assert!(
                src.contains(&format!("viewBox=\"0 0 {size} {size}\"")),
                "{f} viewBox mismatch"
            );
        } else {
            assert!(known.contains(stem), "{f} missing from table");
            assert!(
                svg(stem).unwrap().contains("viewBox=\"0 0 16 16\""),
                "{f} must be a 16x16 design"
            );
        }
    }
}

#[test]
fn every_embedded_icon_has_a_file_and_names_are_kebab_case() {
    let files: BTreeSet<_> = files().into_iter().collect();
    for name in names() {
        assert!(files.contains(&format!("{name}.svg")), "{name}.svg missing");
        assert!(
            name.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
                && !name.starts_with('-'),
            "{name} is not lowercase kebab-case"
        );
    }
    let n_variants: usize = names().map(|n| variants(n).count()).sum();
    assert_eq!(
        n_variants + names().count(),
        files.len(),
        "every file must be in the table"
    );
}

#[test]
fn icons_have_no_text_scripts_or_external_references() {
    for f in files() {
        let src = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("icons")
                .join(&f),
        )
        .unwrap();
        for bad in [
            "<text", "<image", "<script", "<style", "href=", "Gradient", "<filter",
        ] {
            assert!(!src.contains(bad), "{f} contains {bad}");
        }
    }
}
