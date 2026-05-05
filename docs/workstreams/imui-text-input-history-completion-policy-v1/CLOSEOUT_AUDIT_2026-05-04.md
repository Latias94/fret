# Closeout Audit - 2026-05-04

Status: Closed

## Verdict

Closed. Single-line IMUI input text now has command-oriented completion/history key policy without
adding Dear ImGui-style mutable buffer callbacks to the runtime.

## Goal-Backward Audit

- Completion command:
  - Evidence: `ecosystem/fret-ui-kit/src/imui/options/controls.rs` and
    `ecosystem/fret-ui-kit/src/imui/text_controls.rs`
  - Result: `InputTextOptions::completion_command` dispatches on unmodified Tab when focused.
- History commands:
  - Evidence: `ecosystem/fret-ui-kit/src/imui/options/controls.rs` and
    `ecosystem/fret-ui-kit/src/imui/text_controls.rs`
  - Result: `history_previous_command` and `history_next_command` dispatch on unmodified Up/Down.
- Runtime boundary:
  - Evidence: no `crates/fret-ui` changes in this lane.
  - Result: completion/history remains `fret-ui-kit::imui` policy.
- Regression floor:
  - Evidence: `ecosystem/fret-imui/src/tests/models_text.rs`
  - Result: focused tests cover completion command dispatch, history command dispatch, and default
    repeat suppression.

## Gates Run

- `cargo fmt --package fret-ui-kit --package fret-imui`
- `cargo check -p fret-ui-kit --tests`
- `cargo nextest run -p fret-imui input_text_completion_command_dispatches_on_unmodified_tab --no-fail-fast`
- `cargo nextest run -p fret-imui input_text_history_commands_dispatch_on_unmodified_arrows_without_default_repeat --no-fail-fast`
- `cargo nextest run -p fret-imui models_text --no-fail-fast`
- `cargo check -p fret-cookbook --features cookbook-imui --example imui_action_basics`
- `python tools/check_layering.py`
- `python tools/check_workstream_catalog.py`
- `python -m json.tool docs/workstreams/imui-text-input-history-completion-policy-v1/WORKSTREAM.json`
- `python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`
- `git diff --check`

## Follow-On Policy

Do not reopen this lane for broader text editing callbacks. Later narrow follow-ons cover character
filters and undo/redo command routing. Start separate follow-ons for completion/history popup
recipes, callback-edit behavior, or multiline text editing depth.
