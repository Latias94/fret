# ImUi Color Edit Popup Numeric Split v1 Closeout Audit - 2026-05-05

Status: Closed.

This lane moves editable RGB/HSV popup numeric rows into an internal `popup/numeric.rs` owner.

## What Shipped

- Added `ecosystem/fret-ui-editor/src/controls/color_edit/popup/numeric.rs`.
- Moved numeric row composition, field placeholders, error line rendering, and Enter/Escape commit
  handling out of `popup.rs`.
- Updated source-policy anchors to track the new numeric owner.

## Proof

- `cargo fmt --package fret-ui-editor -- --check` passes.
- `cargo nextest run -p fret-ui-editor --features imui color_edit --no-fail-fast` passes.
- `cargo nextest run -p fret-ui-editor --features imui --test imui_surface_policy --no-fail-fast`
  passes.
- `cargo nextest run -p fret-ui-editor --features imui --no-fail-fast` passes.
- `python tools/check_layering.py` passes.
- `python tools/check_workstream_catalog.py` passes.
- `python -m json.tool docs/workstreams/imui-color-edit-popup-numeric-split-v1/WORKSTREAM.json`
  passes.
- `python tools/gate_imui_workstream_source.py` passes.
- `python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols` passes.
- `git diff --check` passes.

## Remaining Work

Continue with separate popup visual/picker/swatches splits, fixture-driven numeric conformance, or
new color feature follow-ons.
