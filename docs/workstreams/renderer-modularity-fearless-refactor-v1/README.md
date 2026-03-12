# Renderer Modularity (Fearless Refactor v1)

Status: Draft

Last updated: 2026-03-12

## Motivation

Fret's renderer stack is already on the right architectural path:

- `crates/fret-render-core` exists as a portable contract crate.
- `crates/fret-render` exists as a public facade.
- `crates/fret-render-wgpu` holds the default backend implementation.
- `crates/fret-render-wgpu` is protected by a strong regression net:
  - `cargo nextest run -p fret-render-wgpu`
  - `python3 tools/check_layering.py`

The current problem is not "the renderer is fundamentally wrong." The current problem is that the
public surface and the internal module boundaries are still wider than they need to be:

- `crates/fret-render` currently re-exports the entire wgpu backend surface wholesale.
- `crates/fret-render-wgpu` still exposes some types that behave more like implementation details
  than stable author-facing contracts.
- `Renderer` remains a large multi-domain owner spanning text, SVG, paths, materials, custom
  effects, intermediate budgeting, diagnostics, and GPU resource management.
- a few API seams still center the "editor-hosted" `WgpuContext` path more strongly than the
  "engine-hosted" topology described in `docs/architecture.md`.

This workstream exists to make renderer refactors boring, staged, and reversible:

- shrink the stable public surface to what we actually want to support,
- keep backend semantics stable while we modularize internals,
- preserve the host-provided GPU context topology as a first-class contract,
- and leave behind evidence/gates strong enough that we can refactor without fear.

## Goals

- Turn `crates/fret-render` into a curated facade instead of a wildcard backend dump.
- Keep `crates/fret-render-core` as the home for portable render-facing contract types.
- Make engine-hosted and editor-hosted GPU topologies equally first-class at the API level.
- Reduce `fret-render-wgpu` internal coupling so text, SVG, plan compilation, execution, and
  diagnostics can evolve independently.
- Preserve current render semantics while modularizing the implementation.

## Non-goals

- Rewriting the renderer from scratch.
- Changing shadcn/Radix/component policy behavior.
- Redesigning render-plan semantics in this workstream.
  - Existing semantic guardrails remain tracked by
    `docs/workstreams/renderer-render-plan-semantics-audit-v1/renderer-render-plan-semantics-audit-v1.md`.
- Replacing `wgpu` with another backend in v1.

## Scope

- Public render facade and contract boundaries:
  - `crates/fret-render`
  - `crates/fret-render-core`
- Default backend implementation:
  - `crates/fret-render-wgpu`
- High-churn consumers that currently depend on the facade:
  - `crates/fret-launch`
  - `ecosystem/fret`
  - demos / cookbook / stress apps under `apps/`

## Current Snapshot

As of 2026-03-12:

- `crates/fret-render/src/lib.rs` is effectively a wildcard re-export facade.
- `crates/fret-render-wgpu/src/lib.rs` re-exports a broad mix of:
  - stable-facing contracts,
  - backend bootstrap helpers,
  - diagnostics stores,
  - and types that may not need to stay public.
- `crates/fret-render-wgpu/src/renderer/mod.rs` still defines a large `Renderer` state owner.
- `crates/fret-render-wgpu/src/text/mod.rs` and `crates/fret-render-wgpu/src/renderer/shaders.rs`
  are the most obvious oversized internal modules.
- `Renderer::new(adapter, device)` and `render_scene(device, queue, ...)` already make
  host-provided GPU objects possible, but some convenience/diagnostics surfaces still privilege
  `WgpuContext`.
- The first code slice has landed:
  - `crates/fret-render` now uses an explicit facade export list instead of wildcard re-export.
  - `RendererCapabilities::from_adapter_device(...)` now exists and is used by first-party runner
    paths.
- The first internal text split has landed:
  - glyph atlas bookkeeping now lives under `crates/fret-render-wgpu/src/text/atlas.rs`
  - `text/mod.rs` no longer owns atlas/page/upload/eviction internals directly
