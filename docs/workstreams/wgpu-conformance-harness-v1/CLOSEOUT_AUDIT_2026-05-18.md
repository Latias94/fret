# WGPU Conformance Harness v1 — Closeout Audit

Date: 2026-05-18
Status: Closed

## Verdict

This narrow test-surface lane is closed. The shared WGPU readback/render harness has been extracted
into `crates/fret-render-wgpu/tests/support/mod.rs`, and the first path-related batch now uses it.

The behavior of the migrated tests did not change. Only duplicated harness code moved.

## Evidence

- `crates/fret-render-wgpu/tests/support/mod.rs`
- `crates/fret-render-wgpu/tests/path_base_conformance.rs`
- `crates/fret-render-wgpu/tests/path_stroke_style_v2_conformance.rs`
- `crates/fret-render-wgpu/tests/path_paint_conformance.rs`
- `crates/fret-render-wgpu/tests/path_material_paint_conformance.rs`

## Gates Run

2026-05-18:

- PASS: `cargo fmt --package fret-render-wgpu`
- PASS: `cargo test -p fret-render-wgpu --locked --test path_base_conformance --test path_stroke_style_v2_conformance --test path_paint_conformance --test path_material_paint_conformance -j 1`
- PASS: `cargo check -p fret-render-wgpu --locked --tests -j 1`
- PASS: `python tools/check_layering.py`
- PASS: `python tools/check_workstream_catalog.py`
- PASS: `git diff --check`

## Follow-On Policy

Do not reopen this lane broadly. If another test family has the same duplication pattern, split a
new narrower follow-on that names that family explicitly.
