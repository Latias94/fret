# ImUi Selectable Highlight Policy Evidence And Gates

Status: closed
Last updated: 2026-05-16

Status note (2026-05-16): canonical gates below passed for the closeout slice.

## Canonical Gates

```text
cargo fmt --package fret-ui-kit --check
cargo nextest run -p fret-ui-kit --features imui --test imui_selectable_smoke --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui selectable_palette --no-fail-fast
python tools/gate_imui_workstream_source.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-selectable-highlight-policy-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json
git diff --check
```

## Evidence Anchors

- `ecosystem/fret-ui-kit/src/imui/options/controls.rs`
- `ecosystem/fret-ui-kit/src/imui/selectable_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/text_picker_controls.rs`
- `ecosystem/fret-ui-kit/tests/imui_selectable_smoke.rs`
- `docs/workstreams/imui-imgui-gap-closure-v1/P3_COMPONENT_SURFACE_CATALOG_2026-05-06.md`
- `repo-ref/imgui/imgui.h`
- `repo-ref/imgui/imgui_demo.cpp`

## Current Results

2026-05-16 implementation slice:

- `cargo fmt --package fret-ui-kit` passed.
- `cargo fmt --package fret-ui-kit --check` passed.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_selectable_smoke --no-fail-fast`
  passed: 1 test.
- `cargo nextest run -p fret-ui-kit --features imui selectable_palette --no-fail-fast` passed: 2
  tests, 641 skipped.
- `python tools/gate_imui_workstream_source.py` passed.
- `python tools/check_workstream_catalog.py` passed: 380 dedicated directories, 47 standalone
  markdown files.
- `python -m json.tool docs/workstreams/imui-selectable-highlight-policy-v1/WORKSTREAM.json`
  passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.
