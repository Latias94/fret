# ImUi Active Trigger Behavior Kernel v1 Evidence And Gates

Status: closed gate list
Last updated: 2026-04-24

## Smallest Repro

```bash
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-imui switch_model_reports_changed_once_after_click --no-fail-fast
cargo nextest run -p fret-imui interaction_menu_tabs:: --no-fail-fast
cargo nextest run -p fret-imui menu_item_lifecycle_edges_follow_press_session --no-fail-fast
```

## Required Gates

```bash
cargo fmt --package fret-ui-kit
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-imui switch_model_reports_changed_once_after_click --no-fail-fast
cargo nextest run -p fret-imui interaction_menu_tabs:: --no-fail-fast
cargo nextest run -p fret-imui menu_item_lifecycle_edges_follow_press_session --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui --no-fail-fast
cargo nextest run -p fret-imui --no-fail-fast
cargo build -p fret-demo --bin imui_interaction_showcase_demo
cargo build -p fret-demo --bin imui_editor_proof_demo
python3 tools/check_layering.py
python3 tools/check_workstream_catalog.py
python3 -m json.tool docs/workstreams/imui-active-trigger-behavior-kernel-v1/WORKSTREAM.json
git diff --check
```

## Evidence Anchors

Lane docs:

- `docs/workstreams/imui-active-trigger-behavior-kernel-v1/WORKSTREAM.json`
- `docs/workstreams/imui-active-trigger-behavior-kernel-v1/DESIGN.md`
- `docs/workstreams/imui-active-trigger-behavior-kernel-v1/M0_M1_ACTIVE_TRIGGER_SLICE_2026-04-24.md`
- `docs/workstreams/imui-active-trigger-behavior-kernel-v1/CLOSEOUT_AUDIT_2026-04-24.md`
- `docs/workstreams/imui-active-trigger-behavior-kernel-v1/TODO.md`
- `docs/workstreams/imui-active-trigger-behavior-kernel-v1/MILESTONES.md`
- `docs/workstreams/imui-active-trigger-behavior-kernel-v1/EVIDENCE_AND_GATES.md`

Prior lane evidence:

- `docs/workstreams/imui-item-behavior-kernel-v1/CLOSEOUT_AUDIT_2026-04-24.md`

Implementation anchors:

- `ecosystem/fret-ui-kit/src/imui/active_trigger_behavior.rs`
- `ecosystem/fret-ui-kit/src/imui/boolean_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/menu_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/menu_family_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/tab_family_controls.rs`
- `ecosystem/fret-imui/src/tests`

## Verified Gates

Passed on 2026-04-24:

```bash
cargo fmt --package fret-ui-kit
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-imui switch_model_reports_changed_once_after_click --no-fail-fast
cargo nextest run -p fret-imui interaction_menu_tabs:: --no-fail-fast
cargo nextest run -p fret-imui menu_item_lifecycle_edges_follow_press_session --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui --no-fail-fast
cargo nextest run -p fret-imui --no-fail-fast
cargo build -p fret-demo --bin imui_interaction_showcase_demo
cargo build -p fret-demo --bin imui_editor_proof_demo
python3 tools/check_layering.py
python3 tools/check_workstream_catalog.py
python3 -m json.tool docs/workstreams/imui-active-trigger-behavior-kernel-v1/WORKSTREAM.json
git diff --check
```
