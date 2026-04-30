# Fret Examples Build Latency v1 - M24 IMUI Interaction Showcase Source Gate - 2026-04-30

Status: complete

## Decision

Move the source-only IMUI interaction showcase marker checks out of the monolithic
`fret-examples` Rust unit-test module and into `tools/gate_imui_facade_teaching_source.py`.

## Migrated Checks

- `imui_interaction_showcase_demo_avoids_fixed_compact_lab_width_workaround`
- the `IMUI_INTERACTION_SHOWCASE_DEMO` marker group from
  `selected_advanced_examples_prefer_grouped_state_actions_and_effects`

## Behavior

The IMUI facade/teaching source gate now covers:

- compact rail sizing markers for `imui_interaction_showcase_demo.rs`,
- removal of the old fixed compact side-column workaround,
- grouped state/action markers for the showcase demo's local state reads and model-backed controls.

No runtime behavior change is intended. This slice only moves pure source scanning away from Rust
unit tests that require compiling `fret-examples`.

## Evidence

- `tools/gate_imui_facade_teaching_source.py`
- `apps/fret-examples/src/lib.rs`
- `apps/fret-examples-imui/src/imui_interaction_showcase_demo.rs`

## Result

The Rust `#[test]` count in `apps/fret-examples/src/lib.rs` dropped from 63 to 62, and the
`include_str!` count dropped from 280 to 279.

## Gates

```text
python tools/gate_imui_facade_teaching_source.py
python -m py_compile tools/gate_imui_facade_teaching_source.py
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
