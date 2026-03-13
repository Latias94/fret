# Renderer Modularity (Fearless Refactor v1)

Status: Draft

Last updated: 2026-03-13

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

As of 2026-03-13:

- `crates/fret-render/src/lib.rs` now uses an explicit curated re-export list instead of a
  wildcard backend dump.
- `crates/fret-render-wgpu/src/lib.rs` re-exports a broad mix of:
  - stable-facing contracts,
  - backend bootstrap helpers,
  - diagnostics stores,
  - and types that may not need to stay public.
- `crates/fret-render-wgpu/src/renderer/mod.rs` still defines a large `Renderer` state owner.
- `crates/fret-render-wgpu/src/text/mod.rs` and `crates/fret-render-wgpu/src/renderer/shaders.rs`
  are the most obvious oversized internal modules.
- `Renderer::new(adapter, device)` and `render_scene(device, queue, ...)` already make
  host-provided GPU objects possible, and
  `crates/fret-render-wgpu/tests/host_provided_gpu_topology_smoke.rs` now locks that engine-hosted
  seam with a direct smoke path.
- The public teaching surface now shows both topology entrypoints in:
  - `crates/fret-render/src/lib.rs`
  - `crates/fret-render-wgpu/src/lib.rs`
  - `docs/crate-usage-guide.md`
- The stable v1 default-facade buckets are now explicit in:
  - `docs/workstreams/renderer-modularity-fearless-refactor-v1/SURFACE_INVENTORY.md`
  - `crates/fret-render/tests/facade_surface_snapshot.rs`
- The default facade no longer re-exports nested diagnostics/detail leaf structs that had zero
  first-party consumers; those now stay backend-specific.
- The default facade also no longer re-exports zero-direct-consumer advanced perf/init value
  snapshots; diagnostics/report stores remain, but deeper value types now require
  `fret-render-wgpu` directly.
- The portable-value ownership audit is also closed for v1:
  - render-target metadata already lives in `fret-render-core`
  - no additional backend-owned value type move improved ownership clarity after audit
- `WgpuContext` remains on the stable default facade as the supported convenience/bootstrap path
  for Fret-owned GPU initialization, but the docs and gates now make clear that engine-hosted
  adapter/device flows are equally first-class.
- Shared text type shells now live under `crates/fret-render-wgpu/src/text/types.rs`, and
  `text/mod.rs` no longer owns glyph/blob/shape/helper type definitions directly.
- Text bootstrap assembly now lives under `crates/fret-render-wgpu/src/text/bootstrap.rs`, and
  `TextSystem::new(...)` now delegates initial state assembly through that bootstrap module.
- Initial font-policy bootstrap finalization now lives under
  `crates/fret-render-wgpu/src/text/fonts.rs`, and `TextSystem::new(...)` no longer owns
  fallback-policy/font-stack finalization directly.
- Public `TextSystem::new(...)` now lives under `crates/fret-render-wgpu/src/text/bootstrap.rs`,
  private `prepare_with_key(...)` glue now lives under
  `crates/fret-render-wgpu/src/text/prepare.rs`, and `text/mod.rs` now keeps only the text state
  shell plus module wiring.
- Per-frame text perf state now lives under `crates/fret-render-wgpu/src/text/frame_perf.rs`, and
  `text/mod.rs` no longer owns the per-frame text perf counter fields directly.
- Text face-cache state now lives under `crates/fret-render-wgpu/src/text/face_cache.rs`, and
  `text/mod.rs` no longer owns font-data / instance-coords / family-name cache fields directly.
- Text pin-ring state now lives under `crates/fret-render-wgpu/src/text/pin_state.rs`, and
  `text/mod.rs` no longer owns scene pin-ring bucket fields directly.
- Text blob/cache state now lives under `crates/fret-render-wgpu/src/text/blob_state.rs`, and
  `text/mod.rs` no longer owns blob-cache/LRU state fields directly.
