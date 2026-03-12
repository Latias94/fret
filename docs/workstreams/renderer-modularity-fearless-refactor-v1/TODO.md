# Renderer Modularity (Fearless Refactor v1) — TODO

Status: In progress

Last updated: 2026-03-12

Related:

- Purpose: `docs/workstreams/renderer-modularity-fearless-refactor-v1/README.md`
- Design: `docs/workstreams/renderer-modularity-fearless-refactor-v1/DESIGN.md`
- Milestones: `docs/workstreams/renderer-modularity-fearless-refactor-v1/MILESTONES.md`
- Render semantics audit:
  - `docs/workstreams/renderer-render-plan-semantics-audit-v1/renderer-render-plan-semantics-audit-v1.md`

Tracking legend:

- `[ ]` open
- `[~]` in progress
- `[x]` done
- `[!]` blocked

ID format:

- `RMFR-{area}-{nnn}`

---

## A. Baseline and Surface Audit

- [x] RMFR-audit-001 Confirm the current facade shape.
  - `crates/fret-render` is currently a wildcard re-export of `fret-render-wgpu`.
- [x] RMFR-audit-002 Capture the current backend baseline gates.
  - `cargo nextest run -p fret-render-wgpu`
  - `python3 tools/check_layering.py`
- [x] RMFR-audit-003 Confirm the current topology seam.
  - `Renderer::new(adapter, device)` and `render_scene(device, queue, ...)` already permit
    host-provided GPU objects.
- [x] RMFR-audit-004 Inventory every public export from `crates/fret-render-wgpu/src/lib.rs` and
  classify it:
  - stable facade surface,
  - backend-only but intentionally public,
  - likely accidental public export.
- [x] RMFR-audit-005 Inventory all first-party consumers of `fret_render::*` and group them by
  dependency pattern:
  - bootstrap only,
  - diagnostics only,
  - renderer mutation/services,
  - external texture / viewport integration.

---

## A1. Locked v1 Decisions

- [x] RMFR-decisions-006 Lock v1 to the existing renderer crate layout.
- [x] RMFR-decisions-007 Lock `crates/fret-render` as the stable default facade.
- [x] RMFR-decisions-008 Lock `crates/fret-render-core` as the portable value-contract crate.
- [x] RMFR-decisions-009 Lock `WgpuContext` as supported convenience API, not sole first-class
  path.
- [x] RMFR-decisions-010 Lock host-provided GPU topology closure as P0.
- [x] RMFR-decisions-011 Lock render-plan semantics as frozen inputs for modularization.
- [x] RMFR-decisions-012 Lock `text/mod.rs` as the first high-value internal breakup target.
- [x] RMFR-decisions-013 Lock cache/registry-style exports into "review for shrink" status by
  default.

---

## B. Facade and Contract Closure

- [x] RMFR-facade-010 Replace wildcard re-export in `crates/fret-render` with an explicit export
  list.
- [ ] RMFR-facade-011 Decide the stable v1 facade surface for:
  - `Renderer`
  - `RenderSceneParams`
  - `SurfaceState`
  - `WgpuContext`
  - capability snapshots
  - perf/report stores
- [~] RMFR-facade-012 Decide which current `fret-render-wgpu` exports should stop being re-exported
  by the default facade.
- [ ] RMFR-facade-013 Move or alias portable value contracts from backend-owned exports to
  `fret-render-core` where that improves ownership clarity.
- [ ] RMFR-facade-014 Document the intended stable meaning of `crates/fret-render`.

---

## C. Host-Provided GPU Topology Closure

- [x] RMFR-topology-020 Add capability helpers that work from adapter/device inputs directly rather
  than requiring `WgpuContext`.
- [~] RMFR-topology-021 Review surface/bootstrap helpers and confirm they stay usable for
  engine-hosted integration.
- [ ] RMFR-topology-022 Add or update at least one smoke path that exercises the host-provided GPU
  topology explicitly.
- [ ] RMFR-topology-023 Update docs/examples so both topology entrypoints are visible:
  - editor-hosted convenience path,
  - engine-hosted path.

---

## D. Internal Domain Extraction

### D1. Text

