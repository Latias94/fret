# ImUi Item Behavior Kernel M3 Selectable/Combo Closeout (2026-04-24)

Status: landed selectable and combo trigger proof; closeout-ready boundary verdict
Last updated: 2026-04-24

## Landed Slice

The private `fret-ui-kit::imui` pressable item behavior kernel now owns the duplicated full item
behavior for:

- button controls;
- checkbox and radio controls;
- selectable rows;
- combo triggers.

The selectable migration uses `PressableItemBehaviorOptions { report_pointer_click: true }` because
selectable rows are the only migrated family that reports pointer-click modifiers as an observable
response signal.

The combo migration keeps popup toggling local. The shared kernel only owns the trigger's common
pressable item behavior: pointer lifecycle, active item tracking, drag threshold tracking,
right-click context request, double-click, hover delay response fields, lifecycle response fields,
and enabled-state sanitizing.

## Deleted Duplicate Paths

The migrated families no longer each carry local copies of:

- `pressable_clear_on_pointer_*` / key hook reset setup;
- active item and lifecycle model wiring;
- long-press and drag threshold pointer handlers;
- right-click context-menu transient reporting;
- double-click transient reporting;
- hover delay response population;
- shared pressable drag, rect, lifecycle, and disabled response population.

This is intentionally a replacement, not a compatibility layer. The obsolete family-local behavior
blocks were deleted from the migrated controls.

## Boundary Verdict

The remaining nearby controls should not be forced into the current full pressable item kernel:

- `switch_model_with_options` is an active-only control path. It does not need the long-press,
  context-menu, double-click, or drag threshold behavior owned by this kernel.
- menu item, menu trigger, submenu trigger, and tab trigger paths are active-only plus menu/tab
  policy. They also carry roving focus, menubar switching, submenu grace, or popup close/open
  policy that must stay out of a generic item kernel.
- sliders own value mutation, pointer capture/drag commit, and edit lifecycle policy. They need a
  separate value-editing kernel if the duplication pressure becomes real.

The next narrow follow-on, if needed, should be an active-trigger response/kernel lane for
switch/menu/tab trigger cleanup. It should not reopen this full pressable item kernel lane unless a
new consumer truly needs long-press/context/double-click/drag behavior.

## Verified Gates

Passed on 2026-04-24:

```bash
cargo fmt --package fret-ui-kit
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui --test imui_selectable_smoke --test imui_combo_smoke --test imui_button_smoke --test imui_response_contract_smoke --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui --no-fail-fast
cargo nextest run -p fret-imui --no-fail-fast
cargo build -p fret-demo --bin imui_interaction_showcase_demo
cargo build -p fret-demo --bin imui_editor_proof_demo
python3 tools/audit_crate.py --crate fret-ui-kit
python3 tools/check_layering.py
python3 tools/check_workstream_catalog.py
python3 -m json.tool docs/workstreams/imui-item-behavior-kernel-v1/WORKSTREAM.json
git diff --check
```

## Contract Verdict

No public API was widened.

No changes were made to:

- `ecosystem/fret-imui`;
- `crates/fret-ui`;
- runtime mechanism contracts;
- ADR-level behavior contracts.
