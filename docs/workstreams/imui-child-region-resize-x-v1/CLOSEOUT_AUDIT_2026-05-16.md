# ImUi Child Region ResizeX Closeout

Status: Closed
Date: 2026-05-16

## Verdict

This narrow follow-on is closed. The shipped slice adds horizontal child-region resize policy to
`fret-ui-kit::imui` without reopening `imui-child-region-resize-y-v1` and without copying Dear
ImGui's generic `BeginChild()` flag surface.

## Shipped Surface

- `ChildRegionOptions::resize_x` is optional and defaults to disabled.
- `ChildRegionResizeXOptions` carries min width, max width, and optional handle `test_id`.
- `ChildRegionResponse::resize_x()` exposes horizontal resize state alongside the existing
  `resize_y()` surface.
- `ChildRegionResizeXResponse::width_from_start(...)` keeps width state app-owned and clamps to the
  configured min/max bounds.
- The rendered resize affordance is a right-edge absolute pointer-region handle with column-resize
  cursor and existing IMUI pointer drag response plumbing.
- Child regions can enable `resize_x` and `resize_y` together; the resize wrapper owns the root
  `test_id` while the scroll content keeps explicit content/viewport ids.

## Evidence

- `ecosystem/fret-ui-kit/src/imui/options/containers.rs`
- `ecosystem/fret-ui-kit/src/imui/child_region.rs`
- `ecosystem/fret-ui-kit/src/imui/response/widgets.rs`
- `ecosystem/fret-ui-kit/tests/imui_child_region_smoke.rs`
- `tools/gate_imui_workstream_source.py`

## Gates

- `cargo fmt --package fret-ui-kit --check`
- `cargo nextest run -p fret-ui-kit --features imui --test imui_child_region_smoke --no-fail-fast`
- `cargo nextest run -p fret-ui-kit --features imui child_region_resize --no-fail-fast`
- `python tools/gate_imui_workstream_source.py`
- `python tools/check_workstream_catalog.py`
- `python -m json.tool docs/workstreams/imui-child-region-resize-x-v1/WORKSTREAM.json`
- `git diff --check`

## Stay Closed Unless

Start a new proof-led follow-on instead of reopening this lane for:

- auto-resize,
- focus-boundary flattening,
- `BeginChild() -> bool` visibility semantics,
- diagnostics drag simulation for this handle,
- or any broader Dear ImGui flag mirror.
