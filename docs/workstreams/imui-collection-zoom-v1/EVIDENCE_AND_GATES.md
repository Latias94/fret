# ImUi Collection Zoom v1 - Evidence & Gates

Goal: keep the collection zoom/layout lane tied to one current repro set, one explicit gate floor,
and one bounded evidence set before anyone argues for shared helper or runtime growth.

## Evidence anchors (current)

- `docs/workstreams/imui-collection-zoom-v1/DESIGN.md`
- `docs/workstreams/imui-collection-zoom-v1/M0_BASELINE_AUDIT_2026-04-23.md`
- `docs/workstreams/imui-collection-zoom-v1/M1_APP_OWNED_ZOOM_LAYOUT_SLICE_2026-04-23.md`
- `docs/workstreams/imui-collection-zoom-v1/CLOSEOUT_AUDIT_2026-04-23.md`
- `docs/workstreams/imui-collection-zoom-v1/TODO.md`
- `docs/workstreams/imui-collection-zoom-v1/MILESTONES.md`
- `docs/workstreams/imui-collection-zoom-v1/WORKSTREAM.json`
- `docs/workstreams/imui-collection-context-menu-v1/CLOSEOUT_AUDIT_2026-04-23.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_PROOF_BUDGET_RULE_2026-04-12.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/tests/imui_editor_collection_zoom_surface.rs`
- `apps/fret-examples/src/lib.rs`
- `repo-ref/imgui/imgui_demo.cpp`

## First-open repro surfaces

1. Current collection-first proof surface
   - `cargo run -p fret-demo --bin imui_editor_proof_demo`
2. Current lane-local surface floor
   - `cargo nextest run -p fret-examples --test imui_editor_collection_zoom_surface --no-fail-fast`
3. Current lane-local source-policy and unit-test floor
   - `cargo nextest run -p fret-examples --lib immediate_mode_workstream_freezes_the_p1_collection_zoom_follow_on proof_collection_layout_metrics_fall_back_before_viewport_binding_exists proof_collection_zoom_request_updates_tile_extent_and_scroll_anchor proof_collection_zoom_request_ignores_non_primary_wheel --no-fail-fast`

## Current focused gates

### Lane-local source-policy and unit-test floor

- `cargo nextest run -p fret-examples --lib immediate_mode_workstream_freezes_the_p1_collection_zoom_follow_on proof_collection_layout_metrics_fall_back_before_viewport_binding_exists proof_collection_zoom_request_updates_tile_extent_and_scroll_anchor proof_collection_zoom_request_ignores_non_primary_wheel --no-fail-fast`

This floor currently proves:

- the repo keeps the new follow-on explicit,
- the lane stays separate from the closed context-menu lane and generic helper questions,
- and the app-owned collection zoom/layout math stays reviewable at the unit-test layer.

### Lane-local demo surface floor

- `cargo nextest run -p fret-examples --test imui_editor_collection_zoom_surface --no-fail-fast`

This floor currently proves:

- `imui_editor_proof_demo` keeps the collection zoom surface explicit,
- zoom/layout state, layout metrics, and scroll anchoring remain reviewable,
- and the slice stays app-owned instead of silently becoming a shared helper or runtime contract.

### Lane hygiene gates

- `git diff --check`
- `python3 tools/check_workstream_catalog.py`
- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`
- `python3 -m json.tool docs/workstreams/imui-collection-zoom-v1/WORKSTREAM.json > /dev/null`

## Closeout posture

This folder is now closed.
Do not keep growing the gate package here by default.
If future pressure exceeds this app-owned zoom/layout slice, start a different narrower follow-on.