- The second internal text split has landed:
  - text diagnostics/debug snapshot code now lives under
    `crates/fret-render-wgpu/src/text/diagnostics.rs`
  - `text/mod.rs` no longer owns atlas/debug/perf snapshot helpers directly
- The third internal text split has landed:
  - text quality state and gamma helpers now live under
    `crates/fret-render-wgpu/src/text/quality.rs`
  - `text/mod.rs` no longer owns text quality configuration/state internals directly
- The fourth internal text split has landed:
  - text tests now live under `crates/fret-render-wgpu/src/text/tests.rs`
  - `text/mod.rs` now keeps only `#[cfg(test)] mod tests;` as the test entrypoint
- The fifth internal text split has landed:
  - font catalog / fallback lifecycle helpers now live under
    `crates/fret-render-wgpu/src/text/fonts.rs`
  - `text/mod.rs` no longer owns font enumeration, locale updates, rescan flow, or font-family
    cache reset helpers directly
- The sixth internal text split has landed:
  - text blob access / release / eviction helpers now live under
    `crates/fret-render-wgpu/src/text/blobs.rs`
  - `text/mod.rs` no longer owns released-blob LRU maintenance and blob eviction helpers directly
- The seventh internal text split has landed:
  - text measurement helpers now live under `crates/fret-render-wgpu/src/text/measure.rs`
  - `text/mod.rs` no longer owns the plain/attributed measurement entrypoints directly
- The eighth internal text split has landed:
  - caret / hit-test / selection / line-metrics helpers now live under
    `crates/fret-render-wgpu/src/text/queries.rs`
  - `text/mod.rs` no longer owns text query helpers directly
- The ninth internal text split has landed:
  - atlas runtime helpers now live under `crates/fret-render-wgpu/src/text/atlas.rs`
  - `text/mod.rs` no longer owns atlas bind-group access, scene pinning, or glyph ensure/rasterize
    helpers directly
- The tenth internal text split has landed:
  - prepare entrypoints and prepare-specific trace/decoration helpers now live under
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `text/mod.rs` no longer owns plain/attributed prepare entrypoints or prepare support helpers
    directly
- The eleventh internal text split has landed:
  - blob-cache fast-path reuse and prepared-blob finalization helpers now live under
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `text/mod.rs` no longer owns prepare cache reuse or prepared-blob finalization directly
- The twelfth internal text split has landed:
  - shape-cache hit/store helpers now live under `crates/fret-render-wgpu/src/text/prepare.rs`
  - `text/mod.rs` no longer owns shape-cache hit/store bookkeeping directly
- The thirteenth internal text split has landed:
  - shape-build prelude and shape-finalization helpers now live under
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `text/mod.rs` no longer owns prepare shape-build setup or `TextShape` finalization directly
- The fourteenth internal text split has landed:
  - prepared-line glyph materialization now lives under
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `text/mod.rs` no longer owns active prepared-line glyph materialization directly
- The fifteenth internal text split has landed:
  - prepared-glyph face bookkeeping and paint-span resolution now live behind dedicated helpers in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `materialize_prepared_line(...)` no longer owns font-face bookkeeping directly
- The sixteenth internal text split has landed:
  - prepared-glyph atlas key construction and atlas lookup now live behind dedicated helpers in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `materialize_prepared_line(...)` no longer owns atlas-hit search directly
- The seventeenth internal text split has landed:
  - prepared-glyph atlas-miss rasterization now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `materialize_prepared_line(...)` no longer owns the atlas-miss branch directly
- The eighteenth internal text split has landed:
  - prepared-glyph atlas insertion now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `materialize_prepared_glyph_miss(...)` no longer owns atlas-kind insertion branching directly
