# M0 Baseline Audit - 2026-05-13

Status: baseline captured

## Current Source Shape

Before M1:

| File | Baseline |
| --- | ---: |
| `ecosystem/fret-ui-kit/src/imui/facade_writer.rs` | 1632 lines before M1 |
| `ecosystem/fret-ui-kit/src/imui/facade_writer/text_models.rs` | n/a |

The text model wrapper cluster still lived in `facade_writer.rs`:

- `input_text_model(...)`
- `input_text_model_with_options(...)`
- `input_text_completion_model(...)`
- `input_text_completion_model_with_options(...)`
- `input_text_history_model(...)`
- `input_text_history_model_with_options(...)`
- `textarea_model(...)`
- `textarea_model_with_options(...)`

## Decision

Move only those inherent wrappers to a private `facade_writer/text_models.rs` owner. Leave trait
methods, text control behavior, picker behavior, and public paths unchanged.

## Non-Goals

- No public method renames.
- No `fret-imui` dependency or public surface changes.
- No `crates/fret-ui` runtime contract changes.
- No new text input or picker behavior.