- Text atlas epoch state now lives under `crates/fret-render-wgpu/src/text/atlas_epoch.rs`, and
  `text/mod.rs` no longer owns the raw glyph-atlas epoch field directly.
- Text atlas runtime state now lives under
  `crates/fret-render-wgpu/src/text/atlas_runtime_state.rs`, and `text/mod.rs` no longer owns
  atlas textures/bind-group-layout fields directly.
- Text layout-cache state now lives under
  `crates/fret-render-wgpu/src/text/layout_cache_state.rs`, and `text/mod.rs` no longer owns
  shape-cache/measure-cache fields directly.
- Text font-runtime state now lives under
  `crates/fret-render-wgpu/src/text/font_runtime_state.rs`, and `text/mod.rs` no longer owns
  font-stack key / font-db revision / fallback-policy / generic-injection / font-trace fields
  directly.
- Text atlas `TextSystem` flow now lives under
  `crates/fret-render-wgpu/src/text/atlas_flow.rs`, and `crates/fret-render-wgpu/src/text/atlas.rs`
  no longer owns atlas bind-group access, upload flushing, scene pinning, or glyph ensure glue
  directly.
- Built-in renderer effect helper flow now lives under
  `crates/fret-render-wgpu/src/renderer/render_plan_effects/builtin.rs`, and
  `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs` no longer owns built-in effect
  budget gates, clip-mask target choice, or single-scratch/two-scratch pass-builder helpers
  directly.
- Renderer blur planning helper flow now lives under
  `crates/fret-render-wgpu/src/renderer/render_plan_effects/blur.rs`, and
  `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs` no longer owns blur compile,
  scissor inflation, or padded chain-scissor derivation helpers directly.
- Renderer custom-step apply flow now lives under
  `crates/fret-render-wgpu/src/renderer/render_plan_effects/custom.rs`, and
  `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs` no longer owns custom effect
  V1/V2/V3 step-apply branch handling directly.
- Renderer backdrop step-apply flow now also lives under
  `crates/fret-render-wgpu/src/renderer/render_plan_effects/builtin.rs`, and
  `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs` no longer owns
  `BackdropWarpV1`/`BackdropWarpV2` step-apply branch handling directly.
- Renderer simple built-in step-apply flow now also lives under
  `crates/fret-render-wgpu/src/renderer/render_plan_effects/builtin.rs`, and
  `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs` no longer owns
  `NoiseV1`, `ColorAdjust`, `ColorMatrix`, `AlphaThreshold`, `Pixelate`, or `Dither`
  step-apply branch handling directly.
- Renderer masked chain builtin/backdrop step-apply flow now also lives under
  `crates/fret-render-wgpu/src/renderer/render_plan_effects/builtin.rs`, and
  `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs` no longer owns masked
  `apply_chain_in_place(...)` branch handling for `BackdropWarpV1`/`BackdropWarpV2`, `NoiseV1`,
  `ColorAdjust`, `ColorMatrix`, `AlphaThreshold`, `Pixelate`, or `Dither` directly.
- Renderer masked GaussianBlur chain compile flow now also lives under
  `crates/fret-render-wgpu/src/renderer/render_plan_effects/blur.rs`, and
  `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs` no longer owns masked
  `apply_chain_in_place(...)` branch handling for `GaussianBlur` directly.
- Renderer masked DropShadow chain compile flow now also lives under
  `crates/fret-render-wgpu/src/renderer/render_plan_effects/blur.rs`, and
  `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs` no longer owns masked
  `apply_chain_in_place(...)` branch handling for `DropShadowV1` directly.
- Renderer masked custom chain step-apply flow now also lives under
  `crates/fret-render-wgpu/src/renderer/render_plan_effects/custom.rs`, and
  `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs` no longer owns masked
  `apply_chain_in_place(...)` branch handling for `CustomV1`/`CustomV2`/`CustomV3` directly.
- Renderer padded-chain orchestration flow now also lives under
  `crates/fret-render-wgpu/src/renderer/render_plan_effects/chain.rs`, and
  `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs` no longer owns the padded
  work-buffer / optional raw-target / final-commit orchestration inside `apply_chain_in_place(...)`
  directly.
