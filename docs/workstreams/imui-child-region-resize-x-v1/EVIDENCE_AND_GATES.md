# ImUi Child Region ResizeX Evidence And Gates

Status: closed
Last updated: 2026-05-16

Status note (2026-05-16): canonical gates below passed for the closeout slice.

## Canonical Gates

```text
cargo fmt --package fret-ui-kit --check
cargo nextest run -p fret-ui-kit --features imui --test imui_child_region_smoke --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui child_region_resize --no-fail-fast
python tools/gate_imui_workstream_source.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-child-region-resize-x-v1/WORKSTREAM.json
git diff --check
```

## Evidence Anchors

- `ecosystem/fret-ui-kit/src/imui/options/containers.rs`
- `ecosystem/fret-ui-kit/src/imui/child_region.rs`
- `ecosystem/fret-ui-kit/src/imui/response/widgets.rs`
- `ecosystem/fret-ui-kit/tests/imui_child_region_smoke.rs`
- `tools/gate_imui_workstream_source.py`
- `docs/workstreams/imui-child-region-resize-y-v1/CLOSEOUT_AUDIT_2026-05-15.md`
- `docs/workstreams/imui-imgui-gap-closure-v1/P3_CHILD_REGION_READINESS_2026-05-06.md`

## Current Results

2026-05-16 implementation slice:

- `cargo fmt --package fret-ui-kit` passed.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_child_region_smoke --no-fail-fast`
  passed: 3 tests.
- `cargo nextest run -p fret-ui-kit --features imui child_region_resize --no-fail-fast` passed: 4
  tests, 638 skipped.
- `cargo fmt --package fret-ui-kit --check` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python tools/check_workstream_catalog.py` passed: 379 dedicated directories, 47 standalone
  markdown files.
- `python -m json.tool docs/workstreams/imui-child-region-resize-x-v1/WORKSTREAM.json` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.
