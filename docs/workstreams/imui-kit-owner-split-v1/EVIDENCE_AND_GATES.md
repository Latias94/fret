# ImUi Kit Owner Split v1 - Evidence & Gates

Goal: make `fret-ui-kit::imui` private owner splits reviewable while preserving public IMUI API and
behavior.

Status: closed
Last updated: 2026-05-13

## Evidence Anchors

- `docs/workstreams/imui-kit-owner-split-v1/WORKSTREAM.json`
- `docs/workstreams/imui-kit-owner-split-v1/DESIGN.md`
- `docs/workstreams/imui-kit-owner-split-v1/M0_BASELINE_AUDIT_2026-05-13.md`
- `docs/workstreams/imui-kit-owner-split-v1/M1_BUTTON_ACTIONS_SLICE_2026-05-13.md`
- `docs/workstreams/imui-kit-owner-split-v1/M2_PRESSABLE_RESPONSE_ASSEMBLY_SLICE_2026-05-13.md`
- `docs/workstreams/imui-kit-owner-split-v1/M3_MENU_ITEMS_FACADE_OWNER_SPLIT_2026-05-13.md`
- `docs/workstreams/imui-kit-owner-split-v1/M4_SELECTION_COMBO_FACADE_OWNER_SPLIT_2026-05-13.md`
- `docs/workstreams/imui-kit-owner-split-v1/CLOSEOUT_AUDIT_2026-05-13.md`
- `docs/workstreams/imui-kit-owner-split-v1/TODO.md`
- `docs/workstreams/imui-kit-owner-split-v1/MILESTONES.md`
- `docs/workstreams/imui-kit-owner-split-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-imgui-gap-closure-v1/P3_EXECUTION_PRIORITY_REVIEW_2026-05-06.md`
- `docs/workstreams/imui-imgui-gap-closure-v1/P3_PUBLIC_SURFACE_CATALOG_2026-05-06.md`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/facade_writer.rs`
- `ecosystem/fret-ui-kit/src/imui/facade_writer/button_actions.rs`
- `ecosystem/fret-ui-kit/src/imui/facade_writer/menu_items.rs`
- `ecosystem/fret-ui-kit/src/imui/facade_writer/selection_combo.rs`
- `ecosystem/fret-ui-kit/src/imui/facade_support.rs`
- `ecosystem/fret-ui-kit/src/imui/interaction_runtime.rs`
- `ecosystem/fret-ui-kit/src/imui/interaction_runtime/pressable_response.rs`
- `ecosystem/fret-ui-kit/src/imui/response/hover.rs`

## First-Open Repro

Use the smallest source and behavior preservation proof:

```bash
cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke --test imui_response_contract_smoke --no-fail-fast
```

This does not prove all IMUI behavior. It is the first-open smoke gate for preserving the adapter
seam and response contract while moving private owners.

## Current Gates

### JSON Shape

```bash
python -m json.tool docs/workstreams/imui-kit-owner-split-v1/WORKSTREAM.json
```

### Workstream Catalog

```bash
python tools/check_workstream_catalog.py
```

### IMUI Source Policy

```bash
python tools/gate_imui_facade_teaching_source.py
python tools/gate_imui_workstream_source.py
```

### Focused Kit Smoke

```bash
cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke --test imui_response_contract_smoke --no-fail-fast
```

### Compile Floor

```bash
cargo check -p fret-ui-kit --features imui
```

### Format Floor

```bash
cargo fmt --package fret-ui-kit -- --check
```

### Diff Check

```bash
git diff --check
```

## Non-Gates

Do not treat this lane's smoke gates as proof for:

- new widgets,
- table advanced behavior,
- child-region resize behavior,
- debug draw feature growth,
- docking or OS-window multi-viewport behavior,
- runtime public API changes.

Those require separate follow-ons with their own repro and gates.

## Gate Results

2026-05-13 local results:

- `python -m json.tool docs/workstreams/imui-kit-owner-split-v1/WORKSTREAM.json` passed.
- `python tools/check_workstream_catalog.py` passed and validated 363 dedicated directories plus
  47 standalone markdown files.
- `python tools/gate_imui_facade_teaching_source.py` passed after refreshing the hello-demo marker
  to the current `test_id`-bearing source shape.
- `python tools/gate_imui_workstream_source.py` passed with this lane added to the source-policy
  checks.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke --test imui_response_contract_smoke --no-fail-fast`
  passed: 4 tests run, 4 passed.
