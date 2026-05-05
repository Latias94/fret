# Closeout Audit - 2026-05-04

Status: Closed

## Verdict

Closed. The lane now proves the public `fret::imui::editor` cookbook path for editor-grade
immediate-mode controls without direct `fret_ui_editor` imports.

## Goal-Backward Audit

- App-facing import path:
  - Evidence: `apps/fret-cookbook/examples/imui_editor_controls_basics.rs`
  - Result: the example imports `fret::imui::{editor, prelude::*}`.
- Editor-grade controls:
  - Evidence: `apps/fret-cookbook/examples/imui_editor_controls_basics.rs`
  - Result: numeric input, drag value, color edit, mini search, and text assist are rendered through
    `fret::imui::editor::*` adapters.
- Support noun discoverability:
  - Evidence: `ecosystem/fret-ui-editor/src/controls/mod.rs`
  - Result: text-assist support nouns are re-exported from the editor control surface.
- Teaching-surface regression:
  - Evidence: `apps/fret-cookbook/src/lib.rs`
  - Result: the cookbook source-policy test forbids direct `fret_ui_editor` and raw
    `fret_ui_kit::headless::text_assist` imports in the new example.
- Runnable dev entry:
  - Evidence: `apps/fretboard/src/demos.rs`
  - Result: `fretboard dev native --example imui_editor_controls_basics` can auto-enable
    `cookbook-imui` through the cookbook feature hint table.
- First-contact docs:
  - Evidence: `docs/examples/README.md`
  - Result: generic IMUI starts at `imui_action_basics`; editor-control first contact starts at
    `imui_editor_controls_basics`; the heavier `imui_editor_proof_demo` remains a product proof.

## Gates Run

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

## Follow-On Policy

Do not reopen this lane for broader editor behavior work. Start narrower follow-ons for deeper
TextField behavior, color picker popup parity, drag/slider scalar breadth, or multi-window editor
proofs.