- The nineteenth internal text split has landed:
  - prepared-glyph raster render output now flows through a dedicated `PreparedGlyphRaster`
    helper type in `crates/fret-render-wgpu/src/text/prepare.rs`
  - `materialize_prepared_glyph_miss(...)` now coordinates render, insert, and bounds steps
    without owning the raster payload layout directly
- The twentieth internal text split has landed:
  - prepared-glyph `swash` image rendering now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `render_prepared_glyph_raster(...)` no longer owns `FontRef/scaler/Render::new(...).render(...)`
    directly
- The twenty-first internal text split has landed:
  - prepared-glyph image-to-raster mapping now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `render_prepared_glyph_raster(...)` now just composes image render and raster mapping
- The twenty-second internal text split has landed:
  - prepared-glyph atlas target selection now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `insert_prepared_glyph_raster(...)` now uses one shared atlas insertion path
- The twenty-third internal text split has landed:
  - prepared-glyph font-ref/scaler setup now lives behind dedicated helpers in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `render_prepared_glyph_image(...)` no longer owns `FontRef` and scaler construction directly
- The twenty-fourth internal text split has landed:
  - prepared-glyph subpixel offset construction now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `render_prepared_glyph_image(...)` no longer owns `offset_px` construction directly
- The twenty-fifth internal text split has landed:
  - prepared-glyph render invocation now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `render_prepared_glyph_image(...)` now just wires scaler setup, offset setup, and render
    invocation together
- The twenty-sixth internal text split has landed:
  - prepared-glyph atlas-hit bounds normalization now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `materialize_prepared_line(...)` no longer owns atlas-hit placement math directly
- The twenty-seventh internal text split has landed:
  - prepared-glyph bounds resolution now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `materialize_prepared_line(...)` no longer owns atlas lookup and miss fallback branching directly
- The twenty-eighth internal text split has landed:
  - prepared-glyph instance assembly now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `materialize_prepared_line(...)` no longer owns `GlyphInstance` rect normalization directly
- The twenty-ninth internal text split has landed:
  - prepared-glyph origin/bin quantization now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `materialize_prepared_line(...)` no longer owns `x/y` subpixel bin setup directly
- The thirtieth internal text split has landed:
  - prepared-glyph context assembly now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `materialize_prepared_line(...)` no longer owns `glyph_id/face_key/size_bits` setup directly
- The thirty-first internal text split has landed:
  - prepared-line per-glyph materialization now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `materialize_prepared_line(...)` now just iterates prepared glyphs and pushes returned instances
- The thirty-second internal text split has landed:
  - prepared-line glyph-drain materialization now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `materialize_prepared_line(...)` no longer owns the prepared-glyph loop directly
- The thirty-third internal text split has landed:
  - prepared-glyph face-key construction now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `register_prepared_glyph_face(...)` no longer owns `FontFaceKey` construction directly
- The thirty-fourth internal text split has landed:
  - prepared-glyph face-cache writes now live behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `register_prepared_glyph_face(...)` no longer owns font-data/coords cache writes directly
- The thirty-fifth internal text split has landed:
  - prepared-glyph face-usage accounting now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `register_prepared_glyph_face(...)` no longer owns face-usage counter updates directly
- The thirty-sixth internal text split has landed:
  - prepared-glyph per-kind atlas lookup now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `lookup_prepared_glyph_atlas(...)` no longer owns per-kind atlas lookup branches directly
- The thirty-seventh internal text split has landed:
  - prepared-glyph atlas lookup order now lives behind one shared constant in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `lookup_prepared_glyph_atlas(...)` now just iterates that order and short-circuits on hit
- The thirty-eighth internal text split has landed:
  - prepared-glyph atlas-entry fetch/pack now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `lookup_prepared_glyph_atlas_kind(...)` no longer owns atlas-entry fetch result packing directly
- The thirty-ninth internal text split has landed:
  - prepared-glyph atlas lookup-key construction now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `lookup_prepared_glyph_atlas_kind(...)` no longer owns lookup-key construction directly