- Renderer padded-chain final Custom commit helpers now also live under
  `crates/fret-render-wgpu/src/renderer/render_plan_effects/custom.rs`, keeping effect-specific
  final-step wiring out of the orchestration module.
- Renderer chain-start preparation flow now also lives under
  `crates/fret-render-wgpu/src/renderer/render_plan_effects/chain.rs`, and
  `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs` no longer owns custom-chain budget
  initialization, scratch-target inventory, forced quarter-blur mask-tier choice, or clip-mask
  budget charging directly.
- Renderer unpadded chain driver flow now also lives under
  `crates/fret-render-wgpu/src/renderer/render_plan_effects/chain.rs`, and
  `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs` no longer owns unpadded raw-target
  reservation, final-step mask handoff, or masked step dispatch inside `apply_chain_in_place(...)`
  directly.
- Renderer unmasked chain-step dispatch flow now also lives under
  `crates/fret-render-wgpu/src/renderer/render_plan_effects/chain.rs`, and
  `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs` no longer owns
  `apply_step_in_place_with_scratch_targets(...)` directly.
- Renderer shared chain helper flow now also lives under
  `crates/fret-render-wgpu/src/renderer/render_plan_effects/chain.rs`, and
  `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs` no longer owns scratch-target
  inventory, custom-step/raw-source detection, or backdrop-source-group decomposition helpers
  directly.
- Renderer top-level chain driver flow now also lives under
  `crates/fret-render-wgpu/src/renderer/render_plan_effects/chain.rs`, and
  `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs` now keeps
  `apply_chain_in_place(...)` only as a thin forwarding surface.
- Renderer custom-effect v3 pyramid owner state now also lives under
  `crates/fret-render-wgpu/src/renderer/v3_pyramid.rs`, and
  `crates/fret-render-wgpu/src/renderer/mod.rs` no longer owns the pyramid scratch/cache/write-epoch
  fields or helper methods directly.
- Renderer SVG raster/atlas/perf owner state now also lives under
  `crates/fret-render-wgpu/src/renderer/svg/mod.rs`, and
  `crates/fret-render-wgpu/src/renderer/mod.rs` no longer owns SVG raster cache, atlas storage,
  budget/epoch, or per-frame SVG cache counter fields directly.
- Renderer intermediate budget/perf/pool owner state now also lives under
  `crates/fret-render-wgpu/src/renderer/intermediate_pool.rs`, and
  `crates/fret-render-wgpu/src/renderer/mod.rs` no longer owns intermediate budget/perf/pool
  fields directly.
- Renderer SVG registry/service owner state now also lives under
  `crates/fret-render-wgpu/src/renderer/svg/mod.rs`, and
  `crates/fret-render-wgpu/src/renderer/mod.rs` no longer owns `svg_renderer`, `svgs`, or
  `svg_hash_index` directly.
- Renderer diagnostics/perf owner state now also lives under
  `crates/fret-render-wgpu/src/renderer/diagnostics.rs`, and
  `crates/fret-render-wgpu/src/renderer/mod.rs` no longer owns render perf enablement, pending
  render-target ingest counters, last-frame perf snapshots, render-plan segment history, or render
  scene frame index directly.
- Renderer material/custom-effect runtime owner state now also lives under
  `crates/fret-render-wgpu/src/renderer/material_effects.rs`, and
  `crates/fret-render-wgpu/src/renderer/mod.rs` no longer owns material registries, material
  budgets, custom-effect registries, or their generation counters directly.
- Renderer path registry/cache owner state now also lives under
  `crates/fret-render-wgpu/src/renderer/path.rs`, and
  `crates/fret-render-wgpu/src/renderer/mod.rs` no longer owns prepared path storage, path cache
  entries, path cache capacity, or path cache epoch directly.
