# ImUi Active Trigger Behavior Kernel M0-M1 Slice (2026-04-24)

Status: landed private active-trigger kernel slice
Last updated: 2026-04-24

## Baseline

The previous full item behavior lane intentionally left active-only trigger controls out because
they did not need long press, drag threshold tracking, right-click context-menu reporting, or
double-click reporting.

The duplicate active-only shape was present in:

- `switch_model_with_options`;
- menu items;
- menu triggers;
- submenu triggers;
- tab triggers.

## Landed Slice

Added the private module:

- `ecosystem/fret-ui-kit/src/imui/active_trigger_behavior.rs`

Migrated active-only trigger families:

- switch controls in `boolean_controls.rs`;
- menu items in `menu_controls.rs`;
- menu and submenu triggers in `menu_family_controls.rs`;
- tab triggers in `tab_family_controls.rs`.

The shared kernel owns:

- stale pointer/key hook reset for active-only trigger elements;
- left-pointer active item lifecycle;
- optional focus request on pointer press;
- hover delay and focus-visible response population;
- rect, clicked, changed, lifecycle, and disabled response population.

The migrated families still own:

- activation side effects;
- shortcut semantics;
- command dispatch;
- popup open/close;
- menu/submenu/menubar policy;
- tab selection.

## Boundary Verdict

The following remain out of scope:

- sliders, because they own value mutation, pointer capture, and edit commit semantics;
- text controls, because they own focus/edit/blur lifecycle rather than trigger activation;
- disclosure controls, because their remaining duplication is context/double-click trigger behavior,
  not active-only trigger behavior.

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
