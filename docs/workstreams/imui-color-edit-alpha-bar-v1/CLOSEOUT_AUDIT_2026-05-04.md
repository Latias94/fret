# Closeout Audit - 2026-05-04

Status: Closed

## Verdict

Closed. `ColorEdit` now has a bounded AlphaBar-style popup affordance for direct alpha editing
when `show_alpha=true`.

## Goal-Backward Audit

- AlphaBar visibility:
  - Evidence: `ecosystem/fret-ui-editor/src/controls/color_edit.rs`
  - Result: popup content adds `alpha_bar(...)` only when `show_alpha` is enabled.
- Alpha edit behavior:
  - Evidence: `ecosystem/fret-ui-editor/src/controls/color_edit.rs`
  - Result: pointer down / drag map local x to a clamped alpha value, update `Color.a`, sync
    `#RRGGBBAA`, clear errors, and request redraw.
- Regression floor:
  - Evidence: `ecosystem/fret-ui-editor/src/controls/color_edit.rs` and
    `ecosystem/fret-ui-editor/tests/imui_surface_policy.rs`
  - Result: alpha mapping and a11y percent formatting are unit-tested, while source-policy coverage
    keeps AlphaBar infrastructure in the editor-owned implementation.

## Gates Run

- `cargo fmt --package fret-ui-editor`
- `cargo nextest run -p fret-ui-editor --features imui color_edit --no-fail-fast`
- `cargo nextest run -p fret-ui-editor --features imui --no-fail-fast`
- `cargo check --tests -p fret-ui-editor --features imui`
- `cargo check -p fret-cookbook --features cookbook-imui --example imui_editor_controls_basics`
- `python tools/check_layering.py`
- `python tools/check_workstream_catalog.py`
- `python -m json.tool docs/workstreams/imui-color-edit-alpha-bar-v1/WORKSTREAM.json`
- `python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`
- `git diff --check`

## Follow-On Policy

Do not reopen this lane for full picker parity. HSV/RGB picker depth is covered by
`docs/workstreams/imui-color-edit-hsv-picker-v1/CLOSEOUT_AUDIT_2026-05-04.md`, and per-control
popup defaults are covered by
`docs/workstreams/imui-color-edit-popup-options-v1/CLOSEOUT_AUDIT_2026-05-05.md`. Start separate
follow-ons for color history, eyedropper integration, or color drag/drop payloads.
