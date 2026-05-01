# ImUi Collection Rename v1 - Evidence & Gates

Goal: keep the collection rename lane tied to one current repro set, one explicit gate floor,
and one bounded evidence set before anyone argues for shared helper or runtime growth.

## Evidence anchors (current)

- `docs/workstreams/imui-collection-rename-v1/DESIGN.md`
- `docs/workstreams/imui-collection-rename-v1/M0_BASELINE_AUDIT_2026-04-23.md`
- `docs/workstreams/imui-collection-rename-v1/M1_APP_OWNED_RENAME_SLICE_2026-04-23.md`
- `docs/workstreams/imui-collection-rename-v1/CLOSEOUT_AUDIT_2026-04-23.md`
- `docs/workstreams/imui-collection-rename-v1/TODO.md`
- `docs/workstreams/imui-collection-rename-v1/MILESTONES.md`
- `docs/workstreams/imui-collection-rename-v1/WORKSTREAM.json`
- `docs/workstreams/imui-collection-select-all-v1/CLOSEOUT_AUDIT_2026-04-23.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_PROOF_BUDGET_RULE_2026-04-12.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/tests/imui_editor_collection_rename_surface.rs`
- `apps/fret-examples/src/lib.rs`
- `tools/gate_imui_workstream_source.py`
- `repo-ref/imgui/imgui.h`

## First-open repro surfaces

1. Current collection-first proof surface
   - `cargo run -p fret-demo --bin imui_editor_proof_demo`
2. Current lane-local surface floor
   - `cargo nextest run -p fret-examples --test imui_editor_collection_rename_surface --no-fail-fast`
3. Current lane-local source-policy and unit-test floor
   - `python tools/gate_imui_workstream_source.py`
   - `cargo nextest run -p fret-examples --lib proof_collection_begin_rename_session_prefers_active_visible_asset proof_collection_begin_rename_session_falls_back_to_first_visible_asset proof_collection_rename_shortcut_matches_plain_f2_only proof_collection_commit_rename_updates_label_without_touching_order_or_ids --no-fail-fast`

## Current focused gates

### Lane-local source-policy and unit-test floor

- `python tools/gate_imui_workstream_source.py`
- `cargo nextest run -p fret-examples --lib proof_collection_begin_rename_session_prefers_active_visible_asset proof_collection_begin_rename_session_falls_back_to_first_visible_asset proof_collection_rename_shortcut_matches_plain_f2_only proof_collection_commit_rename_updates_label_without_touching_order_or_ids --no-fail-fast`

This floor currently proves:

- the repo keeps the new follow-on explicit,
- the lane stays separate from the closed select-all lane and generic key-owner/helper questions,
- and the app-owned collection rename policy stays reviewable at the unit-test layer.

### Lane-local demo surface floor

- `cargo nextest run -p fret-examples --test imui_editor_collection_rename_surface --no-fail-fast`

This floor currently proves:

- `imui_editor_proof_demo` keeps the collection rename surface explicit,
- F2, context-menu entry, popup modal, and label-only rename commit remain reviewable,
- and the slice stays app-owned instead of silently becoming a shared helper or runtime contract.

### Lane hygiene gates

- `git diff --check`
- `python3 tools/check_workstream_catalog.py`
- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`
- `python3 -m json.tool docs/workstreams/imui-collection-rename-v1/WORKSTREAM.json > /dev/null`

## Closeout posture

This folder is now closed.
Do not keep growing the gate package here by default.
If future pressure exceeds this app-owned rename slice, start a different narrower follow-on.
