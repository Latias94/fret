# Fret Examples Build Latency v1 - M25 IMUI Response Signals Source Gate - 2026-04-30

Status: complete

## Decision

Move the source-only IMUI response signals marker checks out of the monolithic `fret-examples`
Rust unit-test module and into `tools/gate_imui_facade_teaching_source.py`.

## Migrated Checks

- `imui_response_signals_demo_keeps_menu_and_combo_lifecycle_proof`
- `imui_response_signals_demo_keeps_canonical_menu_tab_trigger_response_proof`
- the `IMUI_RESPONSE_SIGNALS_DEMO` marker group from
  `selected_advanced_examples_prefer_grouped_state_actions_and_effects`

## Behavior

The IMUI facade/teaching source gate now covers:

- menu and combo lifecycle response markers,
- canonical menu/submenu/tab trigger response markers,
- grouped local-state read markers for the response signals demo.

No runtime behavior change is intended. This slice only moves pure source scanning away from Rust
unit tests that require compiling `fret-examples`.

## Evidence

- `tools/gate_imui_facade_teaching_source.py`
- `apps/fret-examples/src/lib.rs`
- `apps/fret-examples-imui/src/imui_response_signals_demo.rs`

## Result

The Rust `#[test]` count in `apps/fret-examples/src/lib.rs` dropped from 62 to 60, and the
`include_str!` count dropped from 279 to 278.

## Gates

```text
python tools/gate_imui_facade_teaching_source.py
python -m py_compile tools/gate_imui_facade_teaching_source.py
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
