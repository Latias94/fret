# Fret Node Paint Root Cache Plan Adapter v1 - TODO

Status: Active
Last updated: 2026-05-25

## CPA-M0 - Scope And Evidence Freeze

- [ ] CPA-010 [owner=unassigned] [deps=none] [scope=docs/workstreams/fret-node-paint-root-cache-plan-adapter-v1]
  Goal: Freeze cache-plan adapter scope, non-goals, and gates.
  Validation: `python3 -m json.tool docs/workstreams/fret-node-paint-root-cache-plan-adapter-v1/WORKSTREAM.json`
  Evidence: `DESIGN.md`, `EVIDENCE_AND_GATES.md`
  Handoff: Do not include frame setup or scene emission.

## CPA-M1 - Cache-Plan Adapter Proof

- [ ] CPA-020 [owner=unassigned] [deps=CPA-010] [scope=ecosystem/fret-node/src/ui/canvas/widget/paint_root]
  Goal: Introduce a named retained-agnostic cache-plan context seam for host access, bounds, and
  scale factor used by `prepare_paint_root_cache_plan`.
  Validation: `cargo check -p fret-node --features compat-retained-canvas`; narrow source-policy test.
  Evidence: `paint_root/cache_plan.rs`, new adapter/binding modules, source-policy test in
  `ecosystem/fret-node/src/lib.rs`
  Handoff: Keep frame setup, static layer replay/store, cached/immediate passes, and tail cleanup
  out of scope.

## CPA-M2 - Next Paint Family Decision

- [ ] CPA-030 [owner=planner] [deps=CPA-020] [scope=docs/workstreams/fret-node-paint-root-cache-plan-adapter-v1]
  Goal: Decide whether to close the lane or split the next paint family.
  Validation: `python3 tools/check_workstream_catalog.py`; `git diff --check`
  Evidence: `EVIDENCE_AND_GATES.md`, `HANDOFF.md`, optional closeout audit.
  Handoff: Candidate follow-ons are frame setup, static layer replay/store, or scene pass emission.