- Renderer path intermediate/composite scratch owner state now also lives under
  `crates/fret-render-wgpu/src/renderer/path.rs`, and
  `crates/fret-render-wgpu/src/renderer/mod.rs` no longer owns path intermediate attachments, path
  composite vertex storage, or path composite vertex capacity directly.
- Renderer render-plan reporting/dump owner state now also lives under
  `crates/fret-render-wgpu/src/renderer/render_plan_reporting.rs`, and
  `crates/fret-render-wgpu/src/renderer/mod.rs` no longer owns render-plan segment report scratch,
  per-segment pass-count scratch, or render-plan JSON dump scratch directly.
- Some convenience/diagnostics surfaces still privilege `WgpuContext`, so ergonomic closure is not
  fully finished yet.
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
- The fifty-third internal text split has landed:
  - prepared-glyph synthesis embolden extraction now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_face_key(...)` no longer owns synthesis-embolden extraction directly
- The fifty-fourth internal text split has landed:
  - prepared-glyph font-data cache writes now live behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `cache_prepared_glyph_face_data(...)` no longer owns font-data entry writes directly
- The fifty-fifth internal text split has landed:
  - prepared-glyph instance-coords cache writes now live behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `cache_prepared_glyph_face_data(...)` no longer owns normalized-coords cache writes directly
- The fifty-sixth internal text split has landed:
  - prepared-glyph size-bit derivation now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepare_prepared_glyph_context(...)` no longer owns font-size bit packing directly
- The fifty-seventh internal text split has landed:
  - prepared-glyph id conversion now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepare_prepared_glyph_context(...)` no longer owns `glyph.id -> u16` conversion directly
- The fifty-eighth internal text split has landed:
  - prepared-glyph font identity derivation now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `register_prepared_glyph_face(...)` no longer owns `font_data_id/face_index` extraction directly
- The fifty-ninth internal text split has landed:
  - prepared-glyph context assembly now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepare_prepared_glyph_context(...)` no longer owns `PreparedGlyphContext` field packing directly
- The sixtieth internal text split has landed:
  - prepared-glyph atlas-hit fallback dispatch now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `resolve_prepared_glyph_bounds(...)` no longer owns atlas-hit vs miss-fallback branching directly
- The sixty-first internal text split has landed:
  - prepared-glyph bin-offset image render dispatch now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `render_prepared_glyph_image(...)` no longer owns bin-offset derivation plus scaler render call directly
- The latest internal text state-shell tightening slice has landed:
  - per-frame text perf state now lives under
    `crates/fret-render-wgpu/src/text/frame_perf.rs`
  - `text/mod.rs` no longer owns the per-frame text perf counter fields directly
- The latest internal text state-shell tightening slice has also moved:
  - text face-cache state under
    `crates/fret-render-wgpu/src/text/face_cache.rs`
  - `text/mod.rs` no longer owns font-data / instance-coords / family-name cache fields directly
- The latest internal text state-shell tightening slice has also moved:
  - text pin-ring state under
    `crates/fret-render-wgpu/src/text/pin_state.rs`
  - `text/mod.rs` no longer owns scene pin-ring bucket fields directly
- The latest internal text state-shell tightening slice has also moved:
  - text blob/cache state under
    `crates/fret-render-wgpu/src/text/blob_state.rs`
  - `text/mod.rs` no longer owns blob-cache/LRU state fields directly
- The latest internal text state-shell tightening slice has also moved:
  - text atlas epoch state under
    `crates/fret-render-wgpu/src/text/atlas_epoch.rs`
  - `text/mod.rs` no longer owns the raw glyph-atlas epoch field directly
- The latest internal text state-shell tightening slice has also moved:
  - text atlas runtime state under
    `crates/fret-render-wgpu/src/text/atlas_runtime_state.rs`
  - `text/mod.rs` no longer owns atlas textures/bind-group-layout fields directly
- The latest internal text state-shell tightening slice has also moved:
  - text layout-cache state under
    `crates/fret-render-wgpu/src/text/layout_cache_state.rs`
  - `text/mod.rs` no longer owns shape-cache/measure-cache fields directly