- `cargo check -p fret-ui-kit --features imui` passed.
- `cargo fmt --package fret-ui-kit -- --check` passed.
- `git diff --check` passed.

M1 slice results:

- `cargo check -p fret-ui-kit --features imui` passed after moving the button/action wrappers.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke --test imui_response_contract_smoke --no-fail-fast`
  passed after moving the button/action wrappers: 4 tests run, 4 passed.

M2 slice results:

- `interaction_runtime::populate_pressable_response(...)` now centralizes the shared pressable
  response core used by `active_trigger_behavior.rs`, `item_behavior.rs`, and
  `slider_controls.rs`.
- `python -m json.tool docs/workstreams/imui-kit-owner-split-v1/WORKSTREAM.json` passed after
  adding the M2 status note.
- `python tools/check_workstream_catalog.py` passed after adding the M2 status note.
- `python tools/gate_imui_facade_teaching_source.py` passed after adding the M2 status note.
- `python tools/gate_imui_workstream_source.py` passed after adding the M2 source-policy markers.
- `cargo fmt --package fret-ui-kit -- --check` passed after the helper split.
- `cargo check -p fret-ui-kit --features imui` passed after the helper split.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke --test imui_response_contract_smoke --no-fail-fast`
  passed after the helper split: 4 tests run, 4 passed.
- `python tools/check_layering.py` passed after the helper split.
- `git diff --check` passed after the helper split and M2 status note.

M3 slice results:

- Menu item, menu action, begin-menu, begin-submenu, and command menu inherent facade wrappers now
  live in `facade_writer/menu_items.rs`.
- `facade_writer.rs` dropped from 1687 to 1582 lines; `menu_items.rs` is 109 lines.
- `python -m json.tool docs/workstreams/imui-kit-owner-split-v1/WORKSTREAM.json` passed after
  adding the M3 status note.
- `python tools/check_workstream_catalog.py` passed after adding the M3 status note.
- `python tools/gate_imui_facade_teaching_source.py` passed after adding the M3 status note.
- `python tools/gate_imui_workstream_source.py` passed after adding the M3 source-policy markers.
- `cargo fmt --package fret-ui-kit -- --check` passed after the menu owner split.
- `cargo check -p fret-ui-kit --features imui` passed after the menu owner split.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke --test imui_response_contract_smoke --no-fail-fast`
  passed after the menu owner split: 4 tests run, 4 passed.
- `python tools/check_layering.py` passed after the menu owner split.
- `git diff --check` passed after the menu owner split and M3 status note.

M4 slice results:

- Selectable, multi-selectable, and combo inherent facade wrappers now live in
  `facade_writer/selection_combo.rs`.
- `facade_writer.rs` dropped from 1582 to 1506 lines; `selection_combo.rs` is 80 lines.
- `python -m json.tool docs/workstreams/imui-kit-owner-split-v1/WORKSTREAM.json` passed after
  adding the M4 status note.
- `python tools/check_workstream_catalog.py` passed after adding the M4 status note.
- `python tools/gate_imui_facade_teaching_source.py` passed after adding the M4 status note.
- `python tools/gate_imui_workstream_source.py` passed after adding the M4 source-policy markers.
- `cargo fmt --package fret-ui-kit -- --check` passed after the selection/combo owner split.
- `cargo check -p fret-ui-kit --features imui` passed after the selection/combo owner split.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke --test imui_response_contract_smoke --no-fail-fast`
  passed after the selection/combo owner split: 4 tests run, 4 passed.
- `python tools/check_layering.py` passed after the selection/combo owner split.
- `git diff --check` passed after the selection/combo owner split and M4 status note.

M5 closeout results:

- `WORKSTREAM.json` is marked `closed` with `default_action: stay_closed`.
- `CLOSEOUT_AUDIT_2026-05-13.md` names `imui-facade-disclosure-owner-split-v1` as the next
  narrower follow-on for disclosure wrappers.
- This lane remains evidence for private owner splits only; additive widgets, docking,
  multi-window, and runtime contract changes stay out of scope.
- `python -m json.tool docs/workstreams/imui-kit-owner-split-v1/WORKSTREAM.json` passed after
  closeout.
- `python tools/check_workstream_catalog.py` passed after closeout.
- `python tools/gate_imui_facade_teaching_source.py` passed after closeout.
- `python tools/gate_imui_workstream_source.py` passed after closeout.
- `git diff --check` passed after closeout.
