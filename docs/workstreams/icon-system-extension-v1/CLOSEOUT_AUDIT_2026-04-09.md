# Closeout Audit — 2026-04-09

This audit records the final closeout read for the icon-system-extension-v1 lane.

Goal:

- verify that the icon contract now covers semantic identity, multicolor declarative rendering,
  and third-party pack protocol/provenance without another planned public-contract rewrite,
- separate the shipped v1 boundary from future pack-specific follow-ons,
- and decide whether this lane should remain active or become historical evidence.

## Audited evidence

Core lane docs:

- `docs/workstreams/icon-system-extension-v1/DESIGN.md`
- `docs/workstreams/icon-system-extension-v1/TARGET_INTERFACE_STATE.md`
- `docs/workstreams/icon-system-extension-v1/TODO.md`
- `docs/workstreams/icon-system-extension-v1/MILESTONES.md`
- `docs/workstreams/icon-system-extension-v1/EVIDENCE_AND_GATES.md`

Contract / alignment docs:

- `docs/adr/0065-icon-system-and-asset-packaging.md`
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md`
- `docs/crate-usage-guide.md`
- `docs/examples/todo-app-golden-path.md`

Implementation anchors:

- `ecosystem/fret-icons/src/lib.rs`
- `ecosystem/fret-icons-lucide/src/{lib.rs,app.rs}`
- `ecosystem/fret-icons-radix/src/{lib.rs,app.rs}`
- `crates/fret-ui/src/{element.rs,elements/cx.rs}`
- `crates/fret-ui/src/declarative/host_widget/paint.rs`
- `crates/fret-ui/src/declarative/tests/svg_image.rs`
- `ecosystem/fret-ui-kit/src/declarative/icon.rs`
- `ecosystem/fret-bootstrap/src/lib.rs`

Validation run used for closeout:

- `cargo nextest run -p fret-icons -p fret-icons-lucide -p fret-icons-radix`
- `cargo test -p fret-icons-lucide --features app-integration app_install_records_pack_metadata_and_freezes_registry`
- `cargo test -p fret-icons-radix --features app-integration app_install_records_pack_metadata_and_freezes_registry`
- `cargo check -p fret-bootstrap --features "icons-lucide,icons-radix,icons-ui-semantic-lucide,icons-ui-semantic-radix"`
- `cargo nextest run -p fret-bootstrap`
- `cargo nextest run -p fret usage_docs_prefer_explicit_app_submodules_for_optional_ecosystems todo_golden_path_keeps_icon_pack_setup_on_app_install_surface`
- `cargo nextest run -p fret-ui svg_image_props_paint_to_svg_image_scene_op foreground_scope_late_binds_foreground_for_text_and_icons inherited_foreground_on_existing_root_late_binds_for_text_icon_and_spinner`
- `cargo nextest run -p fret-ui-kit`
- `cargo check -p fret-ui-material3`
- `python3 tools/check_layering.py`
- `git diff --check`

## Findings

### 1. The icon registry contract is no longer byte-only

The shipped contract now resolves full icon definitions rather than rediscovering render policy in
helpers:

- `IconDefinition` groups source + fallback + presentation,
- aliases preserve the full definition,
- `ResolvedIcon` / `ResolvedIconOwned` carry the render intent across registry resolution.

Conclusion:

- the registry contract is durable enough for multicolor icons and future import-pack tooling.

### 2. The runtime/declarative multicolor story is now first-class

The UI runtime no longer treats monochrome icons as the only declarative SVG surface:

- `SvgIconProps` remains the themed/tinted path,
- `SvgImageProps` is now the explicit authored-color path,
- `fret-ui-kit` preserves the old `icon(...)` posture and adds explicit authored rendering through
  `icon_authored(...)`.

Conclusion:

- multicolor SVG support is now part of the real runtime contract rather than a canvas-side escape
  hatch.

### 3. Third-party pack protocol and provenance are now explicit in code and docs

The pack protocol is no longer implied only by first-party examples:

- `fret-icons` now defines `IconPackRegistration`, `IconPackMetadata`, and
  `InstalledIconPacks`,
- first-party Lucide/Radix crates export `PACK_METADATA`, `PACK`, and `VENDOR_PACK`,
- app-facing installs record pack provenance and keep the registry/frozen snapshot aligned,
- bootstrap has a contract-aware pack entry seam:
  `BootstrapBuilder::register_icon_pack_contract(...)`,
- app-facing docs now describe the explicit custom-pack shape instead of sending users to infer it
  from source alone.

Conclusion:

- the pack/import boundary is explicit enough for third-party Iconify-style generators or vendored
  packs to target without another surface rewrite.

### 4. The remaining questions are no longer v1 contract debt

What remains after this lane is narrower than the original execution scope:

1. pack-specific parity work
   - for example vendor-specific semantic alias taste, curation policy, or generated pack tooling
2. diagnostics / UI proof surfaces
   - additional cookbook/gallery proof if product teams want a richer authored-icon showcase
3. future metadata consumers
   - diagnostics, package listings, or tooling that want to read `InstalledIconPacks`

These are follow-on opportunities, not unfinished contract closure.

Conclusion:

- this lane no longer owns an active cross-layer contract queue.

## Decision from this audit

Treat `icon-system-extension-v1` as:

- closed for the v1 icon contract / runtime surface / pack protocol goal,
- historical maintenance evidence by default,
- reopenable only through a narrower follow-on if future pack-specific or tooling-specific
  pressure appears.

## Immediate execution consequence

From this point forward:

1. keep semantic `IconId` as the reusable component-facing icon contract,
2. keep `SvgIcon` vs `SvgImage` as the explicit runtime split,
3. keep themed `icon(...)` separate from authored `icon_authored(...)`,
4. keep pack provenance explicit through `PACK_METADATA` plus app/bootstrap install seams,
5. do not reopen this lane just to explore one vendor pack or one design-system recipe.
