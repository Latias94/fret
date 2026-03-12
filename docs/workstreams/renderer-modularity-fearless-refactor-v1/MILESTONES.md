# Renderer Modularity (Fearless Refactor v1) — Milestones

Status: In progress

Related:

- Purpose: `docs/workstreams/renderer-modularity-fearless-refactor-v1/README.md`
- Design: `docs/workstreams/renderer-modularity-fearless-refactor-v1/DESIGN.md`
- TODO: `docs/workstreams/renderer-modularity-fearless-refactor-v1/TODO.md`

Current snapshot (2026-03-12):

- The renderer stack is not a rewrite candidate; it is a staged modularization candidate.
- `fret-render-wgpu` baseline gates are green:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `python3 tools/check_layering.py`: passed
- v1 start decisions are now locked:
  - no new renderer crates in v1,
  - `fret-render` stays the stable default facade,
  - `fret-render-core` stays portable/value-only,
  - host-provided GPU topology closure is P0,
  - render-plan semantics are treated as frozen inputs,
  - `text/mod.rs` is the first high-value breakup target.
- The first refactor slice has landed:
  - `crates/fret-render` now uses explicit re-exports instead of wildcard facade export.
  - `RendererCapabilities::from_adapter_device(...)` now exists and is used in first-party runner
    adoption paths.
- The first internal `text/mod.rs` split has landed:
  - glyph atlas bookkeeping moved into `crates/fret-render-wgpu/src/text/atlas.rs`
  - `text/mod.rs` now depends on atlas accessors instead of atlas internals
- The second internal `text/mod.rs` split has landed:
  - diagnostics/debug snapshot helpers moved into
    `crates/fret-render-wgpu/src/text/diagnostics.rs`
  - `text/mod.rs` no longer owns text diagnostics/debug helper implementations directly
- The third internal `text/mod.rs` split has landed:
  - text quality state/gamma helpers moved into
    `crates/fret-render-wgpu/src/text/quality.rs`
  - `text/mod.rs` no longer owns text quality configuration/state internals directly
- The fourth internal `text/mod.rs` split has landed:
  - tests moved into `crates/fret-render-wgpu/src/text/tests.rs`
  - `text/mod.rs` now keeps only a `#[cfg(test)] mod tests;` declaration for the test entrypoint
- The fifth internal `text/mod.rs` split has landed:
  - font catalog / fallback lifecycle helpers moved into
    `crates/fret-render-wgpu/src/text/fonts.rs`
  - `text/mod.rs` no longer owns font enumeration, locale updates, rescan flow, or font-family
    cache reset helpers directly
- The sixth internal `text/mod.rs` split has landed:
  - text blob access / release / eviction helpers moved into
    `crates/fret-render-wgpu/src/text/blobs.rs`
  - `text/mod.rs` no longer owns released-blob LRU maintenance and blob eviction helpers directly
- The seventh internal `text/mod.rs` split has landed:
  - text measurement helpers moved into `crates/fret-render-wgpu/src/text/measure.rs`
  - `text/mod.rs` no longer owns the plain/attributed measurement entrypoints directly
- The eighth internal `text/mod.rs` split has landed:
  - caret / hit-test / selection / line-metrics helpers moved into
    `crates/fret-render-wgpu/src/text/queries.rs`
  - `text/mod.rs` no longer owns text query helpers directly
- The ninth internal `text/mod.rs` split has landed:
  - atlas runtime helpers moved into `crates/fret-render-wgpu/src/text/atlas.rs`
  - `text/mod.rs` no longer owns atlas bind-group access, scene pinning, or glyph
    ensure/rasterize helpers directly
- The tenth internal `text/mod.rs` split has landed:
  - prepare entrypoints and prepare-specific trace/decoration helpers moved into
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `text/mod.rs` no longer owns plain/attributed prepare entrypoints or prepare support helpers
    directly
- The eleventh internal `text/mod.rs` split has landed:
  - blob-cache fast-path reuse and prepared-blob finalization helpers moved into
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `text/mod.rs` no longer owns prepare cache reuse or prepared-blob finalization directly
- The twelfth internal `text/mod.rs` split has landed:
  - shape-cache hit/store helpers moved into `crates/fret-render-wgpu/src/text/prepare.rs`
  - `text/mod.rs` no longer owns shape-cache hit/store bookkeeping directly
- The thirteenth internal `text/mod.rs` split has landed:
  - shape-build prelude and shape-finalization helpers moved into
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `text/mod.rs` no longer owns prepare shape-build setup or `TextShape` finalization directly
- The fourteenth internal `text/mod.rs` split has landed:
  - prepared-line glyph materialization moved into
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `text/mod.rs` no longer owns active prepared-line glyph materialization directly
- The fifteenth internal `text/mod.rs` split has landed:
  - prepared-glyph face bookkeeping and paint-span resolution now live behind dedicated helpers in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `materialize_prepared_line(...)` no longer owns font-face bookkeeping directly
