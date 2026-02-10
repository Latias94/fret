# Delinea Engine Contract Closure v1 — TODO Tracker

Status: Draft (workstream tracker)

Workstream narrative: `docs/workstreams/delinea-engine-contract-closure-v1.md`
Milestone board (one-screen): `docs/workstreams/delinea-engine-contract-closure-v1-milestones.md`

## Tracking format

Each TODO is labeled:

- ID: `DEL-ENG{m}-{area}-{nnn}`
- Status: `[ ]` (open), `[~]` (in progress), `[x]` (done), `[!]` (blocked)

## M0 — Docs honesty + audit closure

- [x] DEL-ENG0-docs-001 Update alignment docs to describe the shipped v1 subset boundaries (Y mapping-first, size-capped indices materialization).
- [x] DEL-ENG0-docs-002 Update ADR 1150 to reflect the shipped v1 subset multi-dimensional `WeakFilter` carrier.
- [x] DEL-ENG0-docs-003 Update ADR implementation alignment notes for ADR 1150 (remove stale “WeakFilter == Filter only” claim).
- [x] DEL-ENG0-docs-004 Update multi-grid adapter notes: retained hosts a single engine; remove stale spec-splitting references.

## M1 — Single-engine multi-grid viewport/layout contract

Design gates (write contracts before code):

- [x] DEL-ENG1-contract-001 Define per-grid viewport/layout carriers (spec/model/output) and routing invariants.
  - Proposed anchors: `ecosystem/delinea/src/spec/mod.rs`, `ecosystem/delinea/src/engine/model/mod.rs`,
    `ecosystem/delinea/src/engine/stages/marks.rs`.
- [x] DEL-ENG1-contract-002 Decide what “linking across grids” means for v1 (none / opt-in / default).
- [x] DEL-ENG1-contract-003 Define the per-grid ordering contract for filter plan steps (X-before-Y within a grid; grid ordering stability).

Implementation steps (keep them small and regression-gated):

- [x] DEL-ENG1-impl-010 Add per-grid plot rect computation in the engine and expose it in output (debuggable).
- [x] DEL-ENG1-impl-011 Teach marks emission to target the correct grid viewport without adapter-side splitting.
- [x] DEL-ENG1-impl-012 Teach axisPointer sampling to route within the correct grid and preserve deterministic series ordering.
- [x] DEL-ENG1-impl-013 Keep brush selection output scoped to the grid/axis pair and preserve link semantics.
- [x] DEL-ENG1-impl-014 Emit `GridId` in `axisPointer` output so UI adapters can route without guessing.
- [x] DEL-ENG1-impl-015 Add global controllers for retained multi-grid (single legend + tooltip/axisPointer overlay).
- [x] DEL-ENG1-impl-016 Add an explicit opt-in policy for cross-grid brush-derived X exports (linking scaffold).

Regression gates:

- [x] DEL-ENG1-tests-020 Add a headless regression test for multi-grid: window writes + marks counts are stable.
- [x] DEL-ENG1-tests-021 Add a retained multi-grid demo with a concrete P0 checklist.
- [x] DEL-ENG1-contract-004 Define link event contracts for axisPointer/cursor sync (ADR 1172).
- [x] DEL-ENG1-contract-005 Define link event contracts for domain window sync (ADR 1173).
- [x] DEL-ENG1-contract-006 Define host-level link mapping policy for cross-spec linking (ADR 1174).
- [x] DEL-ENG1-impl-017 Add key-based chart link coordinator and mapping router (LinkAxisKey).
- [x] DEL-ENG1-impl-018 Add explicit axis map overrides for ambiguous cross-spec linking (ADR 1174).

## M2 — Transform lineage contract (derived datasets/columns)

Design gates:

- [x] DEL-ENG2-contract-001 Define “raw index identity” across transform chains (ADR 1175).
- [x] DEL-ENG2-contract-002 Define the minimum transform node set for v1 (ADR 1176).
- [x] DEL-ENG2-contract-003 Define caching keys and invalidation boundaries (ADR 1177).

Implementation steps:

