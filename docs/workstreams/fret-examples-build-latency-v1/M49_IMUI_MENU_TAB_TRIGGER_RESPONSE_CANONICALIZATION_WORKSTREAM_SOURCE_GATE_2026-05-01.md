# Fret Examples Build Latency v1 - M49 IMUI Menu/Tab Trigger Response Canonicalization Workstream Source Gate - 2026-05-01

Status: complete

## Decision

Move the source-only IMUI menu/tab trigger response canonicalization closeout check out of the
monolithic `fret-examples` Rust unit-test module and into `tools/gate_imui_workstream_source.py`.

## Migrated Check

- `immediate_mode_workstream_freezes_the_p0_menu_tab_trigger_response_canonicalization_follow_on`

## Behavior

The IMUI workstream source gate now covers:

- the `imui-menu-tab-trigger-response-canonicalization-v1` design, evidence, final status, and
  lane-state markers,
- the repo-wide roadmap/workstream-index/todo-tracker references that keep the canonical naming
  cleanup discoverable,
- and the Python source-policy gate marker that replaces the deleted Rust source-marker test.

The real `fret-imui` helper behavior/canonical response tests and IMUI facade teaching source gate
remain behavior/source floors. Only the workstream/document freeze portion moved to Python.

## Evidence

- `tools/gate_imui_workstream_source.py`
- `apps/fret-examples/src/lib.rs`
- `docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/WORKSTREAM.json`
- `docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-menu-tab-trigger-response-canonicalization-v1/FINAL_STATUS.md`

## Result

After the paired menu/tab trigger-response migration, the Rust `#[test]` count in
`apps/fret-examples/src/lib.rs` dropped from 34 to 32, and the `include_str!` count dropped from
149 to 144.

## Gates

```text
python tools/gate_imui_workstream_source.py
python tools/gate_imui_facade_teaching_source.py
cargo nextest run -p fret-imui begin_menu_helper_toggles_popup_and_closes_after_command_activate begin_submenu_helper_opens_nested_menu_and_tracks_expanded_semantics tab_bar_helper_switches_selected_panel_and_updates_selection_model tab_item_activate_shortcut_is_scoped_to_focused_trigger menu_and_submenu_helpers_report_toggle_and_trigger_edges tab_bar_helper_reports_selected_change_and_trigger_edges --no-fail-fast
python -m py_compile tools/gate_imui_workstream_source.py
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
