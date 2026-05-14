# ImUi Facade Floating Popup Owner Split v1 - Evidence & Gates

Goal: move floating/popup facade default implementation bodies behind a private owner while
preserving public IMUI API and behavior.

Status: closed
Last updated: 2026-05-14

## Evidence Anchors

- `docs/workstreams/imui-facade-floating-popup-owner-split-v1/WORKSTREAM.json`
- `docs/workstreams/imui-facade-floating-popup-owner-split-v1/DESIGN.md`
- `docs/workstreams/imui-facade-floating-popup-owner-split-v1/M0_BASELINE_AUDIT_2026-05-14.md`
- `docs/workstreams/imui-facade-floating-popup-owner-split-v1/M1_FLOATING_POPUP_FACADE_OWNER_SPLIT_2026-05-14.md`
- `docs/workstreams/imui-facade-floating-popup-owner-split-v1/CLOSEOUT_AUDIT_2026-05-14.md`
- `ecosystem/fret-ui-kit/src/imui/facade_writer.rs`
- `ecosystem/fret-ui-kit/src/imui/facade_writer/floating_popup.rs`
- `ecosystem/fret-ui-kit/src/imui/floating_surface.rs`
- `ecosystem/fret-ui-kit/src/imui/floating_window.rs`
- `ecosystem/fret-ui-kit/src/imui/popup_overlay.rs`
- `ecosystem/fret-ui-kit/src/imui/tooltip_overlay.rs`
- `ecosystem/fret-ui-kit/src/imui/drag_drop.rs`

## Gates

- `python -m json.tool docs/workstreams/imui-facade-floating-popup-owner-split-v1/WORKSTREAM.json`
- `python tools/check_workstream_catalog.py`
- `python tools/gate_imui_facade_teaching_source.py`
- `python tools/gate_imui_workstream_source.py`
- `cargo fmt --package fret-ui-kit -- --check`
- `cargo check -p fret-ui-kit --features imui`
- `cargo nextest run -p fret-ui-kit --features imui --test imui_tooltip_smoke --test imui_drag_drop_smoke --test imui_adapter_seam_smoke --test imui_response_contract_smoke --no-fail-fast`
- `cargo nextest run -p fret-imui floating popup_hover --no-fail-fast`
- `python tools/check_layering.py`
- `git diff --check`
