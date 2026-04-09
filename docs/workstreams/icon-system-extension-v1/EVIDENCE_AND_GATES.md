# Icon System Extension v1 — Evidence And Gates

Status: Closed closeout lane (contract shipped; follow-on only)
Last updated: 2026-04-09

Closeout note on 2026-04-09:

- `CLOSEOUT_AUDIT_2026-04-09.md` closes this lane on the shipped v1 icon contract / runtime /
  pack protocol goal.
- Read the historical baseline commands below as supporting evidence, not as an active execution
  queue.

## Smallest closeout repro

Use the focused closeout validation set that covers the shipped registry contract, pack metadata
recording, docs guidance, and multicolor runtime split:

```bash
cargo nextest run -p fret-icons -p fret-icons-lucide -p fret-icons-radix
cargo test -p fret-icons-lucide --features app-integration app_install_records_pack_metadata_and_freezes_registry
cargo test -p fret-icons-radix --features app-integration app_install_records_pack_metadata_and_freezes_registry
cargo nextest run -p fret usage_docs_prefer_explicit_app_submodules_for_optional_ecosystems todo_golden_path_keeps_icon_pack_setup_on_app_install_surface
cargo nextest run -p fret-ui svg_image_props_paint_to_svg_image_scene_op foreground_scope_late_binds_foreground_for_text_and_icons inherited_foreground_on_existing_root_late_binds_for_text_icon_and_spinner
cargo nextest run -p fret-ui-kit
cargo nextest run -p fret-bootstrap
python3 tools/check_layering.py
```

## Historical baseline demo

Use the existing cookbook icon/assets demo as the first-open proof surface before changing the icon
contract:

```bash
cargo run -p fret-cookbook --features cookbook-assets --example icons_and_assets_basics
```

Why this is the first repro:

- it exercises semantic ids, vendor ids, SVG assets, icon preloading diagnostics, and package-owned
  assets in one small app-facing surface,
- and it already sits on the current public story (`IconId`, `SvgSource`, `SvgIconProps`,
  package-owned assets).

## Historical baseline inspection commands

Use these before editing:

```bash
cargo nextest run -p fret-icons
cargo nextest run -p fret-ui
cargo nextest run -p fret-ui-kit
cargo nextest run -p fret-bootstrap
python3 tools/check_layering.py
```

## Runtime slice verification commands

These are the commands that currently prove the landed runtime slice:

```bash
cargo nextest run -p fret-ui svg_image_props_paint_to_svg_image_scene_op foreground_scope_late_binds_foreground_for_text_and_icons inherited_foreground_on_existing_root_late_binds_for_text_icon_and_spinner
cargo nextest run -p fret-ui-kit
cargo check -p fret-ui-material3
cargo nextest run -p fret-bootstrap
python3 tools/check_layering.py
```

Note:

- `cargo nextest run -p fret-ui` still fails in this workspace on
  `declarative::tests::layout::basics::pressable_with_auto_width_chrome_container_shrink_wraps_in_grid_auto_track`.
  That failure does not touch the icon/SVG codepaths introduced by this lane and should be treated
  as an external gate blocker until the owning layout lane resolves it.

## M3 closeout verification commands

These are the commands that close the pack protocol / teaching-surface slice:

```bash
cargo nextest run -p fret-icons -p fret-icons-lucide -p fret-icons-radix
cargo test -p fret-icons-lucide --features app-integration app_install_records_pack_metadata_and_freezes_registry
cargo test -p fret-icons-radix --features app-integration app_install_records_pack_metadata_and_freezes_registry
cargo check -p fret-bootstrap --features "icons-lucide,icons-radix,icons-ui-semantic-lucide,icons-ui-semantic-radix"
cargo nextest run -p fret-bootstrap
cargo nextest run -p fret usage_docs_prefer_explicit_app_submodules_for_optional_ecosystems todo_golden_path_keeps_icon_pack_setup_on_app_install_surface
cargo check -p fret-ui-material3
python3 tools/check_layering.py
git diff --check
```

## Current evidence anchors

### Lane status

- baseline audit: `docs/workstreams/icon-system-extension-v1/BASELINE_AUDIT_2026-04-09.md`
- baseline decision: `docs/workstreams/icon-system-extension-v1/BASELINE_DECISION_2026-04-09.md`
- target interface state: `docs/workstreams/icon-system-extension-v1/TARGET_INTERFACE_STATE.md`
- closeout audit: `docs/workstreams/icon-system-extension-v1/CLOSEOUT_AUDIT_2026-04-09.md`

### Contract

- ADR: `docs/adr/0065-icon-system-and-asset-packaging.md`
- User-facing guidance: `docs/crate-usage-guide.md`
- Golden-path guidance: `docs/examples/todo-app-golden-path.md`

### Registry / pack protocol implementation

- icon registry and ids: `ecosystem/fret-icons/src/lib.rs`
- Lucide pack metadata + install seam: `ecosystem/fret-icons-lucide/src/{lib.rs,app.rs}`
- Radix pack metadata + install seam: `ecosystem/fret-icons-radix/src/{lib.rs,app.rs}`
- bootstrap pack registration: `ecosystem/fret-bootstrap/src/lib.rs`
- source-policy / teaching-surface proof: `ecosystem/fret/src/lib.rs`

### Authoring helper posture

- default icon helper + preload: `ecosystem/fret-ui-kit/src/declarative/icon.rs`

### Runtime/render surface

- declarative icon props: `crates/fret-ui/src/element.rs`
- authoring helper exposure: `crates/fret-ui/src/elements/cx.rs`
- declarative lowering to scene ops: `crates/fret-ui/src/declarative/host_widget/paint.rs`
- runtime proof for `SceneOp::SvgImage`: `crates/fret-ui/src/declarative/tests/svg_image.rs`
- low-level canvas split between mask/image SVG paths: `crates/fret-ui/src/canvas.rs`
- scene contract: `crates/fret-core/src/scene/mod.rs`

### Demo / proof surface

- cookbook example: `apps/fret-cookbook/examples/icons_and_assets_basics.rs`

## Gate policy for this lane

- Any future `fret-icons` change must keep component code semantics-first.
- Any runtime surface follow-on must leave the monochrome icon path intact and gated.
- Any bootstrap/doc follow-on must preserve the explicit custom-pack install story and pack
  provenance recording.
- If the accepted ADR changes materially, update:
  - `docs/adr/0065-icon-system-and-asset-packaging.md`
  - `docs/adr/IMPLEMENTATION_ALIGNMENT.md`
