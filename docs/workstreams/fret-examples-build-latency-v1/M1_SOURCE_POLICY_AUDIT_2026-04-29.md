# Fret Examples Build Latency v1 - M1 Source Policy Audit - 2026-04-29

Status: active

## Mechanical Count

After the adapter sortable, control-discoverability, IMUI facade teaching, and table/datatable gates
moved to Python, the remaining `apps/fret-examples/src/lib.rs` test module still contains:

- 281 `include_str!` occurrences inside `authoring_surface_policy_tests`.
- 128 Rust `#[test]` functions.

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
  - `imui_editor_proof_demo_prefers_root_fret_imui_entry_surface`

Migrated after this audit:

- `imui_shadcn_adapter_demo_prefers_root_fret_imui_facade_lane` is now covered by
  `tools/gate_imui_shadcn_adapter_sortable_table_source.py` because the sortable demo proof already
  uses the adapter source gate as its current source proof.
- The remaining IMUI facade / teaching-surface source checks in this group are now covered by
  `tools/gate_imui_facade_teaching_source.py`.
- The matching umbrella docs/source classification checks are also covered by
  `tools/gate_imui_facade_teaching_source.py`: golden-pair roster, mounting rule, stable identity
  rule, helper-widening proof budget, hello-demo demotion, and node-graph retained compatibility.

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

Migrated after this audit:

- The table/datatable source marker group is now covered by
  `tools/gate_table_source_policy.py`.

## Keep In Rust For Now

These should stay in Rust until a stronger split exists:

- Real unit tests that exercise functions, parsers, or runtime helpers, such as
  `parse_editor_theme_preset_key_accepts_supported_values` and
  `parse_editor_theme_preset_key_rejects_empty_and_unknown_values`.
- Layout or behavior tests that call Rust helpers rather than only scanning source text, such as
  `showcase_responsive_layout`.
- Any test whose failure needs Rust type-checking to be meaningful.

## Next Slice Recommendation

Move the source-tree policy checks as a package. They scan the same curated shadcn surface and
escape-hatch vocabulary, so they should become one script-level gate instead of several Rust unit
tests in the monolithic examples crate.

After that source-policy slice, switch to M2 and decide whether `fret-demo` should stop linking
through the full examples library for heavy demo families.
