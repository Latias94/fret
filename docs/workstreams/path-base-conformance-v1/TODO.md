# Path Base Conformance v1 — TODO

Status: Closed
Last updated: 2026-05-18

## M0 — Base Path Gates

- [x] PBC-010 [owner=codex] [deps=none] [scope=crates/fret-render-wgpu/tests/path_base_conformance.rs,crates/fret-render-wgpu/src/renderer/path.rs]
  Goal: Add conformance tests for ADR 0080 base path behavior: fill-rule overlap semantics,
  transformed path rendering under clip, and conservative metrics bounds against tessellated
  vertices.
  Validation: `cargo test -p fret-render-wgpu --locked --test path_base_conformance -j 1`;
  `cargo test -p fret-render-wgpu --locked --lib renderer::path::tests::path_metrics_bounds_contain_tessellated_vertices -j 1`.
  Evidence: test file plus path module unit coverage.
  Handoff: Keep this slice behavior-first. Do not widen path API or merge stroke/paint extension
  concerns back into the base contract.
  Status: Done on 2026-05-18. Added GPU readback conformance for fill rules and transform+clip,
  plus a path module unit test for metrics bounds against tessellated vertices.

## M1 — Contract Evidence

- [x] PBC-020 [owner=codex] [deps=PBC-010] [scope=docs/adr/0080-vector-path-contract.md,docs/adr/IMPLEMENTATION_ALIGNMENT.md,docs/workstreams/path-base-conformance-v1]
  Goal: Update ADR 0080, implementation alignment, and workstream evidence to reflect the new base
  conformance gates.
  Validation: `python tools/check_workstream_catalog.py`; `python tools/check_layering.py`;
  `git diff --check`.
  Evidence: ADR/alignment/workstream docs point at the new gates and leave only genuinely remaining
  follow-ons.
  Status: Done on 2026-05-18. ADR 0080 and implementation alignment now point at the base
  conformance gates.

## M2 — Close Or Split

- [x] PBC-030 [owner=planner] [deps=PBC-020] [scope=docs/workstreams/path-base-conformance-v1]
  Goal: Close this lane if ADR 0080's base gap is resolved, or split a narrower follow-on if the
  tests expose a separate renderer behavior issue.
  Validation: `WORKSTREAM.json`, `TODO.md`, `MILESTONES.md`, and `EVIDENCE_AND_GATES.md` agree.
  Evidence: closeout audit or explicit follow-on note.
  Status: Done on 2026-05-18. Lane closed with `CLOSEOUT_AUDIT_2026-05-18.md`; future path work
  starts as a narrower additive follow-on only when needed.
