# UI Gallery view-cache web perf stabilization (v1)

Status: Draft

## Context

Web runner perf evidence for `magic_patterns_torture` shows that steady-state frame time is dominated by `paint` rather than renderer encode.
In the captured bundle, view-cache reuse is effectively disabled (`cache_roots=0`), which makes paint-side costs (and cache-miss noise) dominate the signal we need for renderer/WebGPU decisions.

This workstream focuses on making web perf captures *stable and interpretable* by making view-cache configuration controllable and repeatable on `wasm32`.

## Goals

- Make view-cache configuration reproducible for `trunk serve` runs (no env vars on web).
- Reduce paint-side noise so renderer/WebGPU evidence is not drowned out.
- Keep changes bounded to UI Gallery harness/policy; do not change renderer semantics in this workstream.

## Non-goals

- Do not expand renderer pipeline key space (e.g. `REN-VNEXT-webgpu-004`) without evidence that fragment material eval dominates.
- Do not change the `fret-ui` contract surface; this stays in ecosystem policy/harness.

## Evidence inputs (baseline)

- Web script: `tools/diag-scripts/ui-gallery-magic-patterns-torture-perf-steady-web.json`
- Exported bundle example: `.fret/diag/exports/<ts>-bundle` (re-capture for each iteration)
- Metrics: `fretboard diag stats` + `fretboard diag triage --json`

## Configuration surface (web)

These URL query flags are read from either `?` or `#...?...` (hash routing):

- `fret_ui_gallery_view_cache=1|0`
- `fret_ui_gallery_view_cache_shell=1|0`
- `fret_ui_gallery_view_cache_inner=1|0`
- `fret_ui_gallery_view_cache_continuous=1|0`

Rationale: UI Gallery already supports env-var configuration on native; web runs need an equivalent that can be embedded into the reproducible URL used for devtools-ws exports.

## Process (repeatable)

1. Start `fret-devtools-ws` and the web gallery.
2. Open the gallery URL with:
   - devtools params (`fret_devtools_ws`, `fret_devtools_token`)
   - view-cache params above (for the experimental condition)
3. Export a bundle via `fret-diag-export` using the web script.
4. Compare `diag stats/triage` against the baseline bundle:
   - total/layout/paint p50/p95
   - `paint_cache_misses`
   - view-cache counters (`cache_roots`, `cache_roots_reused`, `view_cache_active`)

## Next steps

See:
- `docs/workstreams/ui-gallery-view-cache-web-perf-stabilization-v1-todo.md`
- `docs/workstreams/ui-gallery-view-cache-web-perf-stabilization-v1-milestones.md`

