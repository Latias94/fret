# Fret Examples Build Latency v1 - M52 IMUI P3 Multiwindow Package Source Gate - 2026-05-01

Status: complete

## Decision

Move the source-only IMUI P3 multi-window runner-gap and bounded campaign package checks out of the
monolithic `fret-examples` Rust unit-test module and into `tools/gate_imui_workstream_source.py`.

## Migrated Checks

- `immediate_mode_workstream_freezes_the_p3_multiwindow_runner_gap_checklist`
- `immediate_mode_workstream_freezes_the_p3_bounded_multiwindow_parity_package`

## Behavior

The IMUI workstream source gate now covers:

- the P3 runner/backend ownership checklist,
- the bounded multi-window campaign package note,
- the `imui-p3-multiwindow-parity` campaign manifest,
- the docking and macOS parity evidence markers needed by the umbrella package,
- and the product-closure gate state that points these source-policy checks at Python.

The real P3 behavior gates remain campaign validation and launched campaign execution. Later
mixed-DPI, placement, Wayland, and monitor-topology source freezes remain in Rust for now because
they share constants and need separate ownership checks.

## Evidence

- `tools/gate_imui_workstream_source.py`
- `apps/fret-examples/src/lib.rs`
- `docs/workstreams/imui-editor-grade-product-closure-v1/WORKSTREAM.json`
- `docs/workstreams/imui-editor-grade-product-closure-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P3_MULTIWINDOW_RUNNER_GAP_CHECKLIST_2026-04-12.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P3_BOUNDED_MULTIWINDOW_PARITY_PACKAGE_2026-04-12.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity.md`
- `docs/workstreams/standalone/macos-docking-multiwindow-imgui-parity.md`
- `tools/diag-campaigns/imui-p3-multiwindow-parity.json`

## Result

After this migration, the Rust `#[test]` count in `apps/fret-examples/src/lib.rs` dropped from 25
to 23, and the `include_str!` count dropped from 131 to 128.

## Gates

```text
python tools/gate_imui_workstream_source.py
python -m py_compile tools/gate_imui_workstream_source.py tools/check_workstream_catalog.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-editor-grade-product-closure-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/fret-examples-build-latency-v1/WORKSTREAM.json
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
