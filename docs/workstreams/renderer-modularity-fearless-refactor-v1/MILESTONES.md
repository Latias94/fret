# Renderer Modularity (Fearless Refactor v1) — Milestones

Status: In progress

Related:

- Purpose: `docs/workstreams/renderer-modularity-fearless-refactor-v1/README.md`
- Design: `docs/workstreams/renderer-modularity-fearless-refactor-v1/DESIGN.md`
- TODO: `docs/workstreams/renderer-modularity-fearless-refactor-v1/TODO.md`

Current snapshot (2026-03-13):

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
- The fifty-first internal `text/mod.rs` split has landed:
  - prepared-glyph synthesis skew normalization now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_face_key(...)` no longer owns skew clamp/default logic directly
- The fifty-second internal `text/mod.rs` split has landed:
  - prepared-glyph variation-key derivation now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_face_key(...)` no longer owns variation-key derivation directly
- The fifty-third internal `text/mod.rs` split has landed:
  - prepared-glyph synthesis embolden extraction now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_face_key(...)` no longer owns synthesis-embolden extraction directly
- The fifty-fourth internal `text/mod.rs` split has landed:
  - prepared-glyph font-data cache writes now live behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `cache_prepared_glyph_face_data(...)` no longer owns font-data entry writes directly
- The fifty-fifth internal `text/mod.rs` split has landed:
  - prepared-glyph instance-coords cache writes now live behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `cache_prepared_glyph_face_data(...)` no longer owns normalized-coords cache writes directly
- The fifty-sixth internal `text/mod.rs` split has landed:
  - prepared-glyph size-bit derivation now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepare_prepared_glyph_context(...)` no longer owns font-size bit packing directly
- The fifty-seventh internal `text/mod.rs` split has landed:
  - prepared-glyph id conversion now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepare_prepared_glyph_context(...)` no longer owns `glyph.id -> u16` conversion directly
- The fifty-eighth internal `text/mod.rs` split has landed:
  - prepared-glyph font identity derivation now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `register_prepared_glyph_face(...)` no longer owns `font_data_id/face_index` extraction directly
- The fifty-ninth internal `text/mod.rs` split has landed:
  - prepared-glyph context assembly now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepare_prepared_glyph_context(...)` no longer owns `PreparedGlyphContext` field packing directly
- The sixtieth internal `text/mod.rs` split has landed:
  - prepared-glyph atlas-hit fallback dispatch now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `resolve_prepared_glyph_bounds(...)` no longer owns atlas-hit vs miss-fallback branching directly
- The sixty-first internal `text/mod.rs` split has landed:
  - prepared-glyph bin-offset image render dispatch now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `render_prepared_glyph_image(...)` no longer owns bin-offset derivation plus scaler render call directly
- The sixty-second internal `text/mod.rs` split has landed:
  - prepared-glyph scaler size clamp now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `build_prepared_glyph_scaler(...)` no longer owns font-size clamp logic directly
- The sixty-third internal `text/mod.rs` split has landed:
  - prepared-glyph scaler builder assembly now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `build_prepared_glyph_scaler(...)` no longer owns scale-context builder chaining directly
- The sixty-fourth internal `text/mod.rs` split has landed:
  - prepared-glyph normalized-coords presence checks now live behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `cache_prepared_glyph_instance_coords(...)` and
    `apply_prepared_glyph_normalized_coords(...)` no longer own emptiness checks directly
- The sixty-fifth internal `text/mod.rs` split has landed:
  - prepared-glyph normalized-coords builder injection now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `apply_prepared_glyph_normalized_coords(...)` no longer owns `normalized_coords.iter()` builder injection directly
- The sixty-sixth internal `text/mod.rs` split has landed:
  - prepared-glyph scaler-builder normalized-coords assembly now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `build_prepared_glyph_scaler(...)` no longer owns normalized-coords builder assembly directly
- The sixty-seventh internal `text/mod.rs` split has landed:
  - prepared-glyph image rendering after font-ref resolution now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `render_prepared_glyph_image(...)` no longer owns scaler/render handoff after font-ref lookup directly
- The sixty-eighth internal `text/mod.rs` split has landed:
  - prepared-glyph image rendering after scaler construction now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `render_prepared_glyph_image_with_font_ref(...)` no longer owns scaler/render dispatch directly
