# ImUi Collection Command Package v1 - Evidence & Gates

Goal: keep the closed broader app-owned collection command package tied to one reopen repro target,
one explicit closeout gate floor, and one bounded evidence set before anyone argues for shared
helper or runtime growth from a single proof surface.

## Evidence anchors (current)

- `docs/workstreams/imui-collection-command-package-v1/DESIGN.md`
- `docs/workstreams/imui-collection-command-package-v1/M0_BASELINE_AUDIT_2026-04-23.md`
- `docs/workstreams/imui-collection-command-package-v1/M1_APP_OWNED_DUPLICATE_COMMAND_SLICE_2026-04-23.md`
- `docs/workstreams/imui-collection-command-package-v1/M2_APP_OWNED_RENAME_TRIGGER_SLICE_2026-04-23.md`
- `docs/workstreams/imui-collection-command-package-v1/CLOSEOUT_AUDIT_2026-04-23.md`
- `docs/workstreams/imui-collection-command-package-v1/TODO.md`
- `docs/workstreams/imui-collection-command-package-v1/MILESTONES.md`
- `docs/workstreams/imui-collection-command-package-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-collection-command-package-v1/WORKSTREAM.json`
- `docs/workstreams/imui-collection-second-proof-surface-v1/DESIGN.md`
- `docs/workstreams/imui-editor-proof-collection-modularization-v1/CLOSEOUT_AUDIT_2026-04-23.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_NEXT_FOLLOW_ON_PRIORITY_AUDIT_2026-04-23.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- `apps/fret-examples/src/imui_editor_proof_demo/collection.rs`
- `apps/fret-examples/tests/imui_editor_collection_command_package_surface.rs`
- `apps/fret-examples/src/lib.rs`

## First-open repro surfaces

1. Current collection-first proof surface
   - `cargo run -p fret-demo --bin imui_editor_proof_demo`
2. Current command-package surface floor
   - `cargo nextest run -p fret-examples --test imui_editor_collection_command_package_surface --test imui_editor_collection_rename_surface --no-fail-fast`
3. Current command-package closeout/source-policy floor
   - `cargo nextest run -p fret-examples --lib immediate_mode_workstream_closes_the_p1_collection_command_package_follow_on proof_collection_duplicate_shortcut_matches_primary_d_only proof_collection_duplicate_selection_reselects_visible_copies_and_preserves_active_copy proof_collection_duplicate_selection_uses_unique_copy_suffixes_when_copy_exists proof_collection_begin_rename_session_prefers_active_visible_asset proof_collection_begin_rename_session_falls_back_to_first_visible_asset proof_collection_rename_shortcut_matches_plain_f2_only --no-fail-fast`

## Current focused gates

### Lane-local closeout source-policy and unit-test floor

- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_closes_the_p1_collection_command_package_follow_on proof_collection_duplicate_shortcut_matches_primary_d_only proof_collection_duplicate_selection_reselects_visible_copies_and_preserves_active_copy proof_collection_duplicate_selection_uses_unique_copy_suffixes_when_copy_exists proof_collection_begin_rename_session_prefers_active_visible_asset proof_collection_begin_rename_session_falls_back_to_first_visible_asset proof_collection_rename_shortcut_matches_plain_f2_only --no-fail-fast`

This floor currently proves:

- the repo keeps the command-package lane explicit as a closed closeout record,
- duplicate-selected now lives on one app-owned command path,
- the explicit rename trigger reuse stays local to the current proof surface,
- the current package closes without a third verb,
- and the next default follow-on is the second proof surface rather than shared helper or runtime
  widening.

### Lane-local demo surface floor

- `cargo nextest run -p fret-examples --test imui_editor_collection_command_package_surface --test imui_editor_collection_context_menu_surface --test imui_editor_collection_delete_action_surface --test imui_editor_collection_rename_surface --no-fail-fast`

This floor currently proves:

- the duplicate-selected and explicit rename-trigger slices stay explicit and app-owned,
- keyboard/button/context-menu routing still lives inside the collection module,
- and the closed package does not regress the already-landed collection command substrate.

### Lane hygiene gates

- `git diff --check`
- `python3 tools/check_workstream_catalog.py`
- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`
- `python3 -m json.tool docs/workstreams/imui-collection-command-package-v1/WORKSTREAM.json > /dev/null`
