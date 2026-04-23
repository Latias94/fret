# ImUi Collection Keyboard Owner v1 - Evidence & Gates

Goal: keep the collection keyboard-owner lane tied to one current repro set, one explicit gate
floor, and one bounded evidence set before anyone argues for shared helper or generic key-owner
growth.

## Evidence anchors (current)

- `docs/workstreams/imui-collection-keyboard-owner-v1/DESIGN.md`
- `docs/workstreams/imui-collection-keyboard-owner-v1/M0_BASELINE_AUDIT_2026-04-22.md`
- `docs/workstreams/imui-collection-keyboard-owner-v1/M1_APP_OWNED_KEYBOARD_OWNER_SLICE_2026-04-22.md`
- `docs/workstreams/imui-collection-keyboard-owner-v1/CLOSEOUT_AUDIT_2026-04-22.md`
- `docs/workstreams/imui-collection-keyboard-owner-v1/TODO.md`
- `docs/workstreams/imui-collection-keyboard-owner-v1/MILESTONES.md`
- `docs/workstreams/imui-collection-keyboard-owner-v1/WORKSTREAM.json`
- `docs/workstreams/imui-collection-box-select-v1/CLOSEOUT_AUDIT_2026-04-22.md`
- `docs/workstreams/imui-key-owner-surface-v1/CLOSEOUT_AUDIT_2026-04-21.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_PROOF_BUDGET_RULE_2026-04-12.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/tests/imui_editor_collection_keyboard_owner_surface.rs`
- `apps/fret-examples/src/lib.rs`
- `repo-ref/imgui/imgui_demo.cpp`
- `repo-ref/imgui/imgui.h`

## First-open repro surfaces

Use these before reading older historical `imui` notes in depth:

1. Current collection-first proof surface
   - `cargo run -p fret-demo --bin imui_editor_proof_demo`
2. Current lane-local surface floor
   - `cargo nextest run -p fret-examples --test imui_editor_collection_keyboard_owner_surface --no-fail-fast`
3. Current lane-local source-policy and unit-test floor
   - `cargo nextest run -p fret-examples --lib immediate_mode_workstream_freezes_the_p1_collection_keyboard_owner_follow_on proof_collection_keyboard_arrow_replaces_selection_and_moves_active_tile proof_collection_keyboard_shift_navigation_extends_range_from_anchor proof_collection_keyboard_escape_clears_selection_but_keeps_active_tile proof_collection_keyboard_ignores_primary_modifier_shortcuts --no-fail-fast`

Current status summary:

- the collection-first proof demo now ships app-owned collection keyboard-owner depth,
- the logic remains app-owned in the proof surface,
- and the generic key-owner / shared-helper widening verdict still stays closed on current
  evidence.

## Current focused gates

### Lane-local source-policy and unit-test floor

- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_freezes_the_p1_collection_keyboard_owner_follow_on proof_collection_keyboard_arrow_replaces_selection_and_moves_active_tile proof_collection_keyboard_shift_navigation_extends_range_from_anchor proof_collection_keyboard_escape_clears_selection_but_keeps_active_tile proof_collection_keyboard_ignores_primary_modifier_shortcuts --no-fail-fast`

This floor currently proves:

- the repo keeps the new follow-on explicit,
- the lane stays separate from both the closed box-select lane and the closed generic key-owner
  lane,
- and the app-owned collection keyboard selection math stays reviewable at the unit-test layer.

### Lane-local demo surface floor

- `cargo nextest run -p fret-examples --test imui_editor_collection_keyboard_owner_surface --no-fail-fast`

This floor currently proves:

- `imui_editor_proof_demo` keeps the collection keyboard-owner surface explicit,
- the focusable scope and active-tile markers remain reviewable,
- and the slice stays app-owned instead of silently becoming a shared helper or generic key-owner
  facade.

### Lane hygiene gates

- `git diff --check`
- `python3 tools/check_workstream_catalog.py`
- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`
- `python3 -m json.tool docs/workstreams/imui-collection-keyboard-owner-v1/WORKSTREAM.json > /dev/null`

## Closeout posture

This folder is now closed.
Do not keep growing the gate package here by default.
If future pressure exceeds this app-owned keyboard-owner slice, start a different narrower
follow-on.