- The fortieth internal text split has landed:
  - prepared-glyph raster commit now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `materialize_prepared_glyph_miss(...)` no longer owns raster-bounds/atlas-insert sequencing directly
- The forty-first internal text split has landed:
  - prepared-glyph atlas-hit bounds resolution now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `resolve_prepared_glyph_bounds(...)` no longer owns atlas-hit branching directly
- The forty-second internal text split has landed:
  - prepared-glyph raster metadata decoding now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_raster_from_image(...)` no longer owns content-kind/bytes-per-pixel mapping directly
- The forty-third internal text split has landed:
  - prepared-glyph render source selection now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `render_prepared_glyph_image_with_scaler(...)` no longer owns the swash source list literal directly
- The forty-fourth internal text split has landed:
  - prepared-glyph raster packing now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_raster_from_image(...)` no longer owns `PreparedGlyphRaster` field packing directly
- The forty-fifth internal text split has landed:
  - prepared-glyph normalized-coords injection now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `build_prepared_glyph_scaler(...)` no longer owns normalized-coords branching directly
- The forty-sixth internal text split has landed:
  - prepared-glyph raster placement field extraction now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_raster_from_image(...)` no longer owns zero-size placement guarding and field
    unpacking directly
- The forty-seventh internal text split has landed:
  - prepared-glyph atlas insertion argument packing now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `insert_prepared_glyph_raster(...)` no longer owns `GlyphAtlas::get_or_insert(...)` argument
    packing directly
- The forty-eighth internal text split has landed:
  - prepared-glyph image-part raster assembly now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_raster_from_image(...)` no longer owns content decode plus raster assembly
    directly
- The forty-ninth internal text split has landed:
  - prepared-glyph keyed raster packing now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_raster(...)` no longer owns keyed `PreparedGlyphRaster` field packing directly
- The fiftieth internal text split has landed:
  - prepared-glyph raster-key derivation now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_raster(...)` no longer shares the atlas-lookup key helper directly
- The fifty-first internal text split has landed:
  - prepared-glyph synthesis skew normalization now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_face_key(...)` no longer owns skew clamp/default logic directly
- The fifty-second internal text split has landed:
  - prepared-glyph variation-key derivation now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_face_key(...)` no longer owns variation-key derivation directly
- Slice 1 verification passed after the first facade/topology changes:
  - `cargo nextest run -p fret-render -p fret-render-wgpu`: 221/221 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
- Internal text split verification remains green after the test-module extraction:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
- Internal text split verification remains green after the font/fallback extraction:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
- Internal text split verification remains green after the blob lifecycle extraction:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
- Internal text split verification remains green after the measure/query extraction:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
- Internal text split verification remains green after the atlas runtime extraction:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
- Internal text split verification remains green after the prepare support extraction:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
- Internal text split verification remains green after the prepare cache-helper extraction:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
- Internal text split verification remains green after the prepare shape-cache extraction:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
- Internal text split verification remains green after the prepare shape-shell extraction:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
- Internal text split verification remains green after the prepared-line materialization extraction:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
- Internal text split verification remains green after the prepared-glyph bookkeeping extraction:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
- Internal text split verification remains green after the prepared-glyph atlas lookup extraction:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
- Internal text split verification remains green after the prepared-glyph atlas-miss extraction:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
- Internal text split verification remains green after the prepared-glyph atlas-insert extraction:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
- Internal text split verification remains green after the prepared-glyph raster-shell extraction:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
- Internal text split verification remains green after the prepared-glyph image-render extraction:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
- Internal text split verification remains green after the prepared-glyph image-to-raster extraction:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
- Internal text split verification remains green after the prepared-glyph atlas-selection extraction:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
- Internal text split verification remains green after the prepared-glyph scaler extraction:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
- Internal text split verification remains green after the prepared-glyph offset extraction:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
- Internal text split verification remains green after the prepared-glyph render-invocation extraction:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
- Internal text split verification remains green after the prepared-glyph atlas-hit-bounds extraction:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
- Internal text split verification remains green after the prepared-glyph bounds-resolution extraction:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
- Internal text split verification remains green after the prepared-glyph instance extraction:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
- Internal text split verification remains green after the prepared-glyph origin-bin extraction:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
- Internal text split verification remains green after the prepared-glyph context extraction:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
- Internal text split verification remains green after the prepared-line glyph-materialization extraction:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
- Internal text split verification remains green after the prepared-line glyph-drain extraction:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
- Internal text split verification remains green after the prepared-glyph face-key extraction:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
- Internal text split verification remains green after the prepared-glyph face-cache extraction:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
- Internal text split verification remains green after the prepared-glyph face-usage extraction:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
- Internal text split verification remains green after the prepared-glyph atlas-kind extraction:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
- Internal text split verification remains green after the prepared-glyph atlas-order extraction:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
- Internal text split verification remains green after the prepared-glyph atlas-entry extraction:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
- Baseline gates passed during the pre-workstream audit:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `python3 tools/check_layering.py`: passed

