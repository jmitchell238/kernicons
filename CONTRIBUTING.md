# Contributing to Kernicons

Thanks for helping. Contributions are accepted under the same terms as the crate: `MIT OR Apache-2.0`. Only submit drawings you made yourself (no traced or copied third-party artwork).

## Design rules

- **File name = icon name.** Lowercase kebab-case, e.g. `align-left.svg`, `font-color.svg`. Size variants are `<name>@<size>.svg`.
- **Base design:** `viewBox="0 0 16 16"`. Put coordinates on a 0.25 grid where possible.
- **Stroke widths:** 2 for primary strokes at 16 px, 1 for fine detail. The `@24` variants use proportionally thicker strokes (compare `bold.svg` with `bold@24.svg`).
- **Pixel snapping:** place strokes so an odd-width stroke is centred on `.5` and an even-width stroke on a whole number, so edges fall on pixel boundaries at the design size. Check the icon at 16, 20 and 24 px in the gallery (magnified), not just at 100 percent.
- **Size variants** (`@20`, `@24`, and larger only if the icon needs it) are redrawn, not scaled. Their `viewBox` must be `0 0 N N`.
- **Colour:** outline ink is `currentColor`. Other colours are literal `#rrggbb`. `Mode::Disabled` and `Mode::Mono` work by rewriting these, so do not use named colours, `rgb()`, or opacity tricks for colour.
- **Not allowed:** text, gradients, filters, images, scripts, styles or external references (a test enforces this).
- **Every variant needs a base file** (a test enforces this).

## Adding an icon

1. Add `icons/<name>.svg` (and optionally `@20`, `@24`).
2. Add the entries to `src/table.rs`: `ICONS` for the base, `VARIANTS` for variants. Both tables are sorted by name (then size).
3. Add the name to a category in `examples/make_gallery.rs` (it fails if an icon is in no category).
4. Run `cargo run --example make_gallery`, look at the new sheets, and add the name to `docs/GALLERY.md`.
5. Add a line to `CHANGELOG.md` under *Unreleased*.

## Before you open a pull request

```
cargo test
cargo test --no-default-features
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

## Versioning

Semver. New icons: minor. Removing or renaming an icon, or any API break: major.
