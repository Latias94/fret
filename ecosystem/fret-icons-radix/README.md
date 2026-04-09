This crate provides a Radix Icons SVG icon pack embedded via `rust-embed`.

Radix Icons are designed on a 15×15 grid, which is expected to scale to the requested icon size at render time.

## Integration surface

- `register_vendor_icons(...)` / `register_icons(...)` are the low-level registry hooks.
- `PACK_METADATA` / `PACK` keep the pack's provenance and default registration shape explicit.
  `VENDOR_PACK` is the vendor-only contract when bootstrap/manual assembly needs to separate
  vendor ids from semantic alias policy.
- Enable `app-integration` when you want explicit installer surfaces under
  `fret_icons_radix::app::install(...)` and
  `fret_icons_radix::advanced::install_with_ui_services(...)`.
- Component crates should stay on semantic `IconId` / `ui.*` ids; app/bootstrap code decides
  whether this pack owns those semantic aliases.
- Semantic `ui.*` aliases use first-writer-wins (`alias_if_missing(...)`), so later app installers
  can intentionally override a semantic icon without mutating `radix.*` vendor ids.
- If a reusable ecosystem dependency ships this pack plus package bundle assets, compose both on
  one installer/bundle surface so the app wires one named dependency.

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
- `app-integration`: enables `fret_icons_radix::app::install(...)` and
  `fret_icons_radix::advanced::install_with_ui_services(...)`.

## Pack metadata

- `PACK_METADATA.pack_id`: `fret-icons-radix`
- `PACK_METADATA.vendor_namespace`: `radix`
- `PACK_METADATA.import_model`: `Vendored`

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
