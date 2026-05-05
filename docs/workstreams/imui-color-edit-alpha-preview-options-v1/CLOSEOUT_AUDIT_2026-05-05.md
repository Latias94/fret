# ImUi Color Edit Alpha Preview Options v1 Closeout Audit - 2026-05-05

Status: Closed.

This lane adds per-control alpha preview modes to editor `ColorEdit`, matching Dear ImGui's
ColorButton preview policy axis while keeping Fret policy explicit and app-owned.

## What Shipped

- Added `ColorEditAlphaPreview::{Checkerboard, Opaque, NoBackground, Half}`.
- Added `ColorEditOptions::alpha_preview`, defaulting to checkerboard-backed transparent preview.
- Updated the root swatch and preset swatches to use the selected preview mode.
- Added preview rendering helpers for opaque, no-background, and half-alpha previews.
- Added focused tests and source-policy anchors for the new option surface.

## Proof

- `cargo fmt --package fret-ui-editor -- --check` passes.
- `cargo nextest run -p fret-ui-editor --features imui color_edit --no-fail-fast` passes.
- `cargo nextest run -p fret-ui-editor --features imui --test imui_surface_policy --no-fail-fast`
  passes.
- `cargo nextest run -p fret-ui-editor --features imui --no-fail-fast` passes.
- `python tools/check_layering.py` passes.
- `python tools/check_workstream_catalog.py` passes.
- `python -m json.tool docs/workstreams/imui-color-edit-alpha-preview-options-v1/WORKSTREAM.json`
  passes.
- `python tools/gate_imui_workstream_source.py` passes.
- `python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols` passes.
- `git diff --check` passes.

## Remaining Work

Continue with separate ColorEdit/ColorPicker feature follow-ons: HueWheel fidelity, drag/drop
payloads, palette customization, color history, and richer side-preview/reference-color affordances.