- The sixteenth internal `text/mod.rs` split has landed:
  - prepared-glyph atlas key construction and atlas lookup now live behind dedicated helpers in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `materialize_prepared_line(...)` no longer owns atlas-hit search directly
- The seventeenth internal `text/mod.rs` split has landed:
  - prepared-glyph atlas-miss rasterization now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `materialize_prepared_line(...)` no longer owns the atlas-miss branch directly
- The eighteenth internal `text/mod.rs` split has landed:
  - prepared-glyph atlas insertion now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `materialize_prepared_glyph_miss(...)` no longer owns atlas-kind insertion branching directly
- The nineteenth internal `text/mod.rs` split has landed:
  - prepared-glyph raster render output now flows through a dedicated `PreparedGlyphRaster`
    helper type in `crates/fret-render-wgpu/src/text/prepare.rs`
  - `materialize_prepared_glyph_miss(...)` now coordinates render, insert, and bounds steps
    without owning the raster payload layout directly
- The twentieth internal `text/mod.rs` split has landed:
  - prepared-glyph `swash` image rendering now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `render_prepared_glyph_raster(...)` no longer owns the `FontRef/scaler/Render` pipeline
    directly
- The twenty-first internal `text/mod.rs` split has landed:
  - prepared-glyph image-to-raster mapping now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `render_prepared_glyph_raster(...)` now just composes image render and raster mapping
- The twenty-second internal `text/mod.rs` split has landed:
  - prepared-glyph atlas target selection now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `insert_prepared_glyph_raster(...)` now uses one shared atlas insertion path
- The twenty-third internal `text/mod.rs` split has landed:
  - prepared-glyph font-ref/scaler setup now lives behind dedicated helpers in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `render_prepared_glyph_image(...)` no longer owns `FontRef` and scaler construction directly
- The twenty-fourth internal `text/mod.rs` split has landed:
  - prepared-glyph subpixel offset construction now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `render_prepared_glyph_image(...)` no longer owns `offset_px` construction directly
- The twenty-fifth internal `text/mod.rs` split has landed:
  - prepared-glyph render invocation now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `render_prepared_glyph_image(...)` now just wires scaler setup, offset setup, and render
    invocation together
- The twenty-sixth internal `text/mod.rs` split has landed:
  - prepared-glyph atlas-hit bounds normalization now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `materialize_prepared_line(...)` no longer owns atlas-hit placement math directly
- The twenty-seventh internal `text/mod.rs` split has landed:
  - prepared-glyph bounds resolution now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `materialize_prepared_line(...)` no longer owns atlas lookup and miss fallback branching directly
- The twenty-eighth internal `text/mod.rs` split has landed:
  - prepared-glyph instance assembly now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `materialize_prepared_line(...)` no longer owns `GlyphInstance` rect normalization directly
- The twenty-ninth internal `text/mod.rs` split has landed:
  - prepared-glyph origin/bin quantization now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `materialize_prepared_line(...)` no longer owns `x/y` subpixel bin setup directly
- The thirtieth internal `text/mod.rs` split has landed:
  - prepared-glyph context assembly now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `materialize_prepared_line(...)` no longer owns `glyph_id/face_key/size_bits` setup directly
- The thirty-first internal `text/mod.rs` split has landed:
  - prepared-line per-glyph materialization now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `materialize_prepared_line(...)` now just iterates prepared glyphs and pushes returned instances
- The thirty-second internal `text/mod.rs` split has landed:
  - prepared-line glyph-drain materialization now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `materialize_prepared_line(...)` no longer owns the prepared-glyph loop directly
- The thirty-third internal `text/mod.rs` split has landed:
  - prepared-glyph face-key construction now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `register_prepared_glyph_face(...)` no longer owns `FontFaceKey` construction directly
- The thirty-fourth internal `text/mod.rs` split has landed:
  - prepared-glyph face-cache writes now live behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `register_prepared_glyph_face(...)` no longer owns font-data/coords cache writes directly
- The thirty-fifth internal `text/mod.rs` split has landed:
  - prepared-glyph face-usage accounting now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `register_prepared_glyph_face(...)` no longer owns face-usage counter updates directly
- The thirty-sixth internal `text/mod.rs` split has landed:
  - prepared-glyph per-kind atlas lookup now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `lookup_prepared_glyph_atlas(...)` no longer owns per-kind atlas lookup branches directly
- The thirty-seventh internal `text/mod.rs` split has landed:
  - prepared-glyph atlas lookup order now lives behind one shared constant in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `lookup_prepared_glyph_atlas(...)` now just iterates that order and short-circuits on hit
- The thirty-eighth internal `text/mod.rs` split has landed:
  - prepared-glyph atlas-entry fetch/pack now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `lookup_prepared_glyph_atlas_kind(...)` no longer owns atlas-entry fetch result packing directly
- The thirty-ninth internal `text/mod.rs` split has landed:
  - prepared-glyph atlas lookup-key construction now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `lookup_prepared_glyph_atlas_kind(...)` no longer owns lookup-key construction directly