## Evidence Anchors

- Architecture topology:
  - `docs/architecture.md`
- Current facade:
  - `crates/fret-render/src/lib.rs`
- Current backend facade and init path:
  - `crates/fret-render-wgpu/src/lib.rs`
  - `crates/fret-render-wgpu/src/capabilities.rs`
  - `crates/fret-render-wgpu/src/surface.rs`
- Current renderer owner and hot paths:
  - `crates/fret-render-wgpu/src/renderer/mod.rs`
  - `crates/fret-render-wgpu/src/renderer/resources.rs`
  - `crates/fret-render-wgpu/src/renderer/render_scene/render.rs`
  - `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs`
  - `crates/fret-render-wgpu/src/text/mod.rs`
- Existing semantic guardrails:
  - `crates/fret-render-wgpu/src/renderer/render_plan.rs`
  - `docs/workstreams/renderer-render-plan-semantics-audit-v1/renderer-render-plan-semantics-audit-v1.md`

## Documents

- Design: `docs/workstreams/renderer-modularity-fearless-refactor-v1/DESIGN.md`
- TODO: `docs/workstreams/renderer-modularity-fearless-refactor-v1/TODO.md`
- Milestones: `docs/workstreams/renderer-modularity-fearless-refactor-v1/MILESTONES.md`
- Surface inventory: `docs/workstreams/renderer-modularity-fearless-refactor-v1/SURFACE_INVENTORY.md`

## Locked v1 Decisions

The following decisions are considered locked for the start of v1:

1. No new renderer crates in v1.
   - Work happens inside `crates/fret-render`, `crates/fret-render-core`, and
     `crates/fret-render-wgpu`.
2. `crates/fret-render` remains the default stable facade.
   - It stops using wildcard re-export, but it does not stop being the default entrypoint.
3. `crates/fret-render-core` remains value-only and portable.
   - It should not absorb backend bootstrap objects or `wgpu`-bound handles.
4. `WgpuContext` remains supported as a convenience path.
   - It is not the only first-class integration path.
5. Host-provided GPU topology closure is P0.
   - v1 must add capability/bootstrap helpers that work without forcing `WgpuContext`.
6. Render-plan semantics are treated as frozen inputs for this workstream.
   - Modularization work should not quietly redesign pass semantics.
7. The first high-value internal extraction target is `crates/fret-render-wgpu/src/text/mod.rs`.
   - `renderer/shaders.rs` is not the first breakup target.
8. Backend-only cache/registry-style exports are presumed shrink candidates until proven otherwise.

## Recommended v1 Approach

- Keep the refactor staged and behavior-preserving.
- Lock the facade surface before chasing internal cleanup.
- Make host-provided GPU topology closure a P0 seam before any deep internal extraction.
- Prefer extracting cohesive domains out of `Renderer` over inventing new abstraction layers
  prematurely.