- The latest internal text state-shell tightening slice has also moved:
  - text font-runtime state under
    `crates/fret-render-wgpu/src/text/font_runtime_state.rs`
  - `text/mod.rs` no longer owns font-stack key / font-db revision / fallback-policy /
    generic-injection / font-trace fields directly
- The sixty-second internal text split has landed:
  - prepared-glyph scaler size clamp now lives behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `build_prepared_glyph_scaler(...)` no longer owns font-size clamp logic directly
- The sixty-third internal text split has landed:
  - prepared-glyph scaler builder assembly now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `build_prepared_glyph_scaler(...)` no longer owns scale-context builder chaining directly
- The sixty-fourth internal text split has landed:
  - atlas `TextSystem` flow now lives under
    `crates/fret-render-wgpu/src/text/atlas_flow.rs`
  - `crates/fret-render-wgpu/src/text/atlas.rs` no longer owns atlas bind-group access, upload
    flushing, scene pinning, or glyph ensure glue directly
- The first renderer effect-planning split has landed:
  - built-in effect helper flow now lives under
    `crates/fret-render-wgpu/src/renderer/render_plan_effects/builtin.rs`
  - `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs` no longer owns built-in effect
    budget gates, clip-mask target choice, or single-scratch/two-scratch pass-builder helpers
    directly
- The second renderer effect-planning split has landed:
  - blur planning helper flow now lives under
    `crates/fret-render-wgpu/src/renderer/render_plan_effects/blur.rs`
  - `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs` no longer owns blur compile,
    scissor inflation, or padded chain-scissor derivation helpers directly
- The third renderer effect-planning split has landed:
  - custom-step apply flow now lives under
    `crates/fret-render-wgpu/src/renderer/render_plan_effects/custom.rs`
  - `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs` no longer owns custom effect
    V1/V2/V3 step-apply branch handling directly
- The sixty-fourth internal text split has landed:
  - prepared-glyph normalized-coords presence checks now live behind a pure helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `cache_prepared_glyph_instance_coords(...)` and
    `apply_prepared_glyph_normalized_coords(...)` no longer own emptiness checks directly
- The sixty-fifth internal text split has landed:
  - prepared-glyph normalized-coords builder injection now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `apply_prepared_glyph_normalized_coords(...)` no longer owns `normalized_coords.iter()` builder injection directly
- The sixty-sixth internal text split has landed:
  - prepared-glyph scaler-builder normalized-coords assembly now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `build_prepared_glyph_scaler(...)` no longer owns normalized-coords builder assembly directly
- The sixty-seventh internal text split has landed:
  - prepared-glyph image rendering after font-ref resolution now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `render_prepared_glyph_image(...)` no longer owns scaler/render handoff after font-ref lookup directly
- The sixty-eighth internal text split has landed:
  - prepared-glyph image rendering after scaler construction now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `render_prepared_glyph_image_with_font_ref(...)` no longer owns scaler/render dispatch directly
- The sixty-ninth internal text split has landed:
  - prepared-glyph raster assembly after placement extraction now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_raster_from_image(...)` no longer owns placement-backed raster assembly directly
- The seventieth internal text split has landed:
  - prepared-glyph raster assembly after image-content metadata decoding now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_raster_from_image_parts(...)` no longer owns content-metadata-backed raster assembly directly
- The seventy-first internal text split has landed:
  - prepared-glyph raster assembly after image rendering now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `render_prepared_glyph_raster(...)` no longer owns image-to-raster handoff directly
- The seventy-second internal text split has landed:
  - prepared-glyph raster image handoff now uses a projected glyph-id value helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `render_prepared_glyph_raster_from_image(...)` no longer depends on the full glyph record
- The seventy-third internal text split has landed:
  - prepared-glyph raster payload handoff after metadata decoding now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_raster_from_image_parts_with_metadata(...)` no longer owns `image.data` payload handoff directly
- The seventy-fourth internal text split has landed:
  - prepared-glyph raster payload field assembly now lives behind a dedicated helper in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_raster(...)` no longer constructs raster payload fields inline before final assembly
