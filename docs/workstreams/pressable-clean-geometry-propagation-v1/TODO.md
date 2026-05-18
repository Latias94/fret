# Pressable Clean Geometry Propagation v1 - TODO

Status: Closed
Last updated: 2026-05-18

## M0 - Scope And Evidence Freeze

- [x] PGP-010 [owner=planner] [deps=none] [scope=docs/workstreams/pressable-clean-geometry-propagation-v1]
  Goal: Open the narrow follow-on lane, freeze the `Pressable` risk model, link RLO-030 after
  evidence, and define the first proof tasks without runtime changes.
  Validation:
  `python3 -m json.tool docs/workstreams/pressable-clean-geometry-propagation-v1/WORKSTREAM.json`;
  `python3 tools/check_workstream_catalog.py`; `git diff --check`.
  Evidence:
  `docs/workstreams/pressable-clean-geometry-propagation-v1/DESIGN.md`;
  `docs/workstreams/retained-layout-orchestration-v1/CLOSEOUT_AUDIT_2026-05-18.md`.
  Handoff: Complete. Start PGP-020 before editing runtime code.

## M1 - Pressable Source Audit

- [x] PGP-020 [owner=codex] [deps=PGP-010] [scope=crates/fret-ui/src/{tree/layout, declarative/host_widget,tree/dispatch}]
  Goal: Audit the `Pressable` geometry and side-effect paths that a clean-geometry propagation
  fast path must preserve.
  Validation:
  `rg -n "Pressable|pressable" crates/fret-ui/src/tree/layout crates/fret-ui/src/declarative/host_widget crates/fret-ui/src/tree/dispatch crates/fret-ui/src/declarative/tests -S`.
  Evidence: `docs/workstreams/pressable-clean-geometry-propagation-v1/PGP_020_030_SOURCE_AUDIT_AND_RED_PROOF_2026-05-18.md`.
  Handoff: Complete. No audited side effect appears to require rerunning `Pressable` layout during
  clean width-only bounds propagation; PGP-040 still must keep propagated bounds authoritative.

## M2 - First RED Proof

- [x] PGP-030 [owner=codex] [deps=PGP-020] [scope=crates/fret-ui/src/declarative/tests/layout/layout_engine.rs,crates/fret-ui/src/declarative/tests/interactions/pressable.rs]
  Goal: Add a failing proof that current small width-only resize still performs avoidable
  `Pressable` wrapper layout while side-effect invariants are explicit.
  Validation:
  `cargo nextest run -p fret-ui clean_geometry_small_resize_propagates_through_pressable_wrapper --no-fail-fast`;
  `cargo nextest run -p fret-ui pressable_on_activate_hook_runs_on_pointer_activation pressable_on_hover_change_hook_runs_on_pointer_move --no-fail-fast`.
  Evidence: `crates/fret-ui/src/declarative/tests/layout/layout_engine.rs`
  (`clean_geometry_small_resize_propagates_through_pressable_wrapper`);
  `docs/workstreams/pressable-clean-geometry-propagation-v1/PGP_020_030_SOURCE_AUDIT_AND_RED_PROOF_2026-05-18.md`.
  Handoff: Complete. The focused layout proof is RED as expected:
  `layout_engine_solves=0`, no clean-geometry rejection noise, but `layout_nodes_performed=2`.
  Continue with PGP-040; do not widen scope beyond the support-matrix slice.

## M3 - Minimal Runtime Slice Or No-Change Verdict

- [x] PGP-040 [owner=codex] [deps=PGP-030] [scope=crates/fret-ui/src/tree/layout/clean_geometry.rs]
  Goal: Either add `Pressable` to the clean-geometry execution support matrix with targeted tests,
  or record a no-change verdict.
  Validation:
  `cargo nextest run -p fret-ui clean_geometry_small_resize_propagates_through_pressable_wrapper --no-fail-fast`;
  `cargo nextest run -p fret-ui layout_engine pressable --no-fail-fast`.
  Evidence: `crates/fret-ui/src/tree/layout/clean_geometry.rs`;
  `docs/workstreams/pressable-clean-geometry-propagation-v1/EVIDENCE_AND_GATES.md`.
  Handoff: Complete. `ElementInstance::Pressable(_)` is now in the execution allowlist, the focused
  layout proof is green, and Pressable interaction guards remain green.

## M4 - Perf Confirmation And Closeout

- [x] PGP-050 [owner=codex] [deps=PGP-040] [scope=docs/workstreams/pressable-clean-geometry-propagation-v1,target/fret-diag]
  Goal: Capture or reuse UI Gallery resize-jitter evidence to confirm whether the local hotspot
  moved, then close the lane.
  Validation:
  `target/release/fretboard-dev diag stats <bundle.schema2.json> --sort time --top 20`;
  `python3 tools/check_layering.py`; `cargo fmt --check`; `git diff --check`;
  `python3 tools/check_workstream_catalog.py`.
  Evidence: Bundle path, diff summary, closeout note, and final `WORKSTREAM.json` status.
  Handoff: Complete. Fresh PGP-050 evidence shows `Pressable` moved off the worst-frame layout
  hotspot list. Remaining hotspots are `ViewCache`, `Scroll`, and a small `Flex` owner; split those
  to separate lanes only with fresh attribution.
