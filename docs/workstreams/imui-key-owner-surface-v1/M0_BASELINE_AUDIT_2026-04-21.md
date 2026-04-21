# M0 Baseline Audit — 2026-04-21

Purpose: justify why `imui-key-owner-surface-v1` is a new narrow follow-on instead of a reopened
umbrella backlog or a widened lifecycle / collection / menu lane.

## Evidence reviewed

- `docs/workstreams/imui-editor-grade-product-closure-v1/P0_IMMEDIATE_PARITY_STATUS_2026-04-13.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- `docs/workstreams/imui-response-status-lifecycle-v1/FINAL_STATUS.md`
- `docs/workstreams/imui-collection-pane-proof-v1/CLOSEOUT_AUDIT_2026-04-21.md`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-imui/src/tests/interaction.rs`
- `ecosystem/fret-imui/src/tests/models.rs`
- `ecosystem/fret-imui/src/tests/popup_hover.rs`
- `apps/fret-examples/src/imui_response_signals_demo.rs`

## Findings

### 1) The remaining shortcut/key-owner gap is now narrow enough to justify its own lane

The current parity read no longer points to a broad missing immediate interaction stack.
The focused helper-local shortcut work already landed:

- command metadata hinting exists,
- `activate_shortcut` exists on the current helper families,
- repeated activation is explicitly opt-in,
- and interaction tests already prove focus-scoped local behavior across the major immediate
  pressable families.

What remains open is the deeper ownership question:

- whether the current helper-local shortcut shape is enough,
- whether any immediate equivalent to `SetNextItemShortcut()` / `SetItemKeyOwner()` is warranted,
- and what first-party proof surface should justify that decision.

Conclusion:

- this is now a narrow follow-on, not umbrella backlog material.

### 2) The owner split is already clear enough to avoid runtime drift

The repo evidence already supports a stable owner split:

- `crates/fret-ui` stays unchanged,
- `crates/fret-app` / `crates/fret-runtime` keep the global command/keymap/arbitration contract,
- `fret-ui-kit::imui` owns any additive immediate facade surface,
- `fret-imui` owns behavior proof,
- and `apps/fret-examples` owns proof/contract demos plus source-policy gates.

Conclusion:

- the lane should start by freezing that owner split instead of arguing for runtime growth.

### 3) The current proof is real, but it is still mostly test-first

The current executable floor is stronger than the first-party demo story:

- `ecosystem/fret-imui/src/tests/interaction.rs`,
- `ecosystem/fret-imui/src/tests/models.rs`,
- and `ecosystem/fret-imui/src/tests/popup_hover.rs`

already prove focus-scoped local activation across direct pressables, menu/menu-submenu triggers,
popup items, tabs, and combo families.

By contrast, `apps/fret-examples/src/imui_response_signals_demo.rs` is still the closest
first-party proof/contract surface, but it is not yet a dedicated key-owner product proof.

Conclusion:

- the lane should freeze a test-first baseline now and let M1 decide whether a stronger demo proof
  is required.

### 4) Adjacent pressures must stay out of this folder

The current closeout/status chain already split the neighboring questions:

- `imui-response-status-lifecycle-v1` owns lifecycle vocabulary,
- `imui-collection-pane-proof-v1` closed collection/pane proof breadth,
- menu/submenu/tab policy depth stays on the trigger-response / policy line,
- and runner/backend multi-window parity stays in the docking lane.

Conclusion:

- this lane should own only key-owner / item-local shortcut ownership depth.

## Execution consequence

Use `imui-key-owner-surface-v1` as the active narrow P0 follow-on for immediate key-owner /
item-local shortcut ownership.

From this note forward:

1. treat the current focused shortcut test floor as real shipped progress, not as missing baseline,
2. keep `crates/fret-ui` and runtime keymap arbitration fixed unless stronger evidence appears,
3. start from `imui_response_signals_demo` plus targeted `fret-imui` tests as the current repro
   set,
4. and split again instead of widening this lane if the work turns into lifecycle, collection/pane,
   menu/tab policy, or runtime shortcut arbitration.
