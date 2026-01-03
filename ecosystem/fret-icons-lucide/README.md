This crate provides a small curated set of Lucide SVG icons embedded via `rust-embed`.

## Attribution

The SVG assets in `assets/icons/*.svg` are derived from the upstream Lucide icon set:

- Source: `repo-ref/lucide/icons/*`
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

## Maintenance

- Update the curated list in `icon-list.txt`.
- Sync SVGs from upstream Lucide into `assets/icons`:
  - Windows: `pwsh tools/sync_icons.ps1 -Pack lucide -Clean`
  - macOS/Linux: `python3 tools/sync_icons.py --pack lucide --clean`