- [~] RMFR-text-030 Split `crates/fret-render-wgpu/src/text/mod.rs` into explicit submodules.
  - Suggested first slices:
    - font catalog / fallback policy
    - shaping + measurement
    - atlas/cache bookkeeping
    - diagnostics / tests
  - Landed so far:
    - glyph atlas bookkeeping moved into `crates/fret-render-wgpu/src/text/atlas.rs`
    - `text/mod.rs` now goes through atlas accessors instead of touching atlas internals directly
    - diagnostics/debug snapshot code moved into `crates/fret-render-wgpu/src/text/diagnostics.rs`
    - `text/mod.rs` no longer owns atlas/debug/perf snapshot helper implementations directly
    - text quality state/gamma helpers moved into `crates/fret-render-wgpu/src/text/quality.rs`
    - `text/mod.rs` no longer owns text quality configuration/state internals directly
    - text tests moved into `crates/fret-render-wgpu/src/text/tests.rs`
    - `text/mod.rs` now only declares `#[cfg(test)] mod tests;` for test coverage
    - font catalog / fallback lifecycle helpers moved into
      `crates/fret-render-wgpu/src/text/fonts.rs`
    - `text/mod.rs` no longer owns font enumeration, locale updates, system font rescan flow, or
      font-family cache reset helpers directly
    - text blob access / release / eviction helpers moved into
      `crates/fret-render-wgpu/src/text/blobs.rs`
    - `text/mod.rs` no longer owns released-blob LRU maintenance and blob eviction helpers
      directly
    - text measurement helpers moved into `crates/fret-render-wgpu/src/text/measure.rs`
    - `text/mod.rs` no longer owns the plain/attributed measurement entrypoints directly
    - caret / hit-test / selection / line-metrics helpers moved into
      `crates/fret-render-wgpu/src/text/queries.rs`
    - `text/mod.rs` no longer owns text query helpers directly
    - atlas runtime helpers moved into `crates/fret-render-wgpu/src/text/atlas.rs`
    - `text/mod.rs` no longer owns atlas bind-group access, scene pinning, or glyph
      ensure/rasterize helpers directly
    - prepare entrypoints and prepare-specific trace/decoration helpers moved into
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `text/mod.rs` no longer owns plain/attributed prepare entrypoints or prepare support helpers
      directly
    - blob-cache fast-path reuse and prepared-blob finalization helpers moved into
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `text/mod.rs` no longer owns prepare cache reuse or prepared-blob finalization directly
    - shape-cache hit/store helpers moved into
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `text/mod.rs` no longer owns shape-cache hit/store bookkeeping directly
    - shape-build prelude and shape-finalization helpers moved into
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `text/mod.rs` no longer owns prepare shape-build setup or `TextShape` finalization directly
    - prepared-line glyph materialization moved into
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `text/mod.rs` no longer owns active prepared-line glyph materialization directly
    - prepared-glyph face bookkeeping and paint-span resolution now live behind dedicated helpers
      in `crates/fret-render-wgpu/src/text/prepare.rs`
    - `materialize_prepared_line(...)` no longer owns font-face bookkeeping directly
    - prepared-glyph atlas key construction and atlas lookup now live behind dedicated helpers in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `materialize_prepared_line(...)` no longer owns atlas-hit search directly
    - prepared-glyph atlas-miss rasterization now lives behind a dedicated helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `materialize_prepared_line(...)` no longer owns the atlas-miss branch directly
    - prepared-glyph atlas insertion now lives behind a dedicated helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `materialize_prepared_glyph_miss(...)` no longer owns atlas-kind insertion branching
      directly
    - prepared-glyph raster render output now flows through a dedicated `PreparedGlyphRaster`
      helper type in `crates/fret-render-wgpu/src/text/prepare.rs`
    - `materialize_prepared_glyph_miss(...)` now only coordinates render, insert, and bounds
      steps
    - prepared-glyph `swash` image rendering now lives behind a dedicated helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `render_prepared_glyph_raster(...)` no longer owns the `FontRef/scaler/Render` pipeline
      directly
    - prepared-glyph image-to-raster mapping now lives behind a pure helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `render_prepared_glyph_raster(...)` now just composes image render and raster mapping
    - prepared-glyph atlas target selection now lives behind a dedicated helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `insert_prepared_glyph_raster(...)` now uses one shared atlas insertion path
    - prepared-glyph font-ref/scaler setup now lives behind dedicated helpers in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `render_prepared_glyph_image(...)` no longer owns `FontRef` and scaler construction
      directly
    - prepared-glyph subpixel offset construction now lives behind a pure helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `render_prepared_glyph_image(...)` no longer owns `offset_px` construction directly
    - prepared-glyph render invocation now lives behind a dedicated helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `render_prepared_glyph_image(...)` now just wires scaler setup, offset setup, and render
      invocation together
    - prepared-glyph atlas-hit bounds normalization now lives behind a pure helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `materialize_prepared_line(...)` no longer owns atlas-hit placement math directly
    - prepared-glyph bounds resolution now lives behind a dedicated helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `materialize_prepared_line(...)` no longer owns atlas lookup and miss fallback branching
      directly
    - prepared-glyph instance assembly now lives behind a pure helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `materialize_prepared_line(...)` no longer owns `GlyphInstance` rect normalization directly
    - prepared-glyph origin/bin quantization now lives behind a pure helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `materialize_prepared_line(...)` no longer owns `x/y` subpixel bin setup directly
    - prepared-glyph context assembly now lives behind a dedicated helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `materialize_prepared_line(...)` no longer owns `glyph_id/face_key/size_bits` setup
      directly
    - prepared-line per-glyph materialization now lives behind a dedicated helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `materialize_prepared_line(...)` now just iterates prepared glyphs and pushes returned
      instances
    - prepared-line glyph-drain materialization now lives behind a dedicated helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `materialize_prepared_line(...)` no longer owns the prepared-glyph loop directly
    - prepared-glyph face-key construction now lives behind a pure helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `register_prepared_glyph_face(...)` no longer owns `FontFaceKey` construction directly
    - prepared-glyph face-cache writes now live behind a dedicated helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `register_prepared_glyph_face(...)` no longer owns font-data/coords cache writes directly
    - prepared-glyph face-usage accounting now lives behind a pure helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `register_prepared_glyph_face(...)` no longer owns face-usage counter updates directly
    - prepared-glyph per-kind atlas lookup now lives behind a dedicated helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `lookup_prepared_glyph_atlas(...)` no longer owns per-kind atlas lookup branches directly
    - prepared-glyph atlas lookup order now lives behind one shared constant in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `lookup_prepared_glyph_atlas(...)` now just iterates that order and short-circuits on hit
    - prepared-glyph atlas-entry fetch/pack now lives behind a dedicated helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `lookup_prepared_glyph_atlas_kind(...)` no longer owns atlas-entry fetch result packing
      directly
    - prepared-glyph raster metadata decoding now lives behind a pure helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `prepared_glyph_raster_from_image(...)` no longer owns content-kind/bytes-per-pixel mapping
      directly
    - prepared-glyph render source selection now lives behind a pure helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `render_prepared_glyph_image_with_scaler(...)` no longer owns the swash source list literal
      directly
    - prepared-glyph raster packing now lives behind a dedicated helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `prepared_glyph_raster_from_image(...)` no longer owns `PreparedGlyphRaster` field packing
      directly
    - prepared-glyph normalized-coords injection now lives behind a dedicated helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `build_prepared_glyph_scaler(...)` no longer owns normalized-coords branching directly
    - prepared-glyph raster placement field extraction now lives behind a pure helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `prepared_glyph_raster_from_image(...)` no longer owns zero-size placement guarding and
      field unpacking directly
    - prepared-glyph atlas insertion argument packing now lives behind a dedicated helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `insert_prepared_glyph_raster(...)` no longer owns `GlyphAtlas::get_or_insert(...)`
      argument packing directly
    - prepared-glyph image-part raster assembly now lives behind a dedicated helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `prepared_glyph_raster_from_image(...)` no longer owns content decode plus raster assembly
      directly
    - prepared-glyph keyed raster packing now lives behind a dedicated helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `prepared_glyph_raster(...)` no longer owns keyed `PreparedGlyphRaster` field packing
      directly
    - prepared-glyph raster-key derivation now lives behind a dedicated helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `prepared_glyph_raster(...)` no longer shares the atlas-lookup key helper directly
    - prepared-glyph synthesis skew normalization now lives behind a pure helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `prepared_glyph_face_key(...)` no longer owns skew clamp/default logic directly
    - prepared-glyph variation-key derivation now lives behind a pure helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `prepared_glyph_face_key(...)` no longer owns variation-key derivation directly
    - prepared-glyph synthesis embolden extraction now lives behind a pure helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `prepared_glyph_face_key(...)` no longer owns synthesis-embolden extraction directly
    - prepared-glyph font-data cache writes now live behind a dedicated helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `cache_prepared_glyph_face_data(...)` no longer owns font-data entry writes directly
    - prepared-glyph instance-coords cache writes now live behind a dedicated helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `cache_prepared_glyph_face_data(...)` no longer owns normalized-coords cache writes directly
    - prepared-glyph size-bit derivation now lives behind a pure helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `prepare_prepared_glyph_context(...)` no longer owns font-size bit packing directly
    - prepared-glyph id conversion now lives behind a pure helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `prepare_prepared_glyph_context(...)` no longer owns `glyph.id -> u16` conversion directly
    - prepared-glyph font identity derivation now lives behind a pure helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `register_prepared_glyph_face(...)` no longer owns `font_data_id/face_index` extraction directly
    - prepared-glyph context assembly now lives behind a pure helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `prepare_prepared_glyph_context(...)` no longer owns `PreparedGlyphContext` field packing directly
    - prepared-glyph atlas-hit fallback dispatch now lives behind a dedicated helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `resolve_prepared_glyph_bounds(...)` no longer owns atlas-hit vs miss-fallback branching directly
    - prepared-glyph bin-offset image render dispatch now lives behind a dedicated helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `render_prepared_glyph_image(...)` no longer owns bin-offset derivation plus scaler render call directly
    - prepared-glyph scaler size clamp now lives behind a pure helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `build_prepared_glyph_scaler(...)` no longer owns font-size clamp logic directly
    - prepared-glyph scaler builder assembly now lives behind a dedicated helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `build_prepared_glyph_scaler(...)` no longer owns scale-context builder chaining directly
    - prepared-glyph normalized-coords presence checks now live behind a pure helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `cache_prepared_glyph_instance_coords(...)` and
      `apply_prepared_glyph_normalized_coords(...)` no longer own emptiness checks directly
    - prepared-glyph normalized-coords builder injection now lives behind a dedicated helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `apply_prepared_glyph_normalized_coords(...)` no longer owns
      `normalized_coords.iter()` builder injection directly
    - prepared-glyph scaler-builder normalized-coords assembly now lives behind a dedicated helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `build_prepared_glyph_scaler(...)` no longer owns normalized-coords builder assembly directly
    - prepared-glyph image rendering after font-ref resolution now lives behind a dedicated helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `render_prepared_glyph_image(...)` no longer owns scaler/render handoff after font-ref lookup directly
    - prepared-glyph image rendering after scaler construction now lives behind a dedicated helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `render_prepared_glyph_image_with_font_ref(...)` no longer owns scaler/render dispatch directly
    - prepared-glyph raster assembly after placement extraction now lives behind a dedicated helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `prepared_glyph_raster_from_image(...)` no longer owns placement-backed raster assembly directly
    - prepared-glyph raster assembly after image-content metadata decoding now lives behind a dedicated helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `prepared_glyph_raster_from_image_parts(...)` no longer owns content-metadata-backed raster assembly directly
    - prepared-glyph raster assembly after image rendering now lives behind a dedicated helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `render_prepared_glyph_raster(...)` no longer owns image-to-raster handoff directly
    - prepared-glyph raster image handoff now uses a projected glyph-id value helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `render_prepared_glyph_raster_from_image(...)` no longer depends on the full glyph record
    - prepared-glyph raster payload handoff after metadata decoding now lives behind a dedicated helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `prepared_glyph_raster_from_image_parts_with_metadata(...)` no longer owns `image.data` payload handoff directly
    - prepared-glyph raster payload field assembly now lives behind a dedicated helper in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `prepared_glyph_raster(...)` no longer constructs raster payload fields inline before final assembly
    - prepared-glyph raster placement extraction now returns a dedicated placement struct in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `prepared_glyph_raster_from_image(...)` no longer carries raw placement tuples across the next helper boundary
    - prepared-glyph raster metadata decoding now returns a dedicated metadata struct in
      `crates/fret-render-wgpu/src/text/prepare.rs`
    - `prepared_glyph_raster_from_image_parts(...)` no longer forwards raw `kind` and `bytes_per_pixel` values across the next helper boundary
