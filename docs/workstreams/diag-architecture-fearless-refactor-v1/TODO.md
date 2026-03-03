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
- [x] Migrate `diag suite` promoted-suite resolution to `SuiteRegistry` (remove duplicate loading logic).
- [x] Centralize builtin suite script resolution + env defaults in `resolve_builtin_suite_scripts` (reduce `diag_suite.rs` churn).
- [x] Split “artifact materialization + integrity” into one focused module boundary with stable APIs (`artifact_store`).
- [ ] Remove any remaining “global state by convention” patterns in favor of explicit context objects.

## M2 — Runtime extensibility (ecosystem contributions)

- [ ] Add a first-class `debug.extensions` slot in runtime snapshots (bounded, additive).
- [ ] Implement Option A registry in `fret-bootstrap` (closures registered at init).
- [ ] Add one real extension (e.g. docking interaction snapshot) via the registry to validate the seam.
- [ ] Document extension key naming rules and clipping expectations.

## M3 — Layout correctness workflow

- [ ] Add a script-level “layout sidecar request” concept (design first, then implement).
- [ ] Tie a Taffy dump into bundle directories as a sidecar (native only, best-effort).
- [ ] Add a viewer affordance to load/render the sidecar (ok to start as raw JSON view).
- [ ] Add one deterministic layout gate script in `tools/diag-scripts/` that uses semantics bounds.

## M4 — Layout performance workflow

- [ ] Standardize perf suite tags for layout-heavy scenarios (UI gallery sweep subset).
- [ ] Add a “layout hotspots diff summary” in tooling output (bounded).
- [ ] Add one CI-friendly perf gate preset (doc + example command).

## M5 — Frontend UX (optional / later)

- [ ] DevTools GUI: add “extensions browser” panel (lists keys + JSON).
- [ ] DevTools GUI: add “layout sidecars” browser and open-in-viewer affordance.
- [ ] DevTools GUI: add “copy selector + copy gate snippet” flows for layout correctness scripts.

## M6 — Documentation and consolidation

- [ ] Write a short “How to debug layout” cookbook that unifies: inspect → selector → gate → sidecar.
- [ ] Update `docs/workstreams/diag-devtools-gui-v1.md` to reference the extension contract.
- [ ] Add a minimal “ecosystem diagnostics authoring guide” with one end-to-end example.
