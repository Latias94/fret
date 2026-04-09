# Diag perf attribution v1: field inventory (draft)

This document is a **living index** of the perf fields that show up in:

- `bundle.json` (per-run evidence bundle)
- `triage.json` (explainability summary derived from a bundle)
- `check.perf_thresholds.json` / `check.perf_hints.json` (optional gate evidence)

It is intentionally **pragmatic**: it focuses on the fields that are currently the most useful for
UI smoothness work (especially on Windows), and points you at where those fields are measured and
how to interpret them.

## Reading guide: typical vs tail

- **Typical perf**: use percentiles (`p50`, `p95`) from `fretboard-dev diag perf ... --repeat N` and/or
  use `triage.json` → `stats.avg.*` (per-frame averages over the considered snapshots).
- **Tail perf / spikes**: use `triage.json` → `stats.max.*` and the worst-frame hints, then drill
  into the bundle with `fretboard-dev diag stats <bundle> --sort time --top 30`.

## Where the fields come from

High level pipeline for perf diagnostics:

1. UI runtime records per-frame stats during the app loop.
2. `ecosystem/fret-bootstrap` serializes those stats into `bundle.json` snapshots.
3. `crates/fret-diag` reads `bundle.json` and produces:
   - `triage.json` (hints + unit costs + budget view),
   - `diag stats` tables and diffs,
   - optional check outputs (perf thresholds / perf hints).

Key wiring points:

- Snapshot schema (what gets written into `bundle.json`):
  - `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`
- Stats summarization / diff / budget view:
  - `crates/fret-diag/src/stats.rs`
- Perf triage (hints + unit costs):
  - `crates/fret-diag/src/lib.rs` (triage section)

## Core timing fields (per frame, in microseconds)

These are the “first line” metrics that explain where a frame went:

- `total_time_us`
  - Meaning: end-to-end frame time captured by diagnostics.
  - Typical usage: baseline gates, p50/p95 review.
- `layout_time_us`
  - Meaning: total layout time for the frame.
  - Typical usage: smoothness regressions often show up here first.
- `prepaint_time_us`
  - Meaning: prepaint work (building paint primitives, layout-dependent prep).
- `paint_time_us`
  - Meaning: paint work (scene encoding, draw list construction, etc).
- `dispatch_time_us`
  - Meaning: input/command dispatch cost attributed to the frame (when captured).
- `hit_test_time_us`
  - Meaning: hit-test cost attributed to the frame (when captured).

Measurement:

- These are ultimately recorded by the UI runtime, then surfaced into snapshots by
  `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`.

## Layout breakdown fields (why is layout heavy?)

When `layout_time_us` is high but `layout_engine_solve_time_us` is low, you’re usually paying for
mechanism-level work around the solver: root selection, tree walking, observation recording,
view-cache invalidations, etc.

The current sub-breakdown (all in microseconds) is:

- `layout_request_build_roots_time_us`
  - Meaning: time spent building the list of layout roots for this frame.
  - “Bad smell”: large share of layout (`layout.build_roots_heavy` hint).
- `layout_roots_time_us`
  - Meaning: time spent processing layout roots (tree walking + applying layout).
  - “Bad smell”: dominates layout (`layout.roots_heavy` hint).
- `layout_engine_solve_time_us`
  - Meaning: layout solver time (Taffy solve).
  - “Bad smell”: solver dominates layout (`layout.solve_heavy` hint).
- `layout_observation_record_time_us`
  - Meaning: recording layout observation data for the frame.
  - “Bad smell”: recording dominates layout (`layout.observation_heavy` hint).
- `layout_view_cache_time_us`
  - Meaning: time attributed to view-cache work in the layout path.
  - “Bad smell”: view-cache roots become layout-invalidated (`view_cache.layout_invalidated` hint).
- `layout_expand_view_cache_invalidations_time_us`
  - Meaning: time spent expanding view-cache invalidations (if present).

Where measured / wired:

- Layout segmentation is recorded in the UI layout pipeline:
  - `crates/fret-ui/src/tree/layout.rs`
- Values are written into bundle snapshots by:
  - `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`
- Human output “layout_breakdown.us(...)” and JSON stats keys come from:
  - `crates/fret-diag/src/stats.rs`

## Layout observation recording (time + item counts)

If observation recording is a meaningful slice, you should also look at the item counts:

- `layout_observation_record_models_items`
- `layout_observation_record_globals_items`

Interpretation:

- High `layout_observation_record_time_us` with high item counts usually means observation recording
  is on the critical path (not solver time).
- Near-zero observation recording during interactive resize is expected when observation recording
  is intentionally skipped.

## View cache reuse signals (root-level)

These help answer “why did a cached view still relayout?”:

- `view_cache_roots_total`
- `view_cache_roots_reused`
- `view_cache_roots_layout_invalidated`
- `view_cache_roots_cache_key_mismatch`
- `view_cache_roots_not_marked_reuse_root`

Typical workflows:

- Use `triage.json` hints first; then confirm via `diag stats` top frames.
- If `view_cache_roots_layout_invalidated > 0`, the worst frame may be paying a relayout despite
  reuse (expected for some state changes; suspicious if it happens during “toggle-only” actions).

## Invalidation walk (how much work to discover dirtiness?)

- `invalidation_walk_calls`
- `invalidation_walk_nodes`

Interpretation:

- A rising `invalidation_walk_nodes` often correlates with tail spikes during high-frequency input
  (mouse move, resize drag), especially when combined with layout root churn.

## Renderer churn signals (GPU-first, but CPU-visible)

Renderer-related keys are typically surfaced as `top_renderer_*` in perf runs.
Common signals:

- `top_renderer_prepare_text_us` / `top_renderer_text_atlas_upload_bytes`
- `top_renderer_prepare_svg_us` / `top_renderer_svg_upload_bytes`
- `top_renderer_image_upload_bytes`
- `top_renderer_scene_encoding_cache_misses`

Interpretation:

- Upload bytes and cache misses are “churn indicators”: they often correlate with frame spikes and
  should be triaged with a trace/profiler when they regress.

## Practical commands

Typical perf (p50/p95):

- `target/release/fretboard.exe diag perf ui-gallery-steady --repeat 5 --json`

Tail perf (worst frames + attribution):

- `target/release/fretboard.exe diag perf ui-gallery-steady --repeat 3`
- `target/release/fretboard.exe diag triage <bundle.json> --sort time --top 10`
- `target/release/fretboard.exe diag stats <bundle.json> --sort time --top 30`
- `target/release/fretboard.exe diag stats --diff <bundle_a> <bundle_b> --top 30`

Opt-in artifact for timeline correlation:

- `target/release/fretboard.exe diag perf ui-gallery-steady --repeat 1 --trace`
- `target/release/fretboard.exe diag trace <bundle.json>`

