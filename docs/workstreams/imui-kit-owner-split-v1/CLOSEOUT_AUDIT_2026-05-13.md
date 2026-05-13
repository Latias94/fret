# ImUi Kit Owner Split v1 - Closeout Audit - 2026-05-13

Status: closed
Last updated: 2026-05-13

## Objective

Close `imui-kit-owner-split-v1` by:

1. recording the private owner splits landed in `fret-ui-kit::imui`,
2. proving that public IMUI names, `fret::imui` re-export paths, `fret-imui`, and
   `crates/fret-ui` runtime contracts stayed unchanged,
3. naming the next narrower follow-on instead of expanding this lane indefinitely, and
4. leaving the gate/evidence set needed to resume safely.

## Completion Checklist

| Requirement | Evidence |
| --- | --- |
| Keep `fret-imui` thin and unchanged | `docs/workstreams/imui-kit-owner-split-v1/M1_BUTTON_ACTIONS_SLICE_2026-05-13.md`, `M3_MENU_ITEMS_FACADE_OWNER_SPLIT_2026-05-13.md`, `M4_SELECTION_COMBO_FACADE_OWNER_SPLIT_2026-05-13.md` |
| Preserve public `ImUiFacade` names and `fret::imui` paths | same M1/M3/M4 notes plus `cargo check -p fret-ui-kit --features imui` |
| Avoid `crates/fret-ui` runtime contract widening | `docs/workstreams/imui-kit-owner-split-v1/DESIGN.md`, M1-M4 notes |
| Split private facade owners | `ecosystem/fret-ui-kit/src/imui/facade_writer/button_actions.rs`, `facade_writer/menu_items.rs`, `facade_writer/selection_combo.rs` |
| Reduce response/status assembly duplication privately | `ecosystem/fret-ui-kit/src/imui/interaction_runtime/pressable_response.rs`, `M2_PRESSABLE_RESPONSE_ASSEMBLY_SLICE_2026-05-13.md` |
| Keep repro/gate/evidence current | `EVIDENCE_AND_GATES.md`, `python tools/gate_imui_workstream_source.py`, `python tools/gate_imui_facade_teaching_source.py`, `cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke --test imui_response_contract_smoke --no-fail-fast` |

## Outcome

`facade_writer.rs` dropped from 1801 lines at M0 to 1506 lines at closeout. The landed private
owners are:

- `facade_writer/button_actions.rs`
- `facade_writer/menu_items.rs`
- `facade_writer/selection_combo.rs`
- `interaction_runtime/pressable_response.rs`

This lane is closed as a successful private-owner reduction. It intentionally did not add Dear
ImGui widgets, widen helper APIs, move policy into `fret-imui`, or touch docking/multi-window
runtime behavior.

## Residual Scope

Start the next narrow follow-on as `imui-facade-disclosure-owner-split-v1` for disclosure-adjacent
facade wrappers such as `collapsing_header` and `tree_node`. Text, boolean, slider/model, table,
debug draw feature growth, docking, multi-window, and additive Dear ImGui component parity remain
separate lanes.

Implementation note: `WORKSTREAM.json` is now marked `closed` and `stay_closed`. Resume only by
opening a narrower follow-on with its own repro, gate, and evidence anchors.