- The seventy-fifth internal text split has landed:
  - prepared-glyph raster placement extraction now returns a dedicated placement struct in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_raster_from_image(...)` no longer carries raw placement tuples across the next helper boundary
- The seventy-sixth internal text split has landed:
  - prepared-glyph raster metadata decoding now returns a dedicated metadata struct in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_raster_from_image_parts(...)` no longer forwards raw `kind` and `bytes_per_pixel` values across the next helper boundary
- The seventy-seventh internal text split has landed:
  - prepared-glyph raster part assembly now forwards a dedicated placement struct into the metadata stage in
    `crates/fret-render-wgpu/src/text/prepare.rs`
  - `prepared_glyph_raster_from_image_with_placement(...)` no longer reprojects placement into raw fields before entering the next helper
- The seventy-eighth internal text split has landed:
  - prepared-glyph raster assembly helpers now live in
    `crates/fret-render-wgpu/src/text/prepare/glyph_raster.rs`
  - `crates/fret-render-wgpu/src/text/prepare.rs` no longer hosts the raster data structs and low-level raster assembly chain inline
- The seventy-ninth internal text split has landed:
  - prepared-glyph atlas lookup and hit-or-miss bounds helpers now live in
    `crates/fret-render-wgpu/src/text/prepare/glyph_bounds.rs`
  - `crates/fret-render-wgpu/src/text/prepare.rs` no longer hosts the prepared-glyph atlas lookup chain inline
- The eightieth internal text split has landed:
  - prepared-glyph image and raster render helpers now live in
    `crates/fret-render-wgpu/src/text/prepare/glyph_render.rs`
  - `crates/fret-render-wgpu/src/text/prepare.rs` no longer hosts the prepared-glyph render/scaler chain inline
- The eighty-first internal text split has landed:
  - prepared-glyph face registration and context helpers now live in
    `crates/fret-render-wgpu/src/text/prepare/glyph_face.rs`
  - `crates/fret-render-wgpu/src/text/prepare.rs` no longer hosts the prepared-glyph face/context chain inline
- The eighty-second internal text split has landed:
  - font face metadata helpers now live in
    `crates/fret-render-wgpu/src/text/prepare/face_metadata.rs`
  - `crates/fret-render-wgpu/src/text/prepare.rs` no longer hosts the font trace and decoration-metrics read chain inline
- The eighty-third internal text split has landed:
  - prepared-glyph materialization helpers now live in
    `crates/fret-render-wgpu/src/text/prepare/glyph_materialize.rs`
  - `crates/fret-render-wgpu/src/text/prepare.rs` no longer hosts the prepared-glyph materialize and atlas-commit chain inline
- The eighty-fourth internal text split has landed:
  - prepare-shape build helpers now live in
    `crates/fret-render-wgpu/src/text/prepare/shape_build.rs`
  - `crates/fret-render-wgpu/src/text/prepare.rs` no longer hosts the prepare-shape begin/finish chain inline
- The eighty-fifth internal text split has landed:
  - prepare cache-flow helpers now live in
    `crates/fret-render-wgpu/src/text/prepare/cache_flow.rs`
  - `crates/fret-render-wgpu/src/text/prepare.rs` no longer hosts the blob/shape cache reuse and blob finalize chain inline
- The eighty-sixth internal text split has landed:
  - the live prepare-with-key driver now lives in
    `crates/fret-render-wgpu/src/text/prepare/driver.rs`
  - `crates/fret-render-wgpu/src/text/mod.rs` now delegates the live path through that driver
- The eighty-seventh internal text split has landed:
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
- The ninety-seventh renderer shader split has landed:
  - the `TEXT_COLOR_SHADER` WGSL source now lives under
    `crates/fret-render-wgpu/src/renderer/pipelines/wgsl/text_color.wgsl`
  - `crates/fret-render-wgpu/src/renderer/shaders.rs` no longer hosts `TEXT_COLOR_SHADER` inline
  - the existing WGSL parse/WebGPU validation coverage in
    `crates/fret-render-wgpu/src/renderer/tests.rs` continued to cover `TEXT_COLOR_SHADER`
    without test-surface changes
