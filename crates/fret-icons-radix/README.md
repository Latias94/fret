This crate provides a small curated set of Radix Icons SVG assets embedded via `include_bytes!`.

Radix Icons are designed on a 15×15 grid, which is expected to scale to the requested icon size at render time.

## Attribution

The SVG assets in `assets/icons/*.svg` are derived from the upstream Radix Icons repository:

- Source: `repo-ref/icons/packages/radix-icons/icons/*`
- Upstream: `https://github.com/radix-ui/icons` @ `112af91ad275a63c3a29b0da2588342af74ef9bf`
- License: see `LICENSE.radix`

## Semantic IDs

- `ui.check` -> `check.svg`
- `ui.chevron.down` -> `chevron-down.svg`
- `ui.close` -> `cross-1.svg`
- `ui.search` -> `magnifying-glass.svg`
- `ui.settings` -> `gear.svg`
- `ui.play` -> `play.svg`

## Features

- `semantic-ui` (default): registers the semantic `ui.*` IDs for the icons listed above.

## Vendor IDs

This crate also registers a small vendor namespace (for convenience in app code):

- `radix.check` -> `check.svg`
- `radix.chevron-down` -> `chevron-down.svg`
- `radix.cross-1` -> `cross-1.svg`
- `radix.magnifying-glass` -> `magnifying-glass.svg`
- `radix.gear` -> `gear.svg`
- `radix.play` -> `play.svg`
