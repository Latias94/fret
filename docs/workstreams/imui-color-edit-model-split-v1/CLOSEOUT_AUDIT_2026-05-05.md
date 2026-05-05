# ImUi Color Edit Model Split v1 Closeout Audit - 2026-05-05

Status: Closed.

This lane closes a cleanup-only follow-on after the completed `ColorEdit` popup/options depth work.
It reduces the editor control god-file risk without changing the public control surface.

## What Shipped

- Added `ecosystem/fret-ui-editor/src/controls/color_edit/model.rs`.
- Added `ecosystem/fret-ui-editor/src/controls/color_edit/tests.rs`.
- Moved pure model helpers out of `color_edit.rs`:
  - HSV model and RGB/HSV conversion,
  - hex and numeric parsing/formatting,
  - numeric input mode selection,
  - pointer coordinate normalization,
  - sanitization helpers,
  - and a11y value text helpers.
- Updated `imui_surface_policy` so source assertions track the new helper owner.
- Kept public `ColorEdit` options, popup composition, swatch rendering, and interaction wiring in
  `color_edit.rs`.

## Proof

- `cargo fmt --package fret-ui-editor -- --check` passes.
- `cargo nextest run -p fret-ui-editor --features imui color_edit --no-fail-fast` passes.
- `cargo nextest run -p fret-ui-editor --features imui --test imui_surface_policy --no-fail-fast`
  passes.
- `cargo nextest run -p fret-ui-editor --features imui --test imui_adapter_smoke --no-fail-fast`
  passes.
- `cargo nextest run -p fret-ui-editor --features imui --no-fail-fast` passes.
- `python tools/check_layering.py` passes.
- `python tools/check_workstream_catalog.py` passes.
- `python -m json.tool docs/workstreams/imui-color-edit-model-split-v1/WORKSTREAM.json` passes.
- `python tools/gate_imui_workstream_source.py` passes.
- `python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols` passes.
- `git diff --check` passes.

## Remaining Work

This split intentionally stops short of a broad module tree. Start separate follow-ons for:

- extracting popup rendering/composition into smaller UI modules,
- fixture-driven color parser/conversion conformance,
- color history or palette customization,
- eyedropper integration,
- color drag/drop payloads,
- or visual HueWheel fidelity.
