# ImUi Collection Delete Action v1 - Evidence & Gates

Goal: keep the collection delete-selected lane tied to one current repro set, one explicit gate
floor, and one bounded evidence set before anyone argues for shared helper or broader collection
command growth.

## Evidence anchors (current)

- `docs/workstreams/imui-collection-delete-action-v1/DESIGN.md`
- `docs/workstreams/imui-collection-delete-action-v1/M0_BASELINE_AUDIT_2026-04-22.md`
- `docs/workstreams/imui-collection-delete-action-v1/M1_APP_OWNED_DELETE_ACTION_SLICE_2026-04-22.md`
- `docs/workstreams/imui-collection-delete-action-v1/CLOSEOUT_AUDIT_2026-04-22.md`
- `docs/workstreams/imui-collection-delete-action-v1/TODO.md`
- `docs/workstreams/imui-collection-delete-action-v1/MILESTONES.md`
- `docs/workstreams/imui-collection-delete-action-v1/WORKSTREAM.json`
- `docs/workstreams/imui-collection-keyboard-owner-v1/CLOSEOUT_AUDIT_2026-04-22.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_PROOF_BUDGET_RULE_2026-04-12.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/tests/imui_editor_collection_delete_action_surface.rs`
- `apps/fret-examples/src/lib.rs`
- `tools/gate_imui_workstream_source.py`
- `repo-ref/imgui/imgui_demo.cpp`

## First-open repro surfaces

Use these before reading older historical `imui` notes in depth:

1. Current collection-first proof surface
   - `cargo run -p fret-demo --bin imui_editor_proof_demo`
2. Current lane-local surface floor
   - `cargo nextest run -p fret-examples --test imui_editor_collection_delete_action_surface --no-fail-fast`
3. Current lane-local source-policy and unit-test floor
   - `python tools/gate_imui_workstream_source.py`
   - `cargo nextest run -p fret-examples --lib proof_collection_delete_selection_removes_selected_assets_and_refocuses_next_visible_item proof_collection_delete_selection_picks_previous_visible_item_at_end --no-fail-fast`

Current status summary:

- the collection-first proof demo now ships app-owned collection delete-selected depth,
- the logic remains app-owned in the proof surface,
- and the shared-helper / runtime widening verdict still stays closed on current evidence.

## Current focused gates

### Lane-local source-policy and unit-test floor

- `python tools/gate_imui_workstream_source.py`
- `cargo nextest run -p fret-examples --lib proof_collection_delete_selection_removes_selected_assets_and_refocuses_next_visible_item proof_collection_delete_selection_picks_previous_visible_item_at_end --no-fail-fast`

This floor currently proves:

- the repo keeps the new follow-on explicit,
- the lane stays separate from both the closed keyboard-owner lane and the closed generic key-owner
  lane,
- and the app-owned collection delete-selection math stays reviewable at the unit-test layer.

### Lane-local demo surface floor

- `cargo nextest run -p fret-examples --test imui_editor_collection_delete_action_surface --no-fail-fast`

This floor currently proves:

- `imui_editor_proof_demo` keeps the collection delete-selected surface explicit,
- the mutable asset model, action button, and key handler markers remain reviewable,
- and the slice stays app-owned instead of silently becoming a shared helper or generic command
  facade.

### Lane hygiene gates

- `git diff --check`
- `python3 tools/check_workstream_catalog.py`
- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`
- `python3 -m json.tool docs/workstreams/imui-collection-delete-action-v1/WORKSTREAM.json > /dev/null`

## Closeout posture

This folder is now closed.
Do not keep growing the gate package here by default.
If future pressure exceeds this app-owned delete-selected slice, start a different narrower
follow-on.
