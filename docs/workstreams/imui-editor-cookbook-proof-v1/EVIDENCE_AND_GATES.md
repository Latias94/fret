# Evidence and Gates

Status: Closed
Last updated: 2026-05-04

## Smallest Repro

```bash
cargo check -p fret-cookbook --features cookbook-imui --example imui_editor_controls_basics
```

## Gates

```bash
cargo fmt --package fretboard --package fret --package fret-ui-editor --package fret-cookbook
cargo check -p fret-cookbook --features cookbook-imui --example imui_editor_controls_basics
cargo nextest run -p fret-cookbook cookbook_imui_editor_example_keeps_public_editor_facade_teaching_surface --no-fail-fast
cargo nextest run -p fretboard-dev cookbook_feature_hints_cover_imui_teaching_examples --no-fail-fast
cargo run -p fretboard-dev -- list cookbook-examples --all
cargo check --tests -p fret-ui-editor --features imui
python tools/check_layering.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-editor-cookbook-proof-v1/WORKSTREAM.json
git diff --check
```

## Verified on 2026-05-04

- `cargo fmt --package fretboard --package fret --package fret-ui-editor --package fret-cookbook`
- `cargo check -p fret-cookbook --features cookbook-imui --example imui_editor_controls_basics`
- `cargo nextest run -p fret-cookbook cookbook_imui_editor_example_keeps_public_editor_facade_teaching_surface --no-fail-fast`
- `cargo nextest run -p fret-cookbook cookbook_imui_example_keeps_current_facade_teaching_surface --no-fail-fast`
- `cargo nextest run -p fretboard-dev cookbook_feature_hints_cover_imui_teaching_examples --no-fail-fast`
- `cargo run -p fretboard-dev -- list cookbook-examples --all`
- `cargo check --tests -p fret-ui-editor --features imui`
- `python tools/check_layering.py`
- `python tools/check_workstream_catalog.py`
- `python -m json.tool docs/workstreams/imui-editor-cookbook-proof-v1/WORKSTREAM.json`
- `git diff --check`

## Evidence Anchors

- `ecosystem/fret/src/lib.rs`
- `ecosystem/fret-ui-editor/src/imui.rs`
- `ecosystem/fret-ui-editor/src/controls/mod.rs`
- `apps/fretboard/src/demos.rs`
- `apps/fret-cookbook/examples/imui_editor_controls_basics.rs`
- `apps/fret-cookbook/src/lib.rs`
- `docs/examples/README.md`

## Upstream Reference Anchors

- `repo-ref/imgui/imgui.h`
- `repo-ref/imgui/imgui_widgets.cpp`
