# Baseline Audit — 2026-04-09

This audit records the starting point for `icon-system-extension-v1`.

Goal:

- confirm which parts of ADR 0065 are already implemented,
- separate real contract/runtime gaps from documentation drift,
- and freeze the first execution slice before changing the icon contract or declarative surface.

## Audit inputs

Contract / lane docs reviewed:

- `docs/adr/0065-icon-system-and-asset-packaging.md`
- `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md`
- `docs/runtime-contract-matrix.md`
- `docs/crate-usage-guide.md`
- `docs/examples/todo-app-golden-path.md`
- `docs/workstreams/icon-system-extension-v1/DESIGN.md`
- `docs/workstreams/icon-system-extension-v1/TODO.md`
- `docs/workstreams/icon-system-extension-v1/MILESTONES.md`
- `docs/workstreams/icon-system-extension-v1/EVIDENCE_AND_GATES.md`

Implementation / proof anchors reviewed:

- `ecosystem/fret-icons/src/lib.rs`
- `ecosystem/fret-icons-lucide/src/lib.rs`
- `ecosystem/fret-icons-radix/src/lib.rs`
- `ecosystem/fret-bootstrap/src/lib.rs`
- `ecosystem/fret/src/integration.rs`
- `ecosystem/fret-ui-kit/src/declarative/icon.rs`
- `crates/fret-core/src/scene/mod.rs`
- `crates/fret-ui/src/element.rs`
- `crates/fret-ui/src/elements/cx.rs`
- `crates/fret-ui/src/declarative/host_widget/paint.rs`
- `crates/fret-ui/src/canvas.rs`
- `crates/fret-render-wgpu/src/renderer/svg/prepare.rs`
- `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/svg.rs`

Related docs with evidence drift reviewed:

- `docs/adr/0317-portable-asset-locator-and-resolver-contract-v1.md`
- `docs/workstreams/resource-loading-fearless-refactor-v1/ECOSYSTEM_INSTALLER_COMPOSITION.md`

## Findings

### 1. ADR 0065's semantics-first identity contract is already real

The current `fret-icons` implementation already preserves the most important part of ADR 0065:

- component-facing identity is `IconId`,
- the registry is renderer-agnostic,
- aliases are explicit data,
- vendor namespaces remain ordinary icon ids rather than special cases,
- and app/bootstrap code owns override order.

`fret-bootstrap` also already exposes an explicit raw pack seam via
`BootstrapBuilder::register_icon_pack(...)`, while first-party Lucide/Radix crates publish both
registry helpers and `app::install` entrypoints.

Conclusion:

- Fret already has a sound base for icon-pack extensibility.
- This lane should extend that contract rather than replace it.

### 2. The runtime already supports both monochrome and RGBA SVG rendering, but the default declarative icon path is still monochrome-only

At the renderer/scene level, the split already exists:

- `SceneOp::SvgMaskIcon` for tinted alpha-mask rendering,
- `SceneOp::SvgImage` for RGBA SVG rendering,
- and `Canvas::svg_image(...)` already exposes the RGBA path.

However, the default declarative icon authoring surface still only models the monochrome path:

- `SvgIconProps` carries color/inherited-color state,
- `ElementContext` exposes `svg_icon(...)` / `svg_icon_props(...)`,
- and paint lowering emits `SceneOp::SvgMaskIcon` unconditionally for `ElementInstance::SvgIcon`.

Conclusion:

- multicolor SVG support exists below the public declarative icon surface,
- so the missing piece is primarily contract/authoring work, not a renderer invention problem.

### 3. Third-party icon-pack crates are already intentionally supported, but the minimum approved pack protocol is still implicit

Current docs and code already teach real pack extension seams:

- `docs/crate-usage-guide.md` tells apps to use `FretApp::setup(my_icons::app::install)` or
  `BootstrapBuilder::register_icon_pack(...)`,
- `docs/examples/todo-app-golden-path.md` repeats that custom-pack guidance,
- and first-party Lucide/Radix crates already publish the same shape.

The missing part is not the ability to build custom packs.
The missing part is a lane-owned statement of the minimum approved contract for those crates, such
as:

- whether pack metadata needs an explicit render-mode story,
- which install seams are required,
- and which parts are pack-private implementation details vs app-facing contract.

Conclusion:

- Fret can already host third-party import-pack crates today,
- but maintainability/reviewability still depends on scattered examples instead of one explicit protocol.

### 4. ADR 0065 names glyph fallback, but `IconSource` still does not model it

ADR 0065 says `IconSource` should support:

- SVG bytes,
- glyph fallback,
- aliases.

The current enum in `ecosystem/fret-icons/src/lib.rs` only exposes:

- `SvgStatic`,
- `SvgBytes`,
- `Alias`.

The registry does provide a deterministic missing-icon SVG fallback at resolve time, but that is
not the same contract as a first-class glyph fallback source.

Conclusion:

- ADR 0065 is ahead of implementation here,
- and the lane should decide whether to implement glyph fallback directly or narrow/correct the ADR language.

### 5. The strongest current proof for install layering lives in `ecosystem/fret`, not in a dedicated `fret-icons` test file

The repo currently has first-party tests proving two important extension behaviors:

- an ecosystem bundle installer can publish both package assets and icons,
- and app follow-up install can override a semantic icon without replaying bundle mounts.

Those proofs live in `ecosystem/fret/src/integration.rs`.
This matters because multiple docs still point to a non-existent
`ecosystem/fret-icons/tests/semantic_alias_conflicts.rs`.

Conclusion:

- install/override behavior is real and already tested,
- but the canonical evidence anchors are stale and should now point at the integration tests that actually exist.

### 6. `docs/adr/IMPLEMENTATION_ALIGNMENT.md` currently overstates ADR 0065 closure

The alignment row currently marks ADR 0065 as `Aligned`.
That is too strong for the current repo state because:

- glyph fallback is still absent from `IconSource`,
- declarative multicolor icon authoring is still missing even though the lower runtime supports it,
- and one cited evidence file does not exist.

Conclusion:

- the row should eventually move to either `Aligned (with known gaps)` or `Partially aligned`,
- but this lane should first decide whether the fix is purely evidence wording or a wider contract update.

## Decision from this audit

Treat the current repo state as follows:

- the semantics-first icon contract is a good foundation and should remain intact,
- renderer/runtime support is already sufficient for a first declarative multicolor icon surface,
- third-party custom packs are already viable and should be made explicit rather than reinvented,
- and the first contract slice should focus on render-mode / metadata placement and pack-protocol wording before broad implementation work.

## Immediate execution consequence

From this point forward:

1. keep this lane scoped to `fret-icons` contract closure, declarative multicolor icon authoring,
   and the third-party pack protocol,
2. do not widen this lane into remote Iconify clients, generic asset-registry redesign, or
   design-system-specific icon policy,
3. use `ecosystem/fret/src/integration.rs` as the current install-layering proof anchor until a
   dedicated `fret-icons` conflict/override test exists,
4. decide the minimum render-mode contract before changing `crates/fret-ui` public authoring APIs,
5. update ADR/alignment wording only after that minimum contract decision is explicit.
