# ImUi Kit Owner Split v1 - M4 Selection Combo Facade Owner Split

Status: selection/combo facade owner split landed
Date: 2026-05-13

## Result

- Moved the selectable, multi-selectable, and combo inherent facade wrappers into
  `ecosystem/fret-ui-kit/src/imui/facade_writer/selection_combo.rs`.
- These methods remain inherent methods on `ImUiFacade`; only their private source owner changed.
- No public method names changed.
- No `fret::imui` re-export path changed.
- No `fret-imui` dependency or public surface changed.
- No `crates/fret-ui` runtime contract changed.
- `ecosystem/fret-ui-kit/src/imui/facade_writer.rs` | 1582 lines | 1506 lines
- `ecosystem/fret-ui-kit/src/imui/facade_writer/selection_combo.rs` | n/a | 80 lines

## Evidence

- `ecosystem/fret-ui-kit/src/imui/facade_writer.rs`
- `ecosystem/fret-ui-kit/src/imui/facade_writer/selection_combo.rs`
- `ecosystem/fret-ui-kit/src/imui/selectable_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/combo_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/multi_select.rs`

## Gates

- `cargo fmt --package fret-ui-kit -- --check`
- `cargo check -p fret-ui-kit --features imui`
- `cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke --test imui_response_contract_smoke --no-fail-fast`
- `python -m json.tool docs/workstreams/imui-kit-owner-split-v1/WORKSTREAM.json`
- `python tools/check_workstream_catalog.py`
- `python tools/gate_imui_facade_teaching_source.py`
- `python tools/gate_imui_workstream_source.py`
- `python tools/check_layering.py`
- `git diff --check`
