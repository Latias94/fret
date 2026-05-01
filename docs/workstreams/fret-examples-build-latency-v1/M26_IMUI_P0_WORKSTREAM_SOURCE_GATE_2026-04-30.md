# Fret Examples Build Latency v1 - M26 IMUI P0 Workstream Source Gate - 2026-04-30

Status: complete

## Decision

Move the source-only IMUI P0 response/key-owner workstream freeze checks out of the monolithic
`fret-examples` Rust unit-test module and into `tools/gate_imui_workstream_source.py`.

## Migrated Checks

- `immediate_mode_workstream_freezes_the_p0_response_status_lifecycle_follow_on`
- `immediate_mode_workstream_freezes_the_p0_key_owner_surface_follow_on`
- `immediate_mode_key_owner_surface_m2_no_new_surface_verdict_is_explicit`

## Behavior

The new IMUI workstream source gate now covers:

- the `imui-response-status-lifecycle-v1` design and lane-state freeze markers,
- the `imui-key-owner-surface-v1` design, M1 proof roster, M2 no-new-surface verdict, closeout, and
  lane-state freeze markers,
- and the umbrella `imui-editor-grade-product-closure-v1` follow-on routing markers.

The closed IMUI workstreams now point their source-policy gate at the Python gate instead of the
deleted `fret-examples` Rust source-marker tests. No runtime behavior change is intended.

## Related Cleanup

While checking for deleted Rust source-marker test names, this slice also re-pointed stale closed
lane gate commands from the earlier M24/M25 source migrations to the existing
`tools/gate_imui_facade_teaching_source.py` gate. That keeps the closed control-chrome and
menu/tab-trigger response records runnable after their source-only proof moved out of
`fret-examples`.

## Evidence

- `tools/gate_imui_workstream_source.py`
- `apps/fret-examples/src/lib.rs`
- `docs/workstreams/imui-response-status-lifecycle-v1/WORKSTREAM.json`
- `docs/workstreams/imui-key-owner-surface-v1/WORKSTREAM.json`
- `docs/workstreams/imui-control-chrome-fearless-refactor-v1/WORKSTREAM.json`
- `docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/WORKSTREAM.json`

## Result

The Rust `#[test]` count in `apps/fret-examples/src/lib.rs` dropped from 60 to 57, and the
`include_str!` count dropped from 278 to 271.

## Gates

```text
python tools/gate_imui_workstream_source.py
python -m py_compile tools/gate_imui_workstream_source.py
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
