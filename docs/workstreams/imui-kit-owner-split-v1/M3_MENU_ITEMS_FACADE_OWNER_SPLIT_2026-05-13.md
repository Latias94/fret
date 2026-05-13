# ImUi Kit Owner Split v1 - M3 Menu Items Facade Owner Split

Status: menu facade owner split landed
Date: 2026-05-13

## Result

- Moved the menu item, menu action, begin-menu, begin-submenu, and command menu inherent facade
  wrappers into `ecosystem/fret-ui-kit/src/imui/facade_writer/menu_items.rs`.
- These methods remain inherent methods on `ImUiFacade`; only their private source owner changed.
- No public method names changed.
- No `fret::imui` re-export path changed.
- No `fret-imui` dependency or public surface changed.
- No `crates/fret-ui` runtime contract changed.
- `ecosystem/fret-ui-kit/src/imui/facade_writer.rs` | 1687 lines | 1582 lines
- `ecosystem/fret-ui-kit/src/imui/facade_writer/menu_items.rs` | n/a | 109 lines

## Evidence

- `ecosystem/fret-ui-kit/src/imui/facade_writer.rs`
- `ecosystem/fret-ui-kit/src/imui/facade_writer/menu_items.rs`
- `ecosystem/fret-ui-kit/src/imui/menu_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/menu_family_controls.rs`

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
