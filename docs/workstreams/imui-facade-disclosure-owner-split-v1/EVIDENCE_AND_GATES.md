# ImUi Facade Disclosure Owner Split v1 - Evidence & Gates

Goal: move disclosure facade wrappers behind a private owner while preserving public IMUI API and
behavior.

Status: closed
Last updated: 2026-05-13

## Evidence Anchors

- `docs/workstreams/imui-facade-disclosure-owner-split-v1/WORKSTREAM.json`
- `docs/workstreams/imui-facade-disclosure-owner-split-v1/DESIGN.md`
- `docs/workstreams/imui-facade-disclosure-owner-split-v1/M0_BASELINE_AUDIT_2026-05-13.md`
- `docs/workstreams/imui-facade-disclosure-owner-split-v1/M1_DISCLOSURE_FACADE_OWNER_SPLIT_2026-05-13.md`
- `docs/workstreams/imui-facade-disclosure-owner-split-v1/CLOSEOUT_AUDIT_2026-05-13.md`
- `ecosystem/fret-ui-kit/src/imui/facade_writer.rs`
- `ecosystem/fret-ui-kit/src/imui/facade_writer/disclosure.rs`
- `ecosystem/fret-ui-kit/src/imui/disclosure_controls.rs`

## Gates

- `python -m json.tool docs/workstreams/imui-facade-disclosure-owner-split-v1/WORKSTREAM.json`
- `python tools/check_workstream_catalog.py`
- `python tools/gate_imui_facade_teaching_source.py`
- `python tools/gate_imui_workstream_source.py`
- `cargo fmt --package fret-ui-kit -- --check`
- `cargo check -p fret-ui-kit --features imui`
- `cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke --test imui_response_contract_smoke --no-fail-fast`
- `python tools/check_layering.py`
- `git diff --check`
