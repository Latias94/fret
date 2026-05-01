# Fret Examples Build Latency v1 - M62 Grouped State Source Gate Closure - 2026-05-01

Status: complete

## Decision

Move the remaining grouped state/action/effect/model-read source-policy matrices out of the
monolithic `fret-examples` Rust unit-test module and into
`tools/examples_source_tree_policy/grouped_state.py`.

## Migrated Checks

- `selected_advanced_examples_prefer_grouped_state_actions_and_effects`
- `selected_element_context_examples_prefer_handle_first_tracked_model_reads`

## Behavior

The examples source-tree policy gate now owns the remaining source-only checks for advanced
examples and element-context examples. The migrated gate keeps the current grouped-state,
action/effect, selector, and handle-first model-read markers without compiling `fret-examples`
tests.

During migration, stale Rust-only markers were corrected to the current source shape:

- `imui_shadcn_adapter_demo` now checks `switch_model_with_options`.
- `editor_notes_demo` now checks the four-field summary selector and no longer forbids the
  intentional `cx.elements().keyed_slot_state(...)` draft-controller access.

## Evidence

- `tools/examples_source_tree_policy/grouped_state.py`
- `tools/examples_source_tree_policy/gate.py`
- `tools/gate_examples_source_tree_policy.py`
- `apps/fret-examples/src/lib.rs`

## Result

After this migration, `apps/fret-examples/src/lib.rs` keeps only the two real
`parse_editor_theme_preset_key_*` Rust behavior tests. The Rust `#[test]` count dropped from 4 to
2, and the `include_str!` count dropped from 25 to 0.

## Gates

```text
python tools/gate_examples_source_tree_policy.py
python -m py_compile tools/gate_examples_source_tree_policy.py tools/examples_source_tree_policy/grouped_state.py tools/examples_source_tree_policy/gate.py tools/check_workstream_catalog.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/fret-examples-build-latency-v1/WORKSTREAM.json
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
cargo nextest run -p fret-examples --lib parse_editor_theme_preset_key --no-fail-fast
```
