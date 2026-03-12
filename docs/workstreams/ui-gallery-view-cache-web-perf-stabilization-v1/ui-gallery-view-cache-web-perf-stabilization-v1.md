# UI Gallery view-cache web perf stabilization (v1)

Status: Landed (evidence harness ready)

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
- Web scripts (code editor torture):
  - `tools/diag-scripts/ui-gallery-code-editor-torture-perf-steady-web.json`
  - `tools/diag-scripts/ui-gallery-code-editor-torture-perf-steady-web-view-cache-sidebar.json`
- Web scripts (view-cache harness, URL-driven):
  - `tools/diag-scripts/ui-gallery-view-cache-harness-perf-steady-web-off.json`
  - `tools/diag-scripts/ui-gallery-view-cache-harness-perf-steady-web-on.json`
- Exported bundle (baseline, view-cache disabled): `.fret/diag/exports/1771829809968-bundle`
- Exported bundle (early attempt, view-cache enabled but no roots): `.fret/diag/exports/1771832191642-bundle`
- Metrics: `fretboard diag stats` + `fretboard diag triage --json`
- Tracked perf baselines (derived from exported bundles; worst bundles are local evidence):
  - `docs/workstreams/perf-baselines/ui-gallery-magic-patterns-torture-no-view-cache.web-local.v1.json`
  - `docs/workstreams/perf-baselines/ui-gallery-magic-patterns-torture-view-cache-shell.web-local.v1.json`
  - `docs/workstreams/perf-baselines/ui-gallery-code-editor-torture-no-view-cache.web-local.v1.json`
  - `docs/workstreams/perf-baselines/ui-gallery-code-editor-torture-view-cache-sidebar.web-local.v1.json`

## Configuration surface (web)

These URL query flags are read from either `?` or `#...?...` (hash routing):

- `fret_ui_gallery_view_cache=1|0`
- `fret_ui_gallery_view_cache_shell=1|0`
- `fret_ui_gallery_view_cache_content=1|0`
- `fret_ui_gallery_view_cache_inner=1|0`
- `fret_ui_gallery_view_cache_continuous=1|0`

Rationale: UI Gallery already supports env-var configuration on native; web runs need an equivalent that can be embedded into the reproducible URL used for devtools-ws exports.

Implementation note: on web targets, UI Gallery routing may rewrite/canonicalize the URL. The web harness persists these flags into
`window.__FRET_UI_GALLERY_VIEW_CACHE*` (and localStorage) early in `index.html`, and the Rust side falls back to reading those globals.

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

## Known pitfalls (from early evidence)

- `view_cache_active=true` does not guarantee `view_cache_roots_total>0`: shell caching must actually wrap stable roots.
- Avoid per-frame model writes that trigger `Invalidation::Layout` churn (e.g. settings/command availability flags) or view-cache reuse will be suppressed.
- On web, scripted diagnostics/export depends on frames advancing. If the tab is throttled (background) or the scene is fully idle,
  scripts can time out waiting for results. For evidence URLs, prefer `fret_ui_gallery_view_cache_continuous=1` while iterating.

## Next steps

See:
- `docs/workstreams/ui-gallery-view-cache-web-perf-stabilization-v1/ui-gallery-view-cache-web-perf-stabilization-v1-todo.md`
- `docs/workstreams/ui-gallery-view-cache-web-perf-stabilization-v1/ui-gallery-view-cache-web-perf-stabilization-v1-milestones.md`
And the renderer/WebGPU decision tracker:
- `docs/workstreams/renderer-vnext-fearless-refactor-v1/renderer-vnext-fearless-refactor-v1-todo.md` (see `REN-VNEXT-webgpu-004`)
