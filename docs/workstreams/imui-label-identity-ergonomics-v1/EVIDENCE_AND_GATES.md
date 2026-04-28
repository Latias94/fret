# ImUi Label Identity Ergonomics v1 - Evidence and Gates

Status: closed
Last updated: 2026-04-28

## Smallest Repro

- `cargo nextest run -p fret-ui-kit --features imui imui_label_identity --no-fail-fast`
- `cargo nextest run -p fret-imui label_identity --no-fail-fast`

## Current Evidence

- `docs/workstreams/imui-control-geometry-stability-v1/CLOSEOUT_AUDIT_2026-04-28.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-04-19.md`
- `docs/workstreams/imui-label-identity-ergonomics-v1/M1_BUTTON_LABEL_IDENTITY_SLICE_2026-04-28.md`
- `docs/workstreams/imui-label-identity-ergonomics-v1/M2_SELECTABLE_MENU_LABEL_IDENTITY_SLICE_2026-04-28.md`
- `docs/workstreams/imui-label-identity-ergonomics-v1/M2_MODEL_AND_EXPLICIT_ID_LABEL_IDENTITY_SLICE_2026-04-28.md`
- `docs/workstreams/imui-label-identity-ergonomics-v1/CLOSEOUT_AUDIT_2026-04-28.md`
- `docs/adr/0319-public-authoring-state-lanes-and-identity-contract-v1.md`
- `Cargo.toml`
- `ecosystem/fret-imui/src/frontend.rs`
- `ecosystem/fret-imui/src/tests/label_identity.rs`
- `ecosystem/fret-ui-kit/src/imui/label_identity.rs`
- `ecosystem/fret-ui-kit/src/imui/button_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/boolean_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/combo_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/disclosure_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/menu_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/menu_family_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/selectable_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/separator_text_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/slider_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/tab_family_controls.rs`
- `repo-ref/imgui/imgui.h`
- `repo-ref/imgui/imgui.cpp`
- `repo-ref/imgui/imgui_widgets.cpp`

## Gate Set

Final lane gates:

- `cargo nextest run -p fret-ui-kit --features imui imui_label_identity --no-fail-fast`
- `cargo nextest run -p fret-imui label_identity --no-fail-fast`
- `cargo check -p fret-ui-kit --features imui --jobs 1`
- `cargo fmt --package fret-ui-kit --package fret-imui --check`
- `python tools/check_workstream_catalog.py`
- `python -m json.tool docs/workstreams/imui-label-identity-ergonomics-v1/WORKSTREAM.json`
- `git diff --check`

## Gate Notes

- `fret-ui-kit` parser tests reproduced a Windows/MSVC LNK1120 failure when test incremental
  artifacts were enabled.
- `[profile.test.package.fret-ui-kit] incremental = false` keeps the standard nextest command
  deterministic without requiring callers to set `CARGO_INCREMENTAL=0`.

## Non-Gates

- No Linux compositor acceptance.
- No full runtime ID-stack debugger.
- No test-id inference from labels.
- No localization policy change.
