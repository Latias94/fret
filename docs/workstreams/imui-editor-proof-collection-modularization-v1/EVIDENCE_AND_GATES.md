# ImUi Editor Proof Collection Modularization v1 - Evidence & Gates

Goal: keep the structural collection modularization slice tied to one current repro set, one
explicit gate floor, and one bounded evidence set before anyone argues for shared helper or runtime
growth from a demo-local maintenance problem.

## Evidence anchors (current)

- `docs/workstreams/imui-editor-proof-collection-modularization-v1/DESIGN.md`
- `docs/workstreams/imui-editor-proof-collection-modularization-v1/M0_BASELINE_AUDIT_2026-04-23.md`
- `docs/workstreams/imui-editor-proof-collection-modularization-v1/M1_DEMO_LOCAL_COLLECTION_MODULE_SLICE_2026-04-23.md`
- `docs/workstreams/imui-editor-proof-collection-modularization-v1/CLOSEOUT_AUDIT_2026-04-23.md`
- `docs/workstreams/imui-editor-proof-collection-modularization-v1/TODO.md`
- `docs/workstreams/imui-editor-proof-collection-modularization-v1/MILESTONES.md`
- `docs/workstreams/imui-editor-proof-collection-modularization-v1/WORKSTREAM.json`
- `docs/workstreams/imui-collection-inline-rename-v1/CLOSEOUT_AUDIT_2026-04-23.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_NEXT_FOLLOW_ON_PRIORITY_AUDIT_2026-04-23.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/src/imui_editor_proof_demo/collection.rs`
- `apps/fret-examples/tests/imui_editor_collection_modularization_surface.rs`
- `apps/fret-examples/src/lib.rs`

## First-open repro surfaces

1. Current collection-first proof surface
   - `cargo run -p fret-demo --bin imui_editor_proof_demo`
2. Current lane-local surface floor
   - `cargo nextest run -p fret-examples --test imui_editor_collection_modularization_surface --no-fail-fast`
3. Current lane-local source-policy and unit-test floor
   - `cargo nextest run -p fret-examples --lib immediate_mode_workstream_freezes_the_p1_collection_modularization_follow_on proof_collection_drag_rect_normalizes_drag_direction proof_collection_commit_rename_rejects_empty_trimmed_label --no-fail-fast`

## Current focused gates

### Lane-local source-policy and unit-test floor

- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_freezes_the_p1_collection_modularization_follow_on proof_collection_drag_rect_normalizes_drag_direction proof_collection_commit_rename_rejects_empty_trimmed_label --no-fail-fast`

This floor currently proves:

- the repo keeps the new structural follow-on explicit,
- collection unit tests now live beside the module instead of the host file,
- and the lane stays a demo-local modularization slice rather than shared helper growth.

### Lane-local demo surface floor

- `cargo nextest run -p fret-examples --test imui_editor_collection_modularization_surface --no-fail-fast`
- `cargo nextest run -p fret-examples --test imui_editor_collection_box_select_surface --test imui_editor_collection_keyboard_owner_surface --test imui_editor_collection_delete_action_surface --test imui_editor_collection_context_menu_surface --test imui_editor_collection_zoom_surface --test imui_editor_collection_select_all_surface --test imui_editor_collection_rename_surface --no-fail-fast`

This floor currently proves:

- the host/module split stays explicit,
- the modularized collection module still exposes the full app-owned behavior surface,
- and the structural cleanup did not silently erase existing collection proof anchors.

### Lane hygiene gates

- `git diff --check`
- `python3 tools/check_workstream_catalog.py`
- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`
- `python3 -m json.tool docs/workstreams/imui-editor-proof-collection-modularization-v1/WORKSTREAM.json > /dev/null`

## Closeout posture

This folder is now closed.
Do not keep growing the gate package here by default.
If future pressure exceeds this structural slice, start a different narrower follow-on.