- The ninety-eighth renderer shader split has landed:
  - the `TEXT_SUBPIXEL_SHADER` WGSL source now lives under
    `crates/fret-render-wgpu/src/renderer/pipelines/wgsl/text_subpixel.wgsl`
  - `crates/fret-render-wgpu/src/renderer/shaders.rs` no longer hosts `TEXT_SUBPIXEL_SHADER`
    inline
  - the existing WGSL parse/WebGPU validation coverage in
    `crates/fret-render-wgpu/src/renderer/tests.rs` continued to cover `TEXT_SUBPIXEL_SHADER`
    without test-surface changes
- The ninety-ninth renderer shader split has landed:
  - the `TEXT_SHADER` WGSL source now lives under
    `crates/fret-render-wgpu/src/renderer/pipelines/wgsl/text.wgsl`
  - `crates/fret-render-wgpu/src/renderer/shaders.rs` no longer hosts `TEXT_SHADER` inline
  - the existing WGSL parse/WebGPU validation coverage in
    `crates/fret-render-wgpu/src/renderer/tests.rs` continued to cover `TEXT_SHADER` without
    test-surface changes
- The one-hundredth renderer shader split has landed:
  - the `PATH_SHADER` WGSL source now lives under
    `crates/fret-render-wgpu/src/renderer/pipelines/wgsl/path.wgsl`
  - `crates/fret-render-wgpu/src/renderer/shaders.rs` no longer hosts `PATH_SHADER` inline
  - the existing WGSL parse/WebGPU validation coverage in
    `crates/fret-render-wgpu/src/renderer/tests.rs` continued to cover `PATH_SHADER` without
    test-surface changes
  - the existing local naga validation test in
    `crates/fret-render-wgpu/src/renderer/shaders.rs` continued to validate `PATH_SHADER`
    unchanged
- The first backend-root export tightening slice has landed:
  - `crates/fret-render-wgpu/src/lib.rs` no longer re-exports zero-first-party-consumer backend
    helpers at the crate root
  - removed root re-exports:
    `ImageRegistry`, `RenderTargetRegistry`, `CachedSvgImage`, `SvgImageCache`,
    `SvgRasterKind`, `SvgRenderer`, and `SMOOTH_SVG_SCALE_FACTOR`
  - existing first-party runner/demo/facade call sites required no source changes because none of
    those names were consumed outside `crates/fret-render-wgpu`
  - this slice also exposed `crates/fret-render-wgpu/src/svg_cache.rs` as a detached legacy path:
    it currently has no active first-party consumers and only surfaces as dead-code follow-up
- The legacy SVG cache retirement slice has landed:
  - `crates/fret-render-wgpu/src/svg_cache.rs` is no longer part of the backend compile path
  - `crates/fret-render-wgpu/src/svg.rs` now keeps only the internal fit-mode renderer entrypoints
    still used by active SVG raster flow
  - the detached app-owned `SvgImageCache` path is now explicitly retired instead of silently
    surviving as dead code
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
- Renderer shader split verification remains green after the `backdrop_warp` externalization:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
  - `python3 tools/report_largest_files.py --top 30 --min-lines 800`: `renderer/shaders.rs`
    dropped from 5986 lines to 4934 lines and no longer appears in the top-30 oversized file
    report
- Renderer shader split verification remains green after the `COMPOSITE_PREMUL` externalization:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
  - `python3 tools/report_largest_files.py --top 30 --min-lines 800`: `renderer/shaders.rs`
    dropped from 4934 lines to 4350 lines while staying out of the top-30 oversized file report
