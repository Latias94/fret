# Diagnostics Architecture (Fearless Refactor v1) — TODO

Status: Draft (workstream tracker)

This TODO is organized by milestones (see `MILESTONES.md`). Keep tasks **small and landable**.

## M0 — Baseline and boundaries

- [x] Write a “current architecture map” note with clickable evidence anchors (crates + key entrypoints).
- [x] Define the “artifact invariants” checklist (what a successful run must always produce).
- [x] List the top 10 churn hotspots in `crates/fret-diag` and why they churn.
- [x] Identify which parts are demo-specific (UI gallery) vs general engine.

## M1 — Tooling engine seams (within `crates/fret-diag`)

- [x] Introduce `SuiteRegistry` (initial scaffolding) and migrate one command path (`diag list suites`).
- [x] Introduce `CheckRegistry` scaffolding (seam only; no behavior change yet).
- [x] Migrate one post-run check to `CheckRegistry` (start with `gc_sweep_liveness`).
- [x] Migrate additional post-run checks to validate registry flexibility (`notify_hotspot_file_max`, `triage_hint_absent_codes`).
- [x] Migrate pixel post-run gates (`--check-pixels-changed`, `--check-pixels-unchanged`) into `CheckRegistry`.
- [x] Centralize “do we need post-run checks?” planning via `CheckRegistry::wants_post_run_checks` (reduce orchestration duplication).
- [x] Centralize “do we need screenshots?” planning via `CheckRegistry::wants_screenshots` (reduce launch wiring duplication).
- [x] Centralize “do we need a bundle artifact?” planning via `CheckRegistry::wants_bundle_artifact` (reduce dump-bundle wiring duplication).
- [x] Migrate `diag suite` promoted-suite resolution to `SuiteRegistry` (remove duplicate loading logic).
- [x] Centralize builtin suite script resolution + env defaults in `resolve_builtin_suite_scripts` (reduce `diag_suite.rs` churn).
- [x] Split “artifact materialization + integrity” into one focused module boundary with stable APIs (`artifact_store`).
- [x] Migrate remaining ad-hoc post-run checks from `post_run_checks.rs` into `CheckRegistry` (leave only bundle path selection).
- [x] Split builtin post-run checks into domain modules under `crates/fret-diag/src/registry/checks/builtin_post_run/` (reduce merge conflicts and navigation cost).
- [x] Split UI gallery post-run checks into submodules (`code_editor`, `markdown_editor`, `web`, `semantics`, `text`) and lock order with a test.
- [ ] Remove any remaining “global state by convention” patterns in favor of explicit context objects.

## M2 — Runtime extensibility (ecosystem contributions)

- [x] Add a first-class `debug.extensions` slot in runtime snapshots (bounded, additive).
- [x] Implement Option A registry in `fret-bootstrap` (closures registered at init).
- [x] Add one real extension via the registry to validate the seam (`dock.graph.v1`).
- [x] Document extension key naming rules and clipping expectations.
  - Design note: `docs/workstreams/diag-architecture-fearless-refactor-v1/DEBUG_EXTENSIONS_V1.md`
  - ADR: `docs/adr/0310-ui-diagnostics-debug-extensions-v1.md`

## M3 — Layout correctness workflow

- [x] Write down the layout sidecar v1 contract (file naming, JSON shape, clipping rules).
- [x] Add a script-level “layout sidecar request” concept (design first, then implement).
- [x] Tie a Taffy dump into bundle directories as a sidecar (native only, best-effort).
- [x] Add a viewer affordance to load/render the sidecar (ok to start as raw JSON view).
  - CLI: `fretboard diag layout-sidecar ...` (prints path by default; `--print` and `--json` supported)
- [x] Add one deterministic layout gate script in `tools/diag-scripts/` that uses semantics bounds.
  - `tools/diag-scripts/ui-gallery/layout/ui-gallery-empty-outline-layout-sidecar.json`

## M4 — Layout performance workflow

- [x] Standardize perf suite tags for layout-heavy scenarios (UI gallery sweep subset).
- [x] Add a bounded “layout perf summary” viewer (top frame solves + hotspots).
  - CLI: `fretboard diag layout-perf-summary ...`
- [x] Attach a worst-run layout perf summary to perf gate evidence outputs (bounded).
  - Files: `layout.perf.summary.v1.json`, `check.perf_thresholds.json`, `check.perf_hints.json`
- [x] Add one CI-friendly perf gate preset (doc + example command).
  - Suite: `perf-ui-gallery-layout-steady` (CLI alias: `ui-gallery-layout-steady`)

## M5 — Frontend UX (optional / later)

- [ ] DevTools GUI: add “extensions browser” panel (lists keys + JSON).
- [ ] DevTools GUI: add “layout sidecars” browser and open-in-viewer affordance.
- [ ] DevTools GUI: add “copy selector + copy gate snippet” flows for layout correctness scripts.

## M6 — Documentation and consolidation

- [ ] Write a short “How to debug layout” cookbook that unifies: inspect → selector → gate → sidecar.
- [ ] Update `docs/workstreams/diag-devtools-gui-v1/diag-devtools-gui-v1.md` to reference the extension contract.
- [x] Document merge-friendly script authoring practices (normalize/lint + suite composition).
- [x] Add a low-noise alternative for suite membership (single-file suite manifest) for low-churn suites.
- [x] Add a minimal “ecosystem diagnostics authoring guide” with one end-to-end example.
  - `docs/workstreams/diag-architecture-fearless-refactor-v1/MIGRATION_GUIDE.md` (includes `fretboard diag extensions ...`).
