# Docking N-ary Split Graph — Milestones (v1)

Status: Draft (workstream plan; normative contracts live in ADRs)

This plan is designed to be landable and gate-driven. Each milestone must leave the repo in a
state where:

- `cargo nextest run -p fret-core` and `cargo nextest run -p fret-docking` remain green, and
- at least one scripted diag/perf gate exists for the changed behavior.

## M0 — Spec lock + baseline evidence

Outcome:

- Lock the canonical invariants and the “insert instead of wrap” rule.
- Decide initial defaults (share split ratio, preview semantics).
- Identify the minimum set of correctness + perf gates.

Deliverables:

- `docs/workstreams/docking-nary-split-graph-v1.md` finalized.
- A “baseline bundle” captured from `docking_arbitration_demo` for later comparisons.
  - Store under `.fret/diag/exports/<timestamp>/` (local only; do not commit).

Gates:

- None new required, but record how to reproduce:
  - `cargo run -p fretboard -- dev native --bin docking_arbitration_demo`

## M1 — Core: N-ary safe mutations + simplification pipeline

Outcome:

- Core graph helpers no longer assume binary splits.
- Post-op simplification produces canonical form.

Status:

- Implemented in `crates/fret-core/src/dock/mutate.rs` (`simplify_window_forest`).
- Regression gates live in `crates/fret-core/src/dock/tests.rs` (e.g. pruning empty tabs in 3+ child splits).

Implementation targets:

- `crates/fret-core/src/dock/mutate.rs`
  - N-ary safe collapse/prune logic.
- `crates/fret-core/src/dock/query.rs`
  - Add helpers needed for parent/sibling discovery without repeated subtree scans.
- `crates/fret-core/src/dock/apply.rs`
  - call `simplify` after ops that can create/remove nodes.

Gates:

- Add unit tests that:
  - build 3+ child splits and remove the middle tabs,
  - verify no invalid split/tabs nodes remain,
  - verify fractions normalize.

## M2 — Core: insert-into-same-axis split semantics

Outcome:

- `MovePanel` / `MoveTabs` edge docking prefers inserting into a same-axis split.
- Fractions are updated by splitting the target share, not resetting to 50/50.

Status:

- Implemented in `crates/fret-core/src/dock/mutate.rs` (`insert_edge_child_prefer_same_axis_split`).
- Deterministic gates live in `crates/fret-core/src/dock/tests.rs`:
  - `edge_dock_inserts_into_existing_same_axis_split_and_splits_share`
  - `repeated_edge_dock_keeps_same_axis_splits_flat`

Implementation targets:

- `crates/fret-core/src/dock/mutate.rs` (new helpers):
  - locate nearest same-axis parent,
  - insert child at index,
  - split a share.
- `crates/fret-core/src/dock/apply.rs`:
  - route edge docking to new insertion helpers.

Gates:

- Deterministic unit tests for repeated edge docking sequences:
  - depth does not grow unbounded,
  - nested same-axis splits are flattened,
  - total panel set remains correct.
- Add a unit test that verifies share splitting preserves the target child’s approximate pixel size
  under `compute_layout` for a fixed bounds (sanity, not pixel-perfect).

## M3 — Docking UI: preview geometry and splitter drags

Outcome:

- Drop previews match commit semantics for insertion vs wrapping.
- Splitter drags update adjacent shares only (stable, no oscillation).
- Nested split stabilization is reduced as canonical form guarantees take over.

Status:

- Preview semantics aligned via `DockGraph::edge_dock_decision` (insert vs wrap).
- Splitter drag updates use adjacent-only resizing for N-ary splits.
- Remaining: deterministic preview geometry tests + N-ary handle hit-test coverage + stabilization cleanup.

Implementation targets:

- `ecosystem/fret-docking/src/dock/layout.rs`:
  - compute preview rects based on simulated commit (pure function).
- `ecosystem/fret-docking/src/dock/space.rs`:
  - update drag intent resolution to request the new semantics.
- `ecosystem/fret-docking/src/dock/split_stabilize.rs`:
  - simplify or remove; keep only what remains necessary after M2.
- `ecosystem/fret-docking/src/dock/paint.rs`:
  - adjust overlay rendering to match the new preview model (no “always half” assumption).

Gates:

- Existing docking tests remain green.
- Add at least one new geometry test that:
  - simulates insertion into an existing split and checks the preview rect.

## M4 — Diagnostics: scripted correctness gates (`fretboard diag`)

Outcome:

- At least one end-to-end scripted gate exists that fails with evidence when docking behavior drifts.

Deliverables:

- A small suite under `tools/diag-scripts/` targeting `docking_arbitration_demo`:
  - repeated edge-dock sequence,
  - splitter drag,
  - Escape cancel during drag (to ensure arbitration remains correct).

Recommended invariants (bundle-based, not pixels):

- active tab changes as expected after commit,
- viewport capture and docking drag never both own the same pointer session,
- hit-test trace shows the intended dock target,
- no stuck internal drag hover after completion/cancel.

Gates:

- Prefer running via `--launch` so environment is applied consistently:
  - `pwsh -NoProfile -Command "$env:FRET_DIAG=1; cargo run -p fretboard -- diag suite <suite-name> --launch -- cargo run -p fret-demo --bin docking_arbitration_demo --release"`
- If any script uses screenshots:
  - `pwsh -NoProfile -Command "$env:FRET_DIAG=1; $env:FRET_DIAG_SCREENSHOTS=1; cargo run -p fretboard -- diag suite <suite-name> --launch -- cargo run -p fret-demo --bin docking_arbitration_demo --release"`

## M5 — Performance: probe and gate “editor worst-cases”

Outcome:

- Prevent perf regressions from the new semantics (especially on pointer-move + split drags).

Deliverables:

- A perf probe (diag-perf or scripted loop) that exercises:
  - repeated splitter drags in a layout with many panels,
  - repeated tab-drag hover updates (drop hints).
  - Prefer the smallest deterministic demo binary; fall back to `docking_arbitration_demo` if needed.

Gates:

- Use an existing gate where possible (resize probes), plus one docking-specific gate:
  - `tools/perf/diag_resize_probes_gate.sh --suite ui-resize-probes --attempts 3`
  - plus a docking-specific `diag perf` suite (name TBD).

## M6 — Cleanup: remove transitional code paths and update workstream evidence

Outcome:

- Old “binary-only” code paths and assumptions are removed.
- Parity and checklist docs point to the new invariants and gates.

Deliverables:

- Update:
  - `docs/docking-imgui-parity-matrix.md` (preview semantics and drop rule notes),
  - `docs/docking-arbitration-checklist.md` (add new scripted gate references).
  - `docs/workstreams/docking-nary-split-graph-v1-todo.md` (mark done items; keep it honest).

Gates:

- All existing tests and new diag/perf gates pass.
