# Fret Examples Build Latency v1 - M1 Source Policy Audit - 2026-04-29

Status: active

## Mechanical Count

After the adapter sortable and control-discoverability gates moved to Python, the remaining
`apps/fret-examples/src/lib.rs` test module still contains:

- 207 `include_str!` constants inside `authoring_surface_policy_tests`.
- 146 Rust `#[test]` functions.

The important smell is not the literal count by itself. The problem is that many tests only scan
checked-in source text, but Cargo still has to compile the full examples crate before those tests can
run.

## High-Confidence Python Gate Candidates

These are pure source-policy checks and should migrate before broader crate splitting:

- IMUI facade / teaching-surface source checks:
  - `first_party_imui_examples_keep_current_facade_teaching_surface`
  - `imui_hello_demo_prefers_root_fret_imui_facade_lane`
  - `imui_floating_windows_demo_prefers_root_fret_imui_facade_lane`
  - `imui_response_signals_demo_prefers_root_fret_imui_facade_lane`
  - `imui_interaction_showcase_demo_prefers_root_fret_imui_facade_lane`
  - `imui_shadcn_adapter_demo_prefers_root_fret_imui_facade_lane`
  - `imui_editor_proof_demo_prefers_root_fret_imui_entry_surface`

- IMUI source/doc freeze checks:
  - `immediate_mode_examples_docs_name_the_mounting_rule_for_imui_vs_imui_raw`
  - `immediate_mode_examples_docs_name_the_stable_identity_rule`
  - `immediate_mode_workstream_freezes_the_p0_imui_facade_internal_modularization_follow_on`
  - `immediate_mode_workstream_closes_the_p1_imui_next_gap_audit`

- Source-tree policy checks already shaped like script gates:
  - `examples_source_tree_prefers_curated_shadcn_facade_imports`
  - `examples_source_tree_limits_raw_shadcn_escape_hatches`
  - `examples_source_tree_avoids_raw_action_notify_helpers`
  - `examples_source_tree_keeps_setup_on_named_installers`
  - `examples_source_tree_limits_setup_with_to_explicit_one_off_case`
  - `first_party_examples_use_curated_shadcn_surface`

- Table/datatable source markers:
  - `table_examples_prefer_local_state_menu_bridges_over_clone_model`
  - `table_demo_uses_structured_table_debug_ids`
  - `datatable_examples_prefer_local_state_table_bridges`
  - `datatable_demo_uses_structured_table_debug_ids`
  - `table_stress_demo_uses_structured_table_debug_ids`

## Keep In Rust For Now

These should stay in Rust until a stronger split exists:

- Real unit tests that exercise functions, parsers, or runtime helpers, such as
  `parse_editor_theme_preset_key_accepts_supported_values` and
  `parse_editor_theme_preset_key_rejects_empty_and_unknown_values`.
- Layout or behavior tests that call Rust helpers rather than only scanning source text, such as
  `showcase_responsive_layout`.
- Any test whose failure needs Rust type-checking to be meaningful.

## Next Slice Recommendation

Move the IMUI facade / teaching-surface checks as a package. They share a small file roster, mostly
assert import/entrypoint vocabulary, and should become one script-level gate instead of several Rust
unit tests in the monolithic examples crate.

Keep the table/datatable markers as the slice after that, because they are isolated and have stable
file-level owners.
