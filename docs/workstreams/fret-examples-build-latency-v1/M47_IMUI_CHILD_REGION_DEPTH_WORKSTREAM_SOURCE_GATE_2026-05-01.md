# Fret Examples Build Latency v1 - M47 IMUI Child Region Depth Workstream Source Gate - 2026-05-01

Status: complete

## Decision

Move the source-only IMUI child-region depth closeout check out of the monolithic `fret-examples`
Rust unit-test module and into `tools/gate_imui_workstream_source.py`.

## Migrated Check

- `immediate_mode_workstream_freezes_the_p1_child_region_depth_follow_on`

## Behavior

The IMUI workstream source gate now covers:

- the `imui-child-region-depth-v1` design, M0/M1/M2 notes, evidence, closeout, and lane-state
  markers,
- the umbrella TODO, roadmap, workstream-index, and todo-tracker references that keep the closed
  child-region depth verdict discoverable,
- and the Python source-policy gate marker that replaces the deleted Rust source-marker test.

The real `fret-ui-kit`, `fret-imui`, and pane-proof surface gates remain behavior floors. Only the
source-policy/document freeze portion moved to Python.

## Evidence

- `tools/gate_imui_workstream_source.py`
- `apps/fret-examples/src/lib.rs`
- `docs/workstreams/imui-child-region-depth-v1/WORKSTREAM.json`
- `docs/workstreams/imui-child-region-depth-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-child-region-depth-v1/CLOSEOUT_AUDIT_2026-04-22.md`

## Result

The Rust `#[test]` count in `apps/fret-examples/src/lib.rs` dropped from 35 to 34, and the
`include_str!` count dropped from 155 to 149.

## Gates

```text
python tools/gate_imui_workstream_source.py
cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke --no-fail-fast
cargo nextest run -p fret-imui child_region_helper_stacks_content_and_forwards_scroll_options child_region_helper_can_host_menu_bar_and_popup_menu child_region_helper_can_switch_between_framed_and_bare_chrome --no-fail-fast
python -m py_compile tools/gate_imui_workstream_source.py
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