- The sixty-ninth internal `text/mod.rs` split has landed:
  - prepared-glyph raster assembly after placement extraction now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_raster_from_image(...)` no longer owns placement-backed raster assembly directly
- The seventieth internal `text/mod.rs` split has landed:
  - prepared-glyph raster assembly after image-content metadata decoding now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_raster_from_image_parts(...)` no longer owns content-metadata-backed raster assembly directly
- The seventy-first internal `text/mod.rs` split has landed:
  - prepared-glyph raster assembly after image rendering now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `render_prepared_glyph_raster(...)` no longer owns image-to-raster handoff directly
- The seventy-second internal `text/mod.rs` split has landed:
  - prepared-glyph raster image handoff now uses a projected glyph-id value helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `render_prepared_glyph_raster_from_image(...)` no longer depends on the full glyph record
- The seventy-third internal `text/mod.rs` split has landed:
  - prepared-glyph raster payload handoff after metadata decoding now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_raster_from_image_parts_with_metadata(...)` no longer owns `image.data` payload handoff directly
- The seventy-fourth internal `text/mod.rs` split has landed:
  - prepared-glyph raster payload field assembly now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_raster(...)` no longer constructs raster payload fields inline before final assembly
- The seventy-fifth internal `text/mod.rs` split has landed:
  - prepared-glyph raster placement extraction now returns a dedicated placement struct in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_raster_from_image(...)` no longer carries raw placement tuples across the next helper boundary
- The seventy-sixth internal `text/mod.rs` split has landed:
  - prepared-glyph raster metadata decoding now returns a dedicated metadata struct in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_raster_from_image_parts(...)` no longer forwards raw `kind` and `bytes_per_pixel` values across the next helper boundary