- Renderer shader split verification remains green after the `VIEWPORT_SHADER` externalization:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
  - `python3 tools/report_largest_files.py --top 30 --min-lines 800`: `renderer/shaders.rs`
    dropped from 4350 lines to 4021 lines while staying out of the top-30 oversized file report
- Renderer shader split verification remains green after the `MASK_SHADER` externalization:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
  - `python3 tools/report_largest_files.py --top 30 --min-lines 800`: `renderer/shaders.rs`
    dropped from 4021 lines to 3701 lines while staying out of the top-30 oversized file report
- Renderer shader split verification remains green after the `PATH_CLIP_MASK_SHADER`
  externalization:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
  - `python3 tools/report_largest_files.py --top 30 --min-lines 800`: `renderer/shaders.rs`
    dropped from 3701 lines to 3666 lines while staying out of the top-30 oversized file report
- Renderer shader split verification remains green after the `QUAD_SHADER_PART_A/B`
  externalization:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
  - `python3 tools/report_largest_files.py --top 30 --min-lines 800`: `renderer/shaders.rs`
    dropped from 3666 lines to 2773 lines while staying out of the top-30 oversized file report
- Renderer shader split verification remains green after the `CLIP_MASK_SHADER_PART_A/B`
  externalization:
  - `cargo nextest run -p fret-render-wgpu`: 220 passed (1 leaky), exit code 0
  - `cargo nextest run -p fret-render-wgpu text::tests::text_measure_key_ignores_width_for_wrap_none`:
    passed on targeted rerun
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
  - `python3 tools/report_largest_files.py --top 30 --min-lines 800`: `renderer/shaders.rs`
    dropped from 2773 lines to 2685 lines while staying out of the top-30 oversized file report
- Renderer shader split verification remains green after the `TEXT_COLOR_SHADER`
  externalization:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
  - `python3 tools/report_largest_files.py --top 30 --min-lines 800`: `renderer/shaders.rs`
    dropped from 2685 lines to 2205 lines while staying out of the top-30 oversized file report
- Renderer shader split verification remains green after the `TEXT_SUBPIXEL_SHADER`
  externalization:
  - `cargo nextest run -p fret-render-wgpu`: 220 passed (1 leaky), exit code 0
  - `cargo nextest run -p fret-render-wgpu renderer::render_plan::tests::blur_scissor_is_mapped_per_pass_dst_size`:
    passed on targeted rerun
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
  - `python3 tools/report_largest_files.py --top 30 --min-lines 800`: `renderer/shaders.rs`
    dropped from 2205 lines to 1592 lines while staying out of the top-30 oversized file report
- Renderer shader split verification remains green after the `TEXT_SHADER` externalization:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
  - `python3 tools/report_largest_files.py --top 30 --min-lines 800`: `renderer/shaders.rs`
    dropped from 1592 lines to 981 lines while staying out of the top-30 oversized file report
- Renderer shader split verification remains green after the `PATH_SHADER` externalization:
  - `cargo nextest run -p fret-render-wgpu`: 220/220 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
  - `python3 tools/report_largest_files.py --top 30 --min-lines 800`: `renderer/shaders.rs`
    dropped from 981 lines to 331 lines while staying out of the top-30 oversized file report
  - `rg -n '= r#"' crates/fret-render-wgpu/src/renderer/shaders.rs`: no inline raw WGSL blocks remain
- Backend-root export tightening verification remains green after removing the zero-consumer
  re-exports:
  - `cargo nextest run -p fret-render -p fret-render-wgpu`: passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
  - residual note: `fret-render-wgpu` now reports dead-code warnings around the legacy
    `svg_cache.rs` helper path, confirming it is no longer on an active first-party route
- Legacy SVG cache retirement verification remains green after removing `svg_cache.rs` from the
  compile path:
  - `cargo nextest run -p fret-render -p fret-render-wgpu`: 221/221 passed
  - `cargo check -p fret-launch -p fret-examples`: passed
  - `python3 tools/check_layering.py`: passed
  - `fret-render-wgpu` no longer emits dead-code warnings for the retired `svg_cache.rs` path
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
