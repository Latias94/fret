Status: Active (workstream tracker)

This document tracks cross-cutting TODOs for:

- `docs/workstreams/image-support-v1.md`

Tracking format:

- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked
- ID: `IMG-{area}-{nnn}`

When completing an item, prefer leaving 1–3 evidence anchors:

- file paths + key functions/tests
- and/or a `fretboard diag` script/suite name

## M0 — Lock decisions (no more “implicit stretch”)

- [x] IMG-adr-001 Draft and land an ADR for image `object-fit` semantics.
  - Must cover: fit enum vocabulary, default behavior, `ImageRegion` interaction rule, and backcompat.
  - References: `docs/adr/0121-streaming-images-and-video-surfaces.md`, `docs/adr/0004-resource-handles.md`.
  - Draft: `docs/adr/1170-image-object-fit-for-sceneop-image-v1.md`
  - Evidence:
    - `docs/adr/1170-image-object-fit-for-sceneop-image-v1.md`

- [ ] IMG-guard-010 Add a minimal regression gate checklist for image work (fast vs full).
  - Fast: `cargo fmt`, `cargo clippy -p fret-ui -p fret-render-wgpu -p fret-ui-shadcn --all-targets -- -D warnings`
  - Full: `cargo nextest run` (workspace) + key web_vs_fret suites.

## M1 — Core fit semantics (mechanism layer)

- [x] IMG-core-100 Add fit semantics to the core image draw primitive (`SceneOp::Image`).
  - Evidence anchors:
    - `crates/fret-core/src/scene/mod.rs`
    - `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/image.rs`
    - `crates/fret-core/src/scene/image_object_fit.rs` (`map_image_object_fit`)

- [x] IMG-core-101 Plumb fit through the declarative element surface (`ImageProps`) and paint path.
  - Evidence anchors:
    - `crates/fret-ui/src/element.rs`
    - `crates/fret-ui/src/declarative/host_widget/paint.rs`

- [x] IMG-test-120 Add conformance tests for fit mapping + UV crop math.
  - Prefer: pure math unit tests (no GPU required) + one renderer smoke test.
  - Must cover: clamp + monotonic UV rules and “no early rounding” guidance (ADR 1170).
  - Evidence:
    - `crates/fret-core/src/scene/image_object_fit.rs` (unit tests)
    - `crates/fret-render-wgpu/src/renderer/tests.rs` (`image_fit_*`)

## M2 — shadcn parity (user-visible win)

- [x] IMG-shadcn-200 Make `AvatarImage` default to cover semantics (no stretching).
  - Evidence anchors:
    - `ecosystem/fret-ui-shadcn/src/avatar.rs`
    - `ecosystem/fret-ui-shadcn/tests/*` (web_vs_fret snapshots)
    - `ecosystem/fret-ui-shadcn/src/avatar.rs` (`avatar_image_emits_cover_fit_scene_op`)

- [x] IMG-shadcn-210 Add a small shadcn `Image` recipe/component for cards/media rows.
  - Must support: `fit`, `loading` skeleton slot, and `fallback` slot (policy lives here).
  - Evidence:
    - `ecosystem/fret-ui-shadcn/src/media_image.rs` (`MediaImage`)
    - `ecosystem/fret-ui-shadcn/src/media_image.rs` (`media_image_emits_fit_scene_op_when_image_present`)

## M3 — Metadata query seam (optional but likely needed)

- [ ] IMG-meta-300 Decide whether to add an `ImageService` to `UiServices` (intrinsic size query).
  - If accepted, land an ADR or extend the fit ADR with the query contract.

- [ ] IMG-meta-310 Implement `ImageService` in the renderer and add a fake implementation for tests.
  - Evidence anchors:
    - `crates/fret-render-wgpu/src/renderer/services.rs`
    - `crates/fret-core/src/services.rs`

## M4 — Ecosystem `img(source)` (deferred until M1/M2 are stable)

- [ ] IMG-eco-400 Add an ecosystem crate (or module) for `ImageSource` + decode/load + cache integration.
  - Non-goal: framework-owned media engine.
  - Must integrate with: `fret-ui-assets` / `fret-asset-cache` budgets.

- [ ] IMG-eco-410 Add a demo + at least one diag script for:
  - scrolling thumbnails (virtual list),
  - video-like streaming updates (existing demos ok; add UI composition around them).

## M5 — Video fast paths (capability-gated)

- [ ] IMG-video-500 Audit streaming update perf on desktop + wasm and record a baseline.
  - References: ADR 0123/0124/0126.

- [ ] IMG-video-510 Decide the next capability-gated step for “external texture import”.
  - Must not leak backend handles into `fret-ui`.
