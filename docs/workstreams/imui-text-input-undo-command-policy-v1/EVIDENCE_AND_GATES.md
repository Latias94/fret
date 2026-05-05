# ImUi Text Input Undo Command Policy v1 Evidence And Gates

Status: Closed
Last updated: 2026-05-04

## Repro

- Render a model-backed single-line IMUI input with `undo_command` and `redo_command`.
- Focus the input.
- Dispatch Ctrl+Z, Ctrl+Y, Ctrl+Shift+Z, unsupported modified keys, and repeated keydown.
- Assert only the app-owned commands allowed by `InputTextOptions` are emitted.

## Evidence Anchors

- `repo-ref/imgui/imgui.h`
- `repo-ref/imgui/imgui_widgets.cpp`
- `ecosystem/fret-ui-kit/src/imui/options/controls.rs`
- `ecosystem/fret-ui-kit/src/imui/text_controls.rs`
- `ecosystem/fret-imui/src/tests/models_text.rs`
- `docs/adr/0024-undo-redo-and-edit-transactions.md`
- `docs/adr/0066-fret-ui-runtime-contract-surface.md`

## Gates

```bash
cargo fmt --package fret-ui-kit --package fret-imui
cargo check -p fret-ui-kit --tests
cargo nextest run -p fret-imui input_text_undo_redo input_text_policy_commands_can_opt_into_repeat --no-fail-fast
cargo nextest run -p fret-imui models_text --no-fail-fast
python tools/check_layering.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-text-input-undo-command-policy-v1/WORKSTREAM.json
python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
git diff --check
```
