# ImUi Collection Box Select v1 - Evidence & Gates

Goal: keep the collection box-select lane tied to one current repro set, one explicit gate floor,
and one bounded evidence set before anyone argues for shared helper growth.

## Evidence anchors (current)

- `docs/workstreams/imui-collection-box-select-v1/DESIGN.md`
- `docs/workstreams/imui-collection-box-select-v1/M0_BASELINE_AUDIT_2026-04-22.md`
- `docs/workstreams/imui-collection-box-select-v1/M1_BACKGROUND_BOX_SELECT_SLICE_2026-04-22.md`
- `docs/workstreams/imui-collection-box-select-v1/CLOSEOUT_AUDIT_2026-04-22.md`
- `docs/workstreams/imui-collection-box-select-v1/TODO.md`
- `docs/workstreams/imui-collection-box-select-v1/MILESTONES.md`
- `docs/workstreams/imui-collection-box-select-v1/WORKSTREAM.json`
- `docs/workstreams/imui-collection-pane-proof-v1/CLOSEOUT_AUDIT_2026-04-21.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_PROOF_BUDGET_RULE_2026-04-12.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- `docs/adr/0084-virtualized-accessibility-and-collection-semantics.md`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/tests/imui_editor_collection_box_select_surface.rs`
- `apps/fret-examples/src/lib.rs`
- `tools/gate_imui_workstream_source.py`
- `repo-ref/imgui/imgui_demo.cpp`
- `repo-ref/imgui/imgui.h`

## First-open repro surfaces

Use these before reading older historical `imui` notes in depth:

1. Current collection-first proof surface
   - `cargo run -p fret-demo --bin imui_editor_proof_demo`
2. Current lane-local surface floor
   - `cargo nextest run -p fret-examples --test imui_editor_collection_box_select_surface --no-fail-fast`
3. Current lane-local source-policy and unit-test floor
   - `python tools/gate_imui_workstream_source.py`
   - `cargo nextest run -p fret-examples --lib proof_collection_drag_rect_normalizes_drag_direction proof_collection_box_select_replace_uses_visible_collection_order proof_collection_box_select_append_preserves_baseline_and_adds_hits --no-fail-fast`

Current status summary:

- the collection-first proof demo now ships background-only marquee / box-select,
- the logic remains app-owned in the proof surface,
- and the frozen proof-budget rule still blocks shared helper widening on current evidence.

## Current focused gates

### Lane-local source-policy and unit-test floor

- `python tools/gate_imui_workstream_source.py`
- `cargo nextest run -p fret-examples --lib proof_collection_drag_rect_normalizes_drag_direction proof_collection_box_select_replace_uses_visible_collection_order proof_collection_box_select_append_preserves_baseline_and_adds_hits --no-fail-fast`

This floor currently proves:

- the repo keeps the new follow-on explicit,
- the lane stays separate from the older collection/pane proof closeout,
- and the box-select selection math stays reviewable at the unit-test layer.

### Lane-local demo surface floor

- `cargo nextest run -p fret-examples --test imui_editor_collection_box_select_surface --no-fail-fast`

This floor currently proves:

- `imui_editor_proof_demo` keeps the collection box-select surface explicit,
- the pointer-region and marquee markers remain reviewable,
- and the slice stays app-owned instead of silently becoming a shared helper.

### Lane hygiene gates

- `git diff --check`
- `python3 tools/check_workstream_catalog.py`
- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`
- `python3 -m json.tool docs/workstreams/imui-collection-box-select-v1/WORKSTREAM.json > /dev/null`

## Closeout posture

This folder is now closed.
Do not keep growing the gate package here by default.
If future pressure exceeds background-only box-select, start a different narrower follow-on.
