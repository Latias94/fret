This crate provides a Radix Icons SVG icon pack embedded via `rust-embed`.

Radix Icons are designed on a 15×15 grid, which is expected to scale to the requested icon size at render time.

## Attribution

The SVG assets in `assets/icons/*.svg` are derived from the upstream Radix Icons repository:

- Source: `third_party/radix-icons/packages/radix-icons/icons/*` (git submodule)
- Upstream: `https://github.com/radix-ui/icons`
- License: see `LICENSE.radix`

## Semantic IDs

- `ui.check` -> `check.svg`
- `ui.chevron.down` -> `chevron-down.svg`
- `ui.chevron.up` -> `chevron-up.svg`
- `ui.close` -> `cross-1.svg`
- `ui.search` -> `magnifying-glass.svg`
- `ui.settings` -> `gear.svg`
- `ui.play` -> `play.svg`

## Features

- `semantic-ui` (default): registers the semantic `ui.*` IDs for the icons listed above.

## Vendor IDs

This crate registers `radix.<icon-name>` for every SVG listed in `icon-list.txt` (where `<icon-name>` matches the
upstream SVG filename stem).

Generated vendor constants are exposed under `generated_ids::radix::*`.

## Maintenance

- Generate full Radix list and Rust constants:
  - Windows/macOS/Linux: `python3 tools/gen_radix.py`
- Generate one/all packs with a single entrypoint:
  - Windows/macOS/Linux: `python3 tools/gen_icons.py --pack radix`
  - Windows/macOS/Linux: `python3 tools/gen_icons.py --pack all`
- Sync SVGs from upstream sources into `assets/icons`:
  - `python3 tools/sync_icons.py --pack radix --clean`
  - macOS/Linux: `python3 tools/sync_icons.py --pack radix --clean`
- Verify referenced vendor IDs resolve to vendored assets:
  - Windows/macOS/Linux: `python3 tools/verify_icons.py --strict`
- Release-time one-shot checks:
  - Icons only: `python3 tools/pre_release_icons.py`
  - Aggregate entrypoint: `python3 tools/pre_release.py`
  - Pack-aware check entrypoint: `python3 tools/check_icons_generation.py --pack radix`
