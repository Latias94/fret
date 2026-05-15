# ImUi Child Region ResizeY Closeout

Status: Closed
Date: 2026-05-15

## Verdict

This narrow follow-on is closed. The shipped slice adds vertical child-region resize policy to
`fret-ui-kit::imui` without reopening `imui-child-region-depth-v1` and without copying Dear ImGui's
generic `BeginChild()` flag surface.

## Shipped Surface

- `ChildRegionOptions::resize_y` is optional and defaults to disabled.
- `ChildRegionResizeYOptions` carries min height, max height, and optional handle `test_id`.
- `child_region(...)` and `child_region_with_options(...)` return `ChildRegionResponse` and remain
  ignorable by existing callers.
- `ChildRegionResizeYResponse::height_from_start(...)` keeps height state app-owned and clamps to
  the configured min/max bounds.
- The rendered resize affordance is a bottom absolute pointer-region handle with row-resize cursor
  and existing IMUI pointer drag response plumbing.

## Evidence

- `ecosystem/fret-ui-kit/src/imui/options/containers.rs`
- `ecosystem/fret-ui-kit/src/imui/child_region.rs`
- `ecosystem/fret-ui-kit/src/imui/response/widgets.rs`
- `ecosystem/fret-ui-kit/tests/imui_child_region_smoke.rs`
- `ecosystem/fret-imui/src/tests/composition/layout_collections.rs`
- `tools/gate_imui_workstream_source.py`

## Gates

- `cargo fmt -p fret-ui-kit -p fret-imui --check`
- `cargo nextest run -p fret-ui-kit --features imui --test imui_child_region_smoke --no-fail-fast`
- `cargo nextest run -p fret-imui child_region --no-fail-fast`
- `python tools/gate_imui_workstream_source.py`
- `python tools/check_workstream_catalog.py`
- `python -m json.tool docs/workstreams/imui-child-region-resize-y-v1/WORKSTREAM.json`
- `git diff --check`

## Stay Closed Unless

Start a new proof-led follow-on instead of reopening this lane for:

- horizontal resize,
- auto-resize,
- focus-boundary flattening,
- `BeginChild() -> bool` visibility semantics,
- diagnostics drag simulation for this handle,
- or any broader Dear ImGui flag mirror.