- The fortieth internal `text/mod.rs` split has landed:
  - prepared-glyph raster commit now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `materialize_prepared_glyph_miss(...)` no longer owns raster-bounds/atlas-insert sequencing directly
- The forty-first internal `text/mod.rs` split has landed:
  - prepared-glyph atlas-hit bounds resolution now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `resolve_prepared_glyph_bounds(...)` no longer owns atlas-hit branching directly
- The forty-second internal `text/mod.rs` split has landed:
  - prepared-glyph raster metadata decoding now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_raster_from_image(...)` no longer owns content-kind/bytes-per-pixel mapping directly
- The forty-third internal `text/mod.rs` split has landed:
  - prepared-glyph render source selection now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `render_prepared_glyph_image_with_scaler(...)` no longer owns the swash source list literal directly
- The forty-fourth internal `text/mod.rs` split has landed:
  - prepared-glyph raster packing now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_raster_from_image(...)` no longer owns `PreparedGlyphRaster` field packing directly
- The forty-fifth internal `text/mod.rs` split has landed:
  - prepared-glyph normalized-coords injection now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `build_prepared_glyph_scaler(...)` no longer owns normalized-coords branching directly
- The forty-sixth internal `text/mod.rs` split has landed:
  - prepared-glyph raster placement field extraction now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_raster_from_image(...)` no longer owns zero-size placement guarding and field
    unpacking directly
- The forty-seventh internal `text/mod.rs` split has landed:
  - prepared-glyph atlas insertion argument packing now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `insert_prepared_glyph_raster(...)` no longer owns `GlyphAtlas::get_or_insert(...)` argument
    packing directly
- The forty-eighth internal `text/mod.rs` split has landed:
  - prepared-glyph image-part raster assembly now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_raster_from_image(...)` no longer owns content decode plus raster assembly
    directly
- The forty-ninth internal `text/mod.rs` split has landed:
  - prepared-glyph keyed raster packing now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_raster(...)` no longer owns keyed `PreparedGlyphRaster` field packing directly
- The fiftieth internal `text/mod.rs` split has landed:
  - prepared-glyph raster-key derivation now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_raster(...)` no longer shares the atlas-lookup key helper directly
- Surface inventory now exists and the first no-consumer facade shrink candidates are identified.
- Slice 1 verification is green:
  - `cargo nextest run -p fret-render -p fret-render-wgpu`: 221/221 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
- Internal text split verification is green:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
- The strongest current risks are:
  - wildcard facade exports,
  - oversized backend public surface,
  - a large multi-domain `Renderer` state owner,
  - and incomplete ergonomic closure for engine-hosted topology helpers.

---

## M0 — Problem and baseline locked

Exit criteria:

- The current public facade and backend export surface are inventoried.
- The current first-party consumers are grouped by usage pattern.
- Baseline gates are green and recorded in this workstream.
- Render-plan semantics are explicitly treated as fixed inputs to the refactor.

---

## M1 — Stable facade contract closed

Exit criteria:

- `crates/fret-render` no longer behaves like a wildcard backend dump.
- The intended stable default renderer surface is explicit.
- Portable value contracts have a clear ownership story.
- Callers can see which surfaces are "default facade contract" vs "backend-specific detail."

---

## M2 — Host-provided GPU topology becomes first-class

Exit criteria:

- Capability/bootstrap helpers no longer force `WgpuContext` as the only ergonomic entrypoint.
- Engine-hosted integration has at least one explicit smoke path or first-party example.
- Docs/examples clearly show both editor-hosted and engine-hosted topology entrypoints.

---

## M3 — Internal domains extracted behind stable semantics

Exit criteria:

- `text/mod.rs` is split into explicit subdomains.
- `Renderer` no longer directly owns every subdomain in one large state block.
- Service trait implementations remain readable and behavior-preserving.
- Conformance and render-plan semantics tests remain green.

Recommended first slice:

- text system breakup
- renderer state/domain breakup
- capability/bootstrap seam cleanup

---

## M4 — Public export tightening completed

Exit criteria:

- Low-value public exports have been reviewed and either justified or removed.
- Cache/registry/internal-only backend types are no longer public by default unless intentionally
  part of the stable story.
- First-party callers compile and run against the curated facade.

---

## M5 — Contract closure and cleanup

Exit criteria:

- Any required ADRs for renderer facade/topology changes have been added or updated.
- Workstream docs reflect the final stable story.
- We can explain, in one short page, which layer owns:
  - portable render contracts,
  - default facade compatibility,
  - backend-specific implementation details.

---

## M6 — Optional follow-up: deeper crate boundary changes

Exit criteria:

- A conscious decision exists on whether further crate splits are still needed.
- If yes, the next split is documented separately with scope and risks.
- If no, this workstream closes with modularity improvements delivered inside the existing crate
  layout.

This milestone is intentionally optional. It should only start after facade closure and internal
domain extraction have already lowered the risk surface.
