# Evidence and Gates

Status: Closed
Last updated: 2026-05-04

## Smallest Repro

```bash
cargo nextest run -p fret-ui-editor --features imui color_edit --no-fail-fast
cargo check -p fret-cookbook --features cookbook-imui --example imui_editor_controls_basics
```

## Gates

```bash
cargo fmt --package fret-ui-editor
cargo nextest run -p fret-ui-editor --features imui color_edit --no-fail-fast
cargo check --tests -p fret-ui-editor --features imui
cargo nextest run -p fret-ui-editor --features imui --no-fail-fast
cargo check -p fret-cookbook --features cookbook-imui --example imui_editor_controls_basics
python tools/check_layering.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-color-edit-hsv-picker-v1/WORKSTREAM.json
python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
git diff --check
```

## Evidence Anchors

- `ecosystem/fret-ui-editor/src/controls/color_edit.rs`
- `ecosystem/fret-ui-editor/tests/imui_surface_policy.rs`
- `apps/fret-cookbook/examples/imui_editor_controls_basics.rs`
- `docs/audits/imui-imgui-gap-audit-2026-04-22.md`

## Upstream Reference Anchors

- `repo-ref/imgui/imgui.cpp`
- `repo-ref/imgui/imgui_widgets.cpp`
- `repo-ref/imgui/imgui_demo.cpp`