- The seventy-seventh internal `text/mod.rs` split has landed:
  - prepared-glyph raster part assembly now forwards a dedicated placement struct into the metadata stage in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_raster_from_image_with_placement(...)` no longer reprojects placement into raw fields before entering the next helper
- The seventy-eighth internal `text/mod.rs` split has landed:
  - prepared-glyph raster assembly helpers now live in
    `crates/fret-render-wgpu/src/text/prepare/glyph_raster.rs`
  - `crates/fret-render-wgpu/src/text/prepare.rs` no longer hosts the raster data structs and low-level raster assembly chain inline
- The seventy-ninth internal `text/mod.rs` split has landed:
  - prepared-glyph atlas lookup and hit-or-miss bounds helpers now live in
    `crates/fret-render-wgpu/src/text/prepare/glyph_bounds.rs`
  - `crates/fret-render-wgpu/src/text/prepare.rs` no longer hosts the prepared-glyph atlas lookup chain inline
- The eightieth internal `text/mod.rs` split has landed:
  - prepared-glyph image and raster render helpers now live in
    `crates/fret-render-wgpu/src/text/prepare/glyph_render.rs`
  - `crates/fret-render-wgpu/src/text/prepare.rs` no longer hosts the prepared-glyph render/scaler chain inline
- The eighty-first internal `text/mod.rs` split has landed:
  - prepared-glyph face registration and context helpers now live in
    `crates/fret-render-wgpu/src/text/prepare/glyph_face.rs`
  - `crates/fret-render-wgpu/src/text/prepare.rs` no longer hosts the prepared-glyph face/context chain inline
- The eighty-second internal `text/mod.rs` split has landed:
  - font face metadata helpers now live in
    `crates/fret-render-wgpu/src/text/prepare/face_metadata.rs`
  - `crates/fret-render-wgpu/src/text/prepare.rs` no longer hosts the font trace and decoration-metrics read chain inline
- The eighty-third internal `text/mod.rs` split has landed:
  - prepared-glyph materialization helpers now live in
    `crates/fret-render-wgpu/src/text/prepare/glyph_materialize.rs`
  - `crates/fret-render-wgpu/src/text/prepare.rs` no longer hosts the prepared-glyph materialize and atlas-commit chain inline
- The eighty-fourth internal `text/mod.rs` split has landed:
  - prepare-shape build helpers now live in
    `crates/fret-render-wgpu/src/text/prepare/shape_build.rs`
  - `crates/fret-render-wgpu/src/text/prepare.rs` no longer hosts the prepare-shape begin/finish chain inline
- The eighty-fifth internal `text/mod.rs` split has landed:
  - prepare cache-flow helpers now live in
    `crates/fret-render-wgpu/src/text/prepare/cache_flow.rs`
  - `crates/fret-render-wgpu/src/text/prepare.rs` no longer hosts the blob/shape cache reuse and blob finalize chain inline
- The eighty-sixth internal `text/mod.rs` split has landed:
  - the live prepare-with-key driver now lives in
    `crates/fret-render-wgpu/src/text/prepare/driver.rs`
  - `crates/fret-render-wgpu/src/text/mod.rs` now delegates the live path through that driver
- The eighty-seventh internal `text/mod.rs` split has landed:
  - the temporary soft-rollback shim has been removed from
    `crates/fret-render-wgpu/src/text/mod.rs`
  - `prepare_with_key(...)` is now a thin delegation layer and
    `crates/fret-render-wgpu/src/text/prepare/driver.rs` fully owns the live prepare flow
- The eighty-eighth renderer shader split has landed:
  - the scale-nearest WGSL sources now live under
    `crates/fret-render-wgpu/src/renderer/pipelines/wgsl/*.wgsl`
  - `crates/fret-render-wgpu/src/renderer/shaders.rs` no longer hosts the scale-nearest shader
    family inline
- The eighty-ninth renderer shader split has landed:
  - the `color_adjust`, `color_matrix`, and `alpha_threshold` WGSL sources now live under
    `crates/fret-render-wgpu/src/renderer/pipelines/wgsl/*.wgsl`
  - `crates/fret-render-wgpu/src/renderer/shaders.rs` no longer hosts those fullscreen effect
    shader families inline
- The ninetieth renderer shader split has landed:
  - the `backdrop_warp` WGSL sources now live under
    `crates/fret-render-wgpu/src/renderer/pipelines/wgsl/{backdrop_warp,backdrop_warp_image,backdrop_warp_masked_part_a,backdrop_warp_masked_part_b,backdrop_warp_image_masked_part_a,backdrop_warp_image_masked_part_b,backdrop_warp_mask,backdrop_warp_image_mask}.wgsl`
  - `crates/fret-render-wgpu/src/renderer/shaders.rs` no longer hosts the `backdrop_warp`
    shader family inline
  - `crates/fret-render-wgpu/src/renderer/tests.rs` now validates the `backdrop_warp_image`
    shader variants explicitly during WGSL parse and WebGPU validation coverage
- The ninety-first renderer shader split has landed:
  - the `COMPOSITE_PREMUL` WGSL sources now live under
    `crates/fret-render-wgpu/src/renderer/pipelines/wgsl/{composite_premul,composite_premul_mask}.wgsl`
  - `crates/fret-render-wgpu/src/renderer/shaders.rs` no longer hosts the `COMPOSITE_PREMUL`
    shader pair inline
  - the existing WGSL parse/WebGPU validation coverage in
    `crates/fret-render-wgpu/src/renderer/tests.rs` continued to cover that shader pair without
    test-surface changes
- The ninety-second renderer shader split has landed:
  - the `VIEWPORT_SHADER` WGSL source now lives under
    `crates/fret-render-wgpu/src/renderer/pipelines/wgsl/viewport.wgsl`
  - `crates/fret-render-wgpu/src/renderer/shaders.rs` no longer hosts `VIEWPORT_SHADER` inline
  - the existing WGSL parse/WebGPU validation coverage in
    `crates/fret-render-wgpu/src/renderer/tests.rs` continued to cover `VIEWPORT_SHADER` without
    test-surface changes
- The ninety-third renderer shader split has landed:
  - the `MASK_SHADER` WGSL source now lives under
    `crates/fret-render-wgpu/src/renderer/pipelines/wgsl/mask.wgsl`
  - `crates/fret-render-wgpu/src/renderer/shaders.rs` no longer hosts `MASK_SHADER` inline
  - the existing WGSL parse/WebGPU validation coverage in
    `crates/fret-render-wgpu/src/renderer/tests.rs` continued to cover `MASK_SHADER` without
    test-surface changes
- The ninety-fourth renderer shader split has landed:
  - the `PATH_CLIP_MASK_SHADER` WGSL source now lives under
    `crates/fret-render-wgpu/src/renderer/pipelines/wgsl/path_clip_mask.wgsl`
  - `crates/fret-render-wgpu/src/renderer/shaders.rs` no longer hosts `PATH_CLIP_MASK_SHADER`
    inline
  - the existing WGSL parse/WebGPU validation coverage in
    `crates/fret-render-wgpu/src/renderer/tests.rs` continued to cover
    `PATH_CLIP_MASK_SHADER` without test-surface changes
- The ninety-fifth renderer shader split has landed:
  - the `QUAD_SHADER_PART_A/B` WGSL sources now live under
    `crates/fret-render-wgpu/src/renderer/pipelines/wgsl/{quad_part_a,quad_part_b}.wgsl`
  - `crates/fret-render-wgpu/src/renderer/shaders.rs` no longer hosts the quad shader envelope
    inline
  - the existing WGSL parse/WebGPU validation coverage in
    `crates/fret-render-wgpu/src/renderer/tests.rs` continued to cover the assembled
    `quad_shader_source()` output without test-surface changes
- The ninety-sixth renderer shader split has landed:
  - the `CLIP_MASK_SHADER_PART_A/B` WGSL sources now live under
    `crates/fret-render-wgpu/src/renderer/pipelines/wgsl/{clip_mask_part_a,clip_mask_part_b}.wgsl`
  - `crates/fret-render-wgpu/src/renderer/shaders.rs` no longer hosts the clip-mask shader
    envelope inline
  - the existing WGSL parse/WebGPU validation coverage in
    `crates/fret-render-wgpu/src/renderer/tests.rs` continued to cover the assembled
    `clip_mask_shader_source()` output without test-surface changes
- Surface inventory now exists and the first no-consumer facade shrink candidates are identified.
- Slice 1 verification is green:
  - `cargo nextest run -p fret-render -p fret-render-wgpu`: 221/221 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
- Internal text split verification is green:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
- Renderer shader split verification is green after the `backdrop_warp` externalization:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
  - `python3 tools/report_largest_files.py --top 30 --min-lines 800`: `renderer/shaders.rs`
    dropped from 5986 lines to 4934 lines and no longer appears in the top-30 oversized file
    report
- Renderer shader split verification is green after the `COMPOSITE_PREMUL` externalization:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
  - `python3 tools/report_largest_files.py --top 30 --min-lines 800`: `renderer/shaders.rs`
    dropped from 4934 lines to 4350 lines while staying out of the top-30 oversized file report
- Renderer shader split verification is green after the `VIEWPORT_SHADER` externalization:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
  - `python3 tools/report_largest_files.py --top 30 --min-lines 800`: `renderer/shaders.rs`
    dropped from 4350 lines to 4021 lines while staying out of the top-30 oversized file report
- Renderer shader split verification is green after the `MASK_SHADER` externalization:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
  - `python3 tools/report_largest_files.py --top 30 --min-lines 800`: `renderer/shaders.rs`
    dropped from 4021 lines to 3701 lines while staying out of the top-30 oversized file report
- Renderer shader split verification is green after the `PATH_CLIP_MASK_SHADER`
  externalization:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
  - `python3 tools/report_largest_files.py --top 30 --min-lines 800`: `renderer/shaders.rs`
    dropped from 3701 lines to 3666 lines while staying out of the top-30 oversized file report
- Renderer shader split verification is green after the `QUAD_SHADER_PART_A/B`
  externalization:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
  - `python3 tools/report_largest_files.py --top 30 --min-lines 800`: `renderer/shaders.rs`
    dropped from 3666 lines to 2773 lines while staying out of the top-30 oversized file report
- Renderer shader split verification is green after the `CLIP_MASK_SHADER_PART_A/B`
  externalization:
  - `cargo nextest run -p fret-render-wgpu`: 220 passed (1 leaky), exit code 0
  - `cargo nextest run -p fret-render-wgpu text::tests::text_measure_key_ignores_width_for_wrap_none`:
    passed on targeted rerun
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
  - `python3 tools/report_largest_files.py --top 30 --min-lines 800`: `renderer/shaders.rs`
    dropped from 2773 lines to 2685 lines while staying out of the top-30 oversized file report
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
