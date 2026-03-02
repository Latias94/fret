# Diagnostics Architecture (Fearless Refactor v1) — TODO

Status: Draft (workstream tracker)

This TODO is organized by milestones (see `MILESTONES.md`). Keep tasks **small and landable**.

## M0 — Baseline and boundaries

- [ ] Write a “current architecture map” note with clickable evidence anchors (crates + key entrypoints).
- [ ] Define the “artifact invariants” checklist (what a successful run must always produce).
- [ ] List the top 10 churn hotspots in `crates/fret-diag` and why they churn.
- [ ] Identify which parts are demo-specific (UI gallery) vs general engine.

## M1 — Tooling engine seams (within `crates/fret-diag`)

- [ ] Introduce `SuiteRegistry` (data-driven suite resolution) and migrate one suite.
- [ ] Introduce `CheckRegistry` for lint/perf checks (no functional change).
- [ ] Split “artifact materialization + integrity” into one focused module boundary with stable APIs.
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

