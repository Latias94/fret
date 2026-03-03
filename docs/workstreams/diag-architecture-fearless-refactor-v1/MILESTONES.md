# Diagnostics Architecture (Fearless Refactor v1) — Milestones

Status: Draft (workstream tracker)

This is a staged plan toward a cleaner, more extensible, more future-proof diagnostics stack.

Each milestone should produce:

- a small set of landable PR-sized diffs,
- at least one regression gate (test or diag script),
- clear evidence anchors (paths + key functions).

## M0 — Baseline map + invariants

Outcome:

- A shared understanding of “what diag is” and “what outputs must always exist”.

Deliverables:

- “Architecture map” (links to crates + entry points).
- “Artifact invariants” checklist.
- A list of top churn hotspots and their owners.

Exit criteria:

- A new contributor can locate: protocol, runtime exporter, tooling engine, CLI, GUI, viewer within 5 minutes.

## M1 — Tooling engine seams (no behavior changes)

Outcome:

- `crates/fret-diag` stops being a single blob; new features plug into registries.

Deliverables:

- `SuiteRegistry` and `CheckRegistry` introduced and used by at least one command path.
- Artifact materialization/integrity boundary becomes a single focused module.

Exit criteria:

- Adding a new suite/check does not require editing a giant central match statement.

## M2 — Runtime extensibility contract (ecosystem)

Outcome:

- Ecosystem crates can contribute diagnostics safely and cheaply.

Deliverables:

- `debug.extensions` slot in snapshots (bounded).
- Option A registry in `fret-bootstrap` and one real extension migrated to prove it.
- Naming + versioning rules for extension keys.

Exit criteria:

- Adding a new ecosystem extension does not require changing core typed snapshot schema.

## M3 — Layout correctness workflow (semantics-first)

Outcome:

- Layout regressions become easy to gate and explain.

Deliverables:

- One deterministic layout correctness script (selectors + bounds predicates).
- A bundle-scoped layout sidecar (Taffy subtree dump) tied to a script step or dump request.

Exit criteria:

- A layout regression bug report can be represented as:
  - script + bundle + (optional) layout sidecar,
  - without requiring ad-hoc debug UI in the demo.

## M4 — Layout performance workflow (hotspots + thresholds)

Outcome:

- Layout perf regressions are caught early with readable evidence.

Deliverables:

- A perf suite preset focused on layout-heavy scenarios.
- A bounded “layout perf summary” report (solve/measure/hotspots deltas).
  - Tooling contract: `docs/workstreams/diag-architecture-fearless-refactor-v1/LAYOUT_PERF_SUMMARY_V1.md`

Exit criteria:

- “p95 solve time got worse” is accompanied by a top-hotspots diff and a stable reason code.

## M5 — DevTools UX (later, but aligned)

Outcome:

- Everyday author workflows don’t require CLI fluency.

Deliverables:

- DevTools GUI can browse extensions and view layout sidecars (even if only as raw JSON).
- “Pick selector → apply to script → run → view evidence” is one coherent flow.

Exit criteria:

- A component author can author a new repro gate in minutes using the GUI.

## M6 — Consolidation (docs + enforcement)

Outcome:

- The architecture stays clean over time.

Deliverables:

- A short “ecosystem diagnostics guide” with one end-to-end example.
- A small set of “do/don’t” rules for adding new diagnostics fields, checks, and extensions.

Exit criteria:

- New diagnostics features are added by extension points first, not by widening monoliths.
