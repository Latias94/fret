This crate provides a Lucide SVG icon pack embedded via `rust-embed`.

## Attribution

The SVG assets in `assets/icons/*.svg` are derived from the upstream Lucide icon set:

- Source: `third_party/lucide/icons/*` (git submodule)
- Upstream: `https://github.com/lucide-icons/lucide` @ `d391bda369305b98a43a812ae2ff8955455dcd5d`
- License: see `LICENSE.lucide`

## Semantic IDs

- `ui.check` -> `check.svg`
- `ui.chevron.down` -> `chevron-down.svg`
- `ui.close` -> `x.svg`
- `ui.search` -> `search.svg`
- `ui.settings` -> `settings.svg`
- `ui.play` -> `play.svg`

## Features

- `semantic-ui` (default): registers the semantic `ui.*` ID aliases listed above.

## Vendor IDs

This crate registers `lucide.<icon-name>` for every SVG listed in `icon-list.txt` (where `<icon-name>` matches the
upstream SVG filename stem).

Generated vendor constants are exposed under `generated_ids::lucide::*`.

## Maintenance

- Generate full Lucide list and Rust constants:
  - Windows/macOS/Linux: `python3 tools/gen_lucide.py`
- Generate one/all packs with a single entrypoint:
  - Windows/macOS/Linux: `python3 tools/gen_icons.py --pack lucide`
  - Windows/macOS/Linux: `python3 tools/gen_icons.py --pack all`
- Sync SVGs from upstream sources into `assets/icons`:
  - `python3 tools/sync_icons.py --pack lucide --clean`
  - macOS/Linux: `python3 tools/sync_icons.py --pack lucide --clean`
- Verify referenced vendor IDs resolve to vendored assets:
  - Windows/macOS/Linux: `python3 tools/verify_icons.py --strict`
- Release-time one-shot checks:
  - Icons only: `python3 tools/pre_release_icons.py`
  - Aggregate entrypoint: `python3 tools/pre_release.py`
  - Pack-aware check entrypoint: `python3 tools/check_icons_generation.py --pack lucide`
