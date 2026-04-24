# ImUi Item Behavior Kernel M0-M2 Slice (2026-04-24)

Status: landed first private-kernel slice
Last updated: 2026-04-24

Status note (2026-04-24): this remains the first-slice record. The later selectable/combo trigger
migration and closeout verdict live in
`docs/workstreams/imui-item-behavior-kernel-v1/M3_SELECTABLE_COMBO_CLOSEOUT_2026-04-24.md` and
`docs/workstreams/imui-item-behavior-kernel-v1/CLOSEOUT_AUDIT_2026-04-24.md`.

## Baseline Audit

The repeated item-like behavior was concentrated in the pressable closures for:

- `button_controls.rs`
- `boolean_controls.rs`
- `selectable_controls.rs`
- `combo_controls.rs`
- `menu_controls.rs`
- `menu_family_controls.rs`
- `tab_family_controls.rs`
- `slider_controls.rs`

The full duplicate shape is not identical across all families. The safest first kernel owns the
behavior that is actually shared by button, checkbox, radio, combo trigger, and selectable rows:

- clear old pressable/key hooks for the element id;
- maintain active item and lifecycle models;
- arm/cancel long press and drag threshold tracking;
- report right-click context menu requests and double-click;
- populate hover delay, long-press, context-menu, drag, rect, lifecycle, and enabled-state response
  fields.

The kernel intentionally does not own:

- activation side effects;
- shortcut semantics;
- visual chrome;
- layout sizing;
- menu navigation/submenu policy;
- slider value mutation;
- switch's lightweight active-only behavior.

## Landed Slice

Added the private module:

- `ecosystem/fret-ui-kit/src/imui/item_behavior.rs`

Migrated first families:

- button family in `button_controls.rs`;
- checkbox and radio in `boolean_controls.rs`.

The migrated families now share:

- `install_pressable_item_behavior`;
- `populate_pressable_item_response`;
- `PressableItemResponseInput`.

Obsolete family-local duplicate pointer/drag/context/hover/lifecycle response code was deleted from
the migrated button, checkbox, and radio paths instead of preserved as fallback.

## Boundary Verdict

No public API was widened.

No changes were made to:

- `ecosystem/fret-imui`;
- `crates/fret-ui`;
- `fret-authoring::Response`;
- runtime mechanism contracts;
- ADRs.

The next candidate is `selectable_controls.rs` because it has the same long-press/context/double
click behavior plus one extra pointer-click modifier signal. `switch_model_with_options` should not
be forced into the current kernel until the kernel has an optional active-only mode or a second
consumer needing the same lighter shape.

## Verified Gates

Passed on 2026-04-24:

```bash
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui --test imui_button_smoke --test imui_adapter_seam_smoke --test imui_response_contract_smoke --no-fail-fast
cargo nextest run -p fret-imui interaction_press:: --no-fail-fast
cargo build -p fret-demo --bin imui_interaction_showcase_demo
cargo build -p fret-demo --bin imui_editor_proof_demo
cargo nextest run -p fret-imui --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui --no-fail-fast
```

Note: the first `imui_interaction_showcase_demo` build attempt hit the 240s command timeout during
cold compilation and left cargo/rustc running. After waiting for that process to finish, the same
build passed incrementally.

The first wide `fret-ui-kit --features imui` run exposed one stale source-policy assertion that was
still checking root `imui.rs` for `B: IntoUiElement<H>` after the earlier facade-support
modularization. The assertion now checks `imui/facade_support.rs`, matching the current owner.
