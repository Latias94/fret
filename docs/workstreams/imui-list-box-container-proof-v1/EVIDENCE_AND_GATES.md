# IMUI List Box Container Proof v1 - Evidence & Gates

Status: Closed
Last updated: 2026-05-25

## Evidence Anchors

- `docs/workstreams/imui-list-box-container-proof-v1/WORKSTREAM.json`
- `docs/workstreams/imui-list-box-container-proof-v1/DESIGN.md`
- `ecosystem/fret-ui-kit/src/imui/options/containers.rs`
- `ecosystem/fret-ui-kit/src/imui/list_box_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/facade_writer.rs`
- `ecosystem/fret-imui/src/tests/composition/layout_collections.rs`
- `tools/gate_imui_workstream_source.py`

## Focused Gates

```powershell
cargo fmt --check -p fret-ui-kit -p fret-imui
cargo check -p fret-ui-kit --features imui
cargo nextest run -p fret-imui list_box_container_stamps_semantics_scroll_and_hosts_selectables --no-fail-fast
python tools/gate_imui_workstream_source.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-list-box-container-proof-v1/WORKSTREAM.json
git diff --check
```

## 2026-05-25 Slice Results

- PASS: `cargo fmt -p fret-ui-kit -p fret-imui`
- PASS: `cargo fmt --check -p fret-ui-kit -p fret-imui`
- PASS: `cargo check -p fret-ui-kit --features imui`
  - Existing warnings only from `crates/fret-ui`: `unexpected cfg` for
    `unstable-retained-bridge` and unused `current_effective_opacity`.
- PASS: `cargo nextest run -p fret-imui list_box_container_stamps_semantics_scroll_and_hosts_selectables --no-fail-fast`
  - 1 passed, 179 skipped.
- PASS: `python tools/gate_imui_workstream_source.py`
- PASS: `python tools/check_workstream_catalog.py`
  - Validated 442 dedicated directories and 47 standalone markdown files.
- PASS: `python -m json.tool docs/workstreams/imui-list-box-container-proof-v1/WORKSTREAM.json`
- PASS_WITH_WARNINGS: `git diff --check`
  - No whitespace errors.
  - Existing line-ending warnings remain for `Cargo.lock` and `apps/fret-examples/src/lib.rs`.
