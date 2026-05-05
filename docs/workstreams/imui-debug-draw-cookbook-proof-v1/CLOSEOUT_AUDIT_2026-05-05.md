# ImUi Debug Draw Cookbook Proof v1 Closeout Audit - 2026-05-05

Status: closed closeout record.

## What Shipped

- Added `imui_debug_draw_basics` as a runnable cookbook proof behind `cookbook-imui`.
- Taught the public `fret::imui::{prelude::*, kit::*}` path for debug draw without direct
  `fret_ui_kit::imui` imports.
- Exercised clip stack, channel split/merge, multi-color rects, vertex-color triangle meshes,
  image triangle meshes, command summaries, and list summaries.
- Added `fretboard-dev` feature-hint coverage so the recommended runner can discover the required
  feature.
- Updated cookbook and examples indexes.

## Evidence

- `apps/fret-cookbook/examples/imui_debug_draw_basics.rs`
- `apps/fret-cookbook/src/lib.rs`
- `apps/fret-cookbook/Cargo.toml`
- `apps/fretboard/src/demos.rs`
- `apps/fret-cookbook/README.md`
- `apps/fret-cookbook/EXAMPLES.md`
- `docs/examples/README.md`
- `repo-ref/imgui/imgui.h`
- `repo-ref/imgui/imgui_draw.cpp`

## Gates Run

```bash
cargo fmt --package fret-cookbook -- --check
cargo build -p fret-cookbook --example imui_debug_draw_basics --features cookbook-imui
cargo nextest run -p fret-cookbook --lib cookbook_imui_debug_draw_example_keeps_current_facade_teaching_surface --no-fail-fast
cargo nextest run -p fretboard-dev cookbook_feature_hints_cover_imui_teaching_examples --no-fail-fast
python tools/check_layering.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-debug-draw-cookbook-proof-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/imui-editor-grade-product-closure-v1/WORKSTREAM.json
python tools/gate_imui_workstream_source.py
python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
git diff --check
```

## Residual Gaps

- No diagnostics screenshot/bundle proof exists yet for the cookbook debug-draw example.
- Image mesh usage is an API proof with `ImageId::default()`, not an asset pipeline lesson.
- Backend draw-call/scissor attribution and hit-test-aware debug geometry remain separate concerns.