- [ ] RMFR-text-031 Keep `fret_render_text` as the low-level text contract crate and avoid moving
  backend-specific state there prematurely.
- [ ] RMFR-text-032 Add focused tests around any extracted text subdomain whose behavior was
  previously only covered through the monolithic module.

### D2. Renderer state owner

- [ ] RMFR-renderer-040 Identify the subdomain state that can move out of `Renderer` without
  changing behavior.
- [ ] RMFR-renderer-041 Extract cohesive domain owners for:
  - text
  - SVG
  - materials/custom effects
  - intermediate budgeting/pools
  - diagnostics state
- [ ] RMFR-renderer-042 Reduce cross-domain mutable coupling inside `Renderer`.
- [ ] RMFR-renderer-043 Keep service trait implementations readable after extraction.

### D3. Shaders and pipelines

- [ ] RMFR-shaders-050 Audit whether `renderer/shaders.rs` needs ownership-oriented splitting or
  only comment/index cleanup.
- [ ] RMFR-shaders-051 Avoid splitting shader source files purely for line count if no boundary
  benefit exists.
- [ ] RMFR-shaders-052 Keep WGSL validation tests aligned with any source reorganization.

---

## E. Public Export Tightening

- [~] RMFR-exports-060 Review cache/registry-style exports and remove public visibility where no
  real consumer exists.
