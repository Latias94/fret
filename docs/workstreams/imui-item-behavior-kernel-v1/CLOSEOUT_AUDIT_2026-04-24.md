# ImUi Item Behavior Kernel v1 Closeout Audit (2026-04-24)

Status: closed
Last updated: 2026-04-24

## Shipped Verdict

This lane shipped the private full pressable item behavior kernel in `fret-ui-kit::imui` and proved
it across four first-party control families:

- buttons;
- checkbox/radio;
- selectable rows;
- combo triggers.

The lane is closed because the remaining IMUI interaction duplication is not the same behavior
shape. Switches, menu/tab triggers, and sliders should be handled by narrower follow-ons rather than
by widening this kernel into a vague Dear ImGui compatibility layer.

## Evidence

Primary implementation anchors:

- `ecosystem/fret-ui-kit/src/imui/item_behavior.rs`
- `ecosystem/fret-ui-kit/src/imui/button_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/boolean_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/selectable_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/combo_controls.rs`

Workstream state anchors:

- `docs/workstreams/imui-item-behavior-kernel-v1/DESIGN.md`
- `docs/workstreams/imui-item-behavior-kernel-v1/M0_M2_KERNEL_SLICE_2026-04-24.md`
- `docs/workstreams/imui-item-behavior-kernel-v1/M3_SELECTABLE_COMBO_CLOSEOUT_2026-04-24.md`
- `docs/workstreams/imui-item-behavior-kernel-v1/TODO.md`
- `docs/workstreams/imui-item-behavior-kernel-v1/MILESTONES.md`
- `docs/workstreams/imui-item-behavior-kernel-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-item-behavior-kernel-v1/WORKSTREAM.json`

## Gates

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

## Follow-On Policy

Start a narrower follow-on instead of reopening this lane when the next change is about:

- active-only trigger response cleanup for switch/menu/tab controls;
- menu, submenu, menubar, or tab navigation policy;
- slider pointer capture, value mutation, or edit commit semantics;
- public API widening in `fret-imui`;
- runtime mechanism changes in `fret-ui`.
