# Renderer Modularity (Fearless Refactor v1) — Closeout Audit

Status: Closed for v1

Last updated: 2026-03-13

## Scope

This audit closes the remaining tracker items that were still open or in progress after the
finishing/shader stop-point audits:

- text extraction closure
- renderer-owner closure
- export tightening closure
- gate verification closure
- docs / ADR follow-up closure
- cleanup / exit closure

## Findings

### 1. `text/mod.rs` reached the intended boring v1 shape

Current shape:

- `crates/fret-render-wgpu/src/text/mod.rs` is now an 84-line state shell plus explicit module
  wiring
- extracted text owners/helpers now live under:
  - `crates/fret-render-wgpu/src/text/*.rs`
  - `crates/fret-render-wgpu/src/text/prepare/*.rs`
- `crates/fret-render-text` remains the low-level text contract crate and still owns shaping,
  measurement, fallback-policy, span, and layout helper logic rather than backend runtime state

Evidence:

- `crates/fret-render-wgpu/src/text/mod.rs`
- `crates/fret-render-wgpu/src/text/tests.rs`
- `crates/fret-render-text/src/lib.rs`

Assessment:

- the original oversized-module risk is gone
- backend-specific text runtime state did not drift into `crates/fret-render-text`
- extracted text subdomains now have focused verification instead of relying only on the original
  monolithic file coverage

Decision:

- close `RMFR-text-030` as done
- close `RMFR-text-031` as done
- close `RMFR-text-032` as done

### 2. Renderer-owner and export-tightening goals are complete for v1

Current shape:

- `Renderer` is now primarily a compact shell of explicit owner states:
  - `TextSystem`
  - `PathState`
  - SVG registry/raster owners
  - diagnostics owner
  - intermediate pool owner
  - material/custom-effect owner
  - `CustomEffectV3PyramidState`
- the remaining "should we split more?" questions for `services.rs`, `recorders/effects.rs`, and
  `shaders.rs` are already closed by the v1 stop-point audits
- zero-consumer cache/registry helper exports were removed from the backend root, and the detached
  legacy `svg_cache.rs` path is retired from the compile path
- remaining registry/cache helper structs stay behind private backend modules instead of the crate
  root

Evidence:

- `crates/fret-render-wgpu/src/renderer/mod.rs`
- `docs/workstreams/renderer-modularity-fearless-refactor-v1/FINISHING_AUDIT.md`
- `docs/workstreams/renderer-modularity-fearless-refactor-v1/SHADERS_AUDIT.md`
- `crates/fret-render-wgpu/src/lib.rs`

Assessment:

- v1 no longer has a meaningful "identify what can move out of `Renderer`" backlog item
- the public backend root no longer leaks the reviewed zero-consumer cache/registry helpers
- first-party callers are already aligned with the curated `fret-render` facade story

Decision:

- close `RMFR-renderer-040` as done
- close `RMFR-exports-060` as done
- close `RMFR-cleanup-090` as done
- close `RMFR-cleanup-091` as done
- close `RMFR-cleanup-092` as done
- close `RMFR-cleanup-093` as done

### 3. Closeout gates and docs are sufficient; no new ADR is required

Current shape:

- the stable default facade is now explicit in:
  - `crates/fret-render/src/lib.rs`
  - `crates/fret-render/tests/facade_surface_snapshot.rs`
  - `docs/crate-usage-guide.md`
- both supported topology stories are taught consistently:
  - editor-hosted convenience path via `WgpuContext`
  - engine-hosted direct path via host-provided adapter/device/queue/surface objects
- the accepted modularization/gate process contract already exists in
  `docs/adr/0201-renderer-internals-modularization-and-gates-v1.md`
- the topology contract already exists in `docs/architecture.md`

Closeout verification run on 2026-03-13:

- `python3 tools/check_layering.py`
- `CARGO_TARGET_DIR=target-codex-render cargo check -p fret-render-wgpu --tests`
- `CARGO_TARGET_DIR=target-codex-render cargo nextest run -p fret-render-wgpu -E 'test(requested_and_emitted_custom_effect_counters_track_all_versions) | test(degradation_counters_track_reason_and_kind_totals) | test(diff_segment_reports_tracks_shape_changes_and_pass_growth) | test(render_plan_dump_assembly_tracks_segment_passes_and_counts) | test(custom_effect_summaries_include_abi_and_input_counts) | test(target_usage_tracks_max_size) | test(encode_custom_effect_v3_pass_keeps_distinct_source_targets)'`
- `CARGO_TARGET_DIR=target-codex-render cargo nextest run -p fret-render-wgpu -E 'test(text_locale_changes_font_stack_key) | test(emoji_sequences_use_color_quads_when_color_font_is_available) | test(cjk_glyphs_populate_mask_or_subpixel_atlas_when_cjk_lite_font_is_available) | test(text_measure_matches_prepare_across_fractional_scale_factors)'`
- `CARGO_TARGET_DIR=target-codex-render cargo nextest run -p fret-render -p fret-render-wgpu -E 'test(facade_surface_snapshot_matches_v1_contract_buckets) | test(renderer_accepts_host_provided_gpu_topology)'`

Assessment:

- render-plan reporting/dump guardrails remain green after the latest modularization slices
- text extracted-domain guards remain green
- facade/topology closure is still locked by compile-time and smoke-test coverage
- this workstream changed curation and internal ownership boundaries, but it did not introduce a
  new cross-layer semantic contract that requires a separate ADR
- the existing doc set plus the finishing/shader audits is enough; extra files such as
  `EVIDENCE_AND_GATES.md`, `OPEN_QUESTIONS.md`, or `MIGRATION_MATRIX.md` would duplicate the
  existing evidence

Decision:

- close `RMFR-gates-073` as done
- close `RMFR-docs-081` as done
- close `RMFR-docs-082` as done with "no new ADR required"
- close `RMFR-docs-083` as done with "no new alignment update required"
- close `RMFR-docs-084` as done

## Closure Summary

For `renderer-modularity-fearless-refactor-v1`:

- the stable default facade is curated and documented
- the host-provided GPU topology seam is explicitly taught and tested
- `text/mod.rs` is reduced to a small state shell
- `Renderer` owner extraction is complete enough to stop splitting for v1
- export tightening and legacy helper retirement are complete enough to stop shrinking for v1
- all remaining tracker items are closed

The next renderer work should start from new semantic or capability goals, not from "keep splitting
files in this workstream."
