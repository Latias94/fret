# ImUi Label Identity Ergonomics v1 - Evidence and Gates

Status: active execution lane
Last updated: 2026-04-28

## Smallest Repro

- `cargo nextest run -p fret-ui-kit --features imui imui_label_identity --no-fail-fast`
- `cargo nextest run -p fret-imui label_identity --no-fail-fast`

## Current Evidence

- `docs/workstreams/imui-control-geometry-stability-v1/CLOSEOUT_AUDIT_2026-04-28.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-04-19.md`
- `docs/adr/0319-public-authoring-state-lanes-and-identity-contract-v1.md`
- `ecosystem/fret-imui/src/frontend.rs`
- `ecosystem/fret-ui-kit/src/imui/button_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/menu_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/selectable_controls.rs`
- `repo-ref/imgui/imgui.h`
- `repo-ref/imgui/imgui.cpp`
- `repo-ref/imgui/imgui_widgets.cpp`

## Gate Set

Initial lane gates:

- `cargo nextest run -p fret-ui-kit --features imui imui_label_identity --no-fail-fast`
- `cargo nextest run -p fret-imui label_identity --no-fail-fast`
- `cargo check -p fret-ui-kit --features imui --jobs 2`
- `python tools/check_workstream_catalog.py`
- `python -m json.tool docs/workstreams/imui-label-identity-ergonomics-v1/WORKSTREAM.json`
- `git diff --check`

## Non-Gates

- No Linux compositor acceptance.
- No full runtime ID-stack debugger.
- No test-id inference from labels.
- No localization policy change.