- [x] DEL-ENG2-impl-010 Introduce a minimal engine-owned transform graph for datasets (not just indices views).
- [x] DEL-ENG2-impl-011 Migrate the ECharts translator’s eager `dataset.transform` table cloning to the engine contract surface (keep v1 subset).

Regression gates:

- [x] DEL-ENG2-tests-020 Add headless goldens for chained transforms with stable raw-index identity assertions.

## M3 — Incremental mutation semantics (append/update)

Design gates:

- [x] DEL-ENG3-contract-001 Document supported mutation operations in v1 (append-only required; updates constrained and explicit).
  - ADR 1178: `docs/adr/1178-delinea-mutation-surface-and-data-revisioning-v1.md`
- [x] DEL-ENG3-contract-002 Define which caches must resume vs invalidate for each mutation type.
  - ADR 1179: `docs/adr/1179-delinea-mutation-cache-invalidation-and-resume-policy-v1.md`

Implementation steps:

- [x] DEL-ENG3-impl-010 Add an explicit “update” mutation API (if in scope) with constrained semantics and deterministic invalidation.
  - Evidence: `ecosystem/delinea/src/data/mod.rs` (`DataTable::{update_row_f64,update_columns_f64}`) and `ecosystem/delinea/src/engine/tests.rs` (`update_mutation_clears_marks_and_forces_rebuild`).
- [x] DEL-ENG3-impl-011 Ensure marks stages and indices views do not regress append-only behavior under budget.
  - Evidence: `ecosystem/delinea/src/engine/mod.rs` (append-only rebuild keeps previous `output.marks`), `ecosystem/delinea/src/engine/stages/marks.rs` (append-only detection survives a marks-stage reset), and `ecosystem/delinea/src/engine/tests.rs` (`append_only_marks_rebuild_preserves_geometry_while_unfinished_multi_series`).

Regression gates:

- [x] DEL-ENG3-tests-020 Add an invariant test that proves append-only scans resume (already partially covered; expand to multi-series).
  - Evidence: `ecosystem/delinea/src/engine/tests.rs` (`append_only_marks_rebuild_preserves_geometry_while_unfinished_multi_series`).
- [x] DEL-ENG3-tests-021 Add one update-semantics test that validates the chosen contract (no silent partial updates).
  - Evidence: `ecosystem/delinea/src/engine/tests.rs` (`update_mutation_clears_marks_and_forces_rebuild`).

## M4 — Conformance harnesses (keep refactors safe)

- [x] DEL-ENG4-gates-001 Expand headless goldens to include a multi-grid scenario once M1 lands.
  - Evidence: `ecosystem/fret-chart/tests/echarts_headless_goldens.rs` (`golden_multi_grid_two_series_with_shared_datazoom`)
  - Golden: `goldens/echarts-headless/v1/multi-grid-two-series-datazoom.json`
- [x] DEL-ENG4-gates-002 Add a “filter mode torture” headless snapshot (WeakFilter + Empty + Y indices cap edge cases).
  - Evidence: `ecosystem/fret-chart/tests/echarts_headless_goldens.rs` (`golden_filter_mode_weakfilter_x_and_empty_y`, `golden_filter_mode_y_indices_skips_when_view_len_exceeds_cap`)
  - Goldens: `goldens/echarts-headless/v1/filtermode-weakfilter-x-empty-y.json`, `goldens/echarts-headless/v1/filtermode-y-indices-view-len-cap.json`
- [x] DEL-ENG4-gates-003 Add one `fretboard diag` script for multi-grid + linking (optional).
  - Script: `tools/diag-scripts/chart-multi-axis-linking-domain-window-pixels-changed.json`
  - Gate: verifies that a domain-window change in the top chart propagates to the bottom chart.
  - Run (example): `cargo run -p fretboard -- diag run tools/diag-scripts/chart-multi-axis-linking-domain-window-pixels-changed.json --check-pixels-changed chart-multi-axis-top --check-pixels-changed chart-multi-axis-bottom --env FRET_DIAG_SCREENSHOTS=1 --launch -- cargo run -p fret-demo --bin chart_multi_axis_demo --release`
