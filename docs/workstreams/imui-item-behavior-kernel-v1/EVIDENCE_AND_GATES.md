# ImUi Item Behavior Kernel v1 Evidence And Gates

Status: closed gate list
Last updated: 2026-04-24

## Smallest Repro

```bash
cargo check -p fret-ui-kit --features imui
python3 tools/audit_crate.py --crate fret-ui-kit
cargo nextest run -p fret-ui-kit --features imui --test imui_selectable_smoke --test imui_combo_smoke --test imui_button_smoke --test imui_response_contract_smoke --no-fail-fast
cargo nextest run -p fret-imui --no-fail-fast
```

## Required Gates

```bash
cargo fmt --package fret-ui-kit
cargo check -p fret-ui-kit --features imui
python3 tools/audit_crate.py --crate fret-ui-kit
cargo nextest run -p fret-imui --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui --test imui_selectable_smoke --test imui_combo_smoke --test imui_button_smoke --test imui_response_contract_smoke --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui --no-fail-fast
cargo build -p fret-demo --bin imui_interaction_showcase_demo
cargo build -p fret-demo --bin imui_editor_proof_demo
python3 tools/check_layering.py
git diff --check
python3 tools/check_workstream_catalog.py
python3 -m json.tool docs/workstreams/imui-item-behavior-kernel-v1/WORKSTREAM.json
```

## Evidence Anchors

Lane docs:

- `docs/workstreams/imui-item-behavior-kernel-v1/WORKSTREAM.json`
- `docs/workstreams/imui-item-behavior-kernel-v1/DESIGN.md`
- `docs/workstreams/imui-item-behavior-kernel-v1/TODO.md`
- `docs/workstreams/imui-item-behavior-kernel-v1/MILESTONES.md`
- `docs/workstreams/imui-item-behavior-kernel-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-item-behavior-kernel-v1/M0_M2_KERNEL_SLICE_2026-04-24.md`
- `docs/workstreams/imui-item-behavior-kernel-v1/M3_SELECTABLE_COMBO_CLOSEOUT_2026-04-24.md`
- `docs/workstreams/imui-item-behavior-kernel-v1/CLOSEOUT_AUDIT_2026-04-24.md`

Prior lane evidence:

- `docs/workstreams/imui-facade-internal-modularization-v1/CLOSEOUT_AUDIT_2026-04-21.md`
- `docs/workstreams/imui-control-chrome-fearless-refactor-v1/FINAL_STATUS.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/WORKSTREAM.json`
- `docs/audits/imui-imgui-gap-audit-2026-04-22.md`

Implementation anchors:

- `ecosystem/fret-ui-kit/src/imui/item_behavior.rs`
- `ecosystem/fret-ui-kit/src/imui/button_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/boolean_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/selectable_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/combo_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/menu_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/menu_family_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/tab_family_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/slider_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/control_chrome.rs`
- `ecosystem/fret-ui-kit/src/imui/interaction_runtime`
- `ecosystem/fret-ui-kit/src/imui/response`
- `ecosystem/fret-imui/src/tests`

Reference anchors:

- `repo-ref/imgui/imgui_widgets.cpp`
- `repo-ref/imgui/imgui.cpp`

## Verified Slice Gates

Passed on 2026-04-24 for the first private-kernel slice:

```bash
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-ui-kit --features imui --test imui_button_smoke --test imui_adapter_seam_smoke --test imui_response_contract_smoke --no-fail-fast
cargo nextest run -p fret-imui interaction_press:: --no-fail-fast
cargo build -p fret-demo --bin imui_interaction_showcase_demo
cargo build -p fret-demo --bin imui_editor_proof_demo
cargo nextest run -p fret-imui --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui --no-fail-fast
```

Passed on 2026-04-24 for the selectable and combo trigger closeout:

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

## Fearless Refactor Gate

Each implementation slice must answer this before review:

- Which duplicate or obsolete behavior path was deleted?
- Which observable behavior proves the replacement is correct?
- Which existing public contract stayed unchanged, or which ADR/alignment doc changed because the
  old contract was wrong?
- Why is the remaining behavior kernel private rather than public API?