- [ ] RMFR-exports-061 Decide whether backend-only diagnostics stores belong in the stable default
  facade or under a more explicit backend namespace.
- [ ] RMFR-exports-062 Confirm whether `WgpuContext` remains a stable convenience surface or should
  be demoted in guidance.
- [ ] RMFR-exports-063 Update first-party callers after any facade shrink.

---

## F. Gates and Evidence

- [x] RMFR-gates-070 Establish backend baseline gates before refactor work.
- [~] RMFR-gates-071 Add a surface snapshot note or test proving the intended `fret-render` export
  set after facade curation.
- [ ] RMFR-gates-072 Add targeted smoke coverage for host-provided GPU topology if absent.
- [ ] RMFR-gates-073 Keep render-plan semantics guardrails green for any planning/execution change.
- [ ] RMFR-gates-074 If facade docs/examples change, leave evidence anchors in the workstream docs.

---

## G. Docs and Contract Follow-up

- [x] RMFR-docs-080 Create this workstream doc set.
- [x] RMFR-docs-085 Capture first-pass surface inventory and consumer buckets.
- [~] RMFR-docs-081 Update this tracker as refactor stages land.
  - Latest landed slice: prepared-glyph raster metadata struct helper in `text/prepare.rs`.
- [ ] RMFR-docs-082 Add or update an ADR if the stable renderer facade contract changes.
- [ ] RMFR-docs-083 If an ADR is added, update `docs/adr/IMPLEMENTATION_ALIGNMENT.md`.
- [ ] RMFR-docs-084 Decide whether this workstream also needs:
  - `EVIDENCE_AND_GATES.md`
  - `OPEN_QUESTIONS.md`
  - `MIGRATION_MATRIX.md`

---

## H. Cleanup / Exit

- [ ] RMFR-cleanup-090 Finish migrating first-party callers to the curated facade surface.
- [ ] RMFR-cleanup-091 Remove or quarantine exports that are now explicitly internal-only.
- [ ] RMFR-cleanup-092 Re-check whether additional crate splits are still necessary after internal
  modularization.
- [ ] RMFR-cleanup-093 Make sure the final docs teach one boring renderer integration story for
  each supported topology.
