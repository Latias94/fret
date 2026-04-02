# Assets

Workspace-level demo and documentation assets.

Keep files here when they are shared across multiple demos, examples, or docs pages and do not
belong to a single crate.

Current contents:

- `assets/fret-icon.svg`: repository/app icon used by root docs and branding surfaces
- `assets/demo/`: small shared demo assets
- `assets/textures/`: shared raster fixtures used by cookbook examples, UI Gallery, and asset-loading
  tests

Do not move crate-owned assets here:

- fonts stay under `crates/fret-fonts/assets/`
- design-system/theme assets stay under their owning ecosystem crate
- generated app assets belong in the generated app's own `assets/` directory
