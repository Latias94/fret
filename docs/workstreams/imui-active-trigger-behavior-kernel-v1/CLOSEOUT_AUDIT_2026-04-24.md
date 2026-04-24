# ImUi Active Trigger Behavior Kernel v1 Closeout Audit (2026-04-24)

Status: closed
Last updated: 2026-04-24

## Shipped Verdict

This lane shipped the private active-only trigger behavior kernel in `fret-ui-kit::imui` and proved
it across switch, menu item, menu trigger, submenu trigger, and tab trigger controls.

The lane is closed because the remaining similar-looking IMUI controls are different behavior
families:

- sliders need a value-editing kernel if duplication pressure grows;
- text controls need text focus/edit lifecycle work;
- disclosure controls need a context/double-click trigger cleanup, not active-only trigger cleanup.

## Evidence

Primary implementation anchors:

- `ecosystem/fret-ui-kit/src/imui/active_trigger_behavior.rs`
- `ecosystem/fret-ui-kit/src/imui/boolean_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/menu_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/menu_family_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/tab_family_controls.rs`

Workstream anchors:

- `docs/workstreams/imui-active-trigger-behavior-kernel-v1/DESIGN.md`
- `docs/workstreams/imui-active-trigger-behavior-kernel-v1/M0_M1_ACTIVE_TRIGGER_SLICE_2026-04-24.md`
- `docs/workstreams/imui-active-trigger-behavior-kernel-v1/TODO.md`
- `docs/workstreams/imui-active-trigger-behavior-kernel-v1/MILESTONES.md`
- `docs/workstreams/imui-active-trigger-behavior-kernel-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-active-trigger-behavior-kernel-v1/WORKSTREAM.json`

## Gates

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

## Follow-On Policy

Start a narrower follow-on instead of reopening this lane when the next change is about:

- slider pointer capture, value mutation, or edit commit semantics;
- text input focus/edit lifecycle;
- disclosure context-menu or double-click response cleanup;
- public `fret-imui` API widening;
- runtime mechanism changes.
