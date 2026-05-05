# Milestones

Status: Closed
Last updated: 2026-05-04

## M0 - Target Surface Freeze

Status: Complete

Exit criteria:

- The example imports `fret::imui::{editor, prelude::*}`.
- The example does not import `fret_ui_editor` directly.
- Any needed supporting nouns are reachable from `fret::imui::editor`.

## M1 - Cookbook Proof

Status: Complete

Exit criteria:

- `imui_editor_controls_basics.rs` compiles with `--features cookbook-imui`.
- `fretboard dev native --example imui_editor_controls_basics` can discover the required
  `cookbook-imui` feature hint.
- The example demonstrates numeric input, drag value, color edit, mini search, and text assist.
- The example installs the editor theme through the public facade.

## M2 - Regression Gates

Status: Complete

Exit criteria:

- Cookbook source-policy tests lock the facade path.
- The workstream catalog, layering check, and formatting gates pass.
