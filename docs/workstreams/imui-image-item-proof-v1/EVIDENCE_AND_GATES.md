# ImUi Image Item Proof Evidence And Gates

Status: closed
Last updated: 2026-05-16

Status note (2026-05-16): canonical gates below passed for the closeout slice.

## Canonical Gates

```text
cargo fmt --package fret-ui-kit
cargo nextest run -p fret-ui-kit --features imui --test imui_image_item_smoke --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui image_item --no-fail-fast
python tools/gate_imui_workstream_source.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-image-item-proof-v1/WORKSTREAM.json
git diff --check
```

## Evidence Anchors

- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/facade_writer.rs`
- `ecosystem/fret-ui-kit/src/imui/image_item_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/options/controls.rs`
- `ecosystem/fret-ui-kit/tests/imui_image_item_smoke.rs`
- `crates/fret-ui/src/element.rs`
- `repo-ref/imgui/imgui.h`
- `docs/workstreams/imui-imgui-gap-closure-v1/P3_COMPONENT_SURFACE_CATALOG_2026-05-06.md`

## Current Results

2026-05-16 implementation slice:

- `cargo fmt --package fret-ui-kit` passed.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_image_item_smoke --no-fail-fast`
  passed: 2 tests.
- `cargo nextest run -p fret-ui-kit --features imui image_item --no-fail-fast` passed: 4 tests,
  636 skipped.
- `python tools/gate_imui_workstream_source.py` passed.
- `python tools/check_workstream_catalog.py` passed.
- `python -m json.tool docs/workstreams/imui-image-item-proof-v1/WORKSTREAM.json` passed.
