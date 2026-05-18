# Path Base Conformance v1 — Closeout Audit

Date: 2026-05-18
Status: Closed

## Verdict

This narrow ADR 0080 follow-on is closed. The base prepared-path contract now has executable WGPU
evidence for the previously recorded gaps:

- fill-rule behavior on intersecting same-winding overlap regions,
- transformed `SceneOp::Path` rendering under an active clip,
- and representative `PathMetrics.bounds` conservativeness against tessellated vertices.

No public path API changed. ADR 0277 stroke-style v2 and ADR 0278 path paint remain closed additive
extension lanes.

## Evidence

Implementation and gate anchors:

- `crates/fret-render-wgpu/tests/path_base_conformance.rs`
- `crates/fret-render-wgpu/src/renderer/path.rs`
  (`path_metrics_bounds_contain_tessellated_vertices`)
- `docs/adr/0080-vector-path-contract.md`
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md`

## Gates Run

2026-05-18:

- PASS: `cargo fmt --package fret-render-wgpu`
- PASS: `cargo test -p fret-render-wgpu --locked --test path_base_conformance -j 1`
- PASS: `cargo test -p fret-render-wgpu --locked --lib renderer::path::tests::path_metrics_bounds_contain_tessellated_vertices -j 1`
- PASS: `cargo check -p fret-render-wgpu --locked --tests -j 1`
- PASS: `python tools/check_layering.py`
- PASS: `python tools/check_workstream_catalog.py`
- PASS: `git diff --check`

## Follow-on Policy

Do not reopen this lane for new path features. Future path work should start as a narrower additive
ADR/workstream only when it changes the path contract or adds a new backend/style surface.
