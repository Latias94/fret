# M0 Baseline Audit - 2026-05-13

Status: baseline captured

## Current Source Shape

Before M1:

| File | Baseline |
| --- | ---: |
| `ecosystem/fret-ui-kit/src/imui/facade_writer.rs` | 1474 lines before M1 |
| `ecosystem/fret-ui-kit/src/imui/facade_writer/value_models.rs` | n/a |

The value model wrapper cluster still lived in `facade_writer.rs`:

- `slider_f32_model(...)`
- `slider_f32_model_with_options(...)`
- `combo_model(...)`
- `combo_model_with_options(...)`

## Decision

Move only those inherent wrappers to a private `facade_writer/value_models.rs` owner. Leave trait
methods, slider/combo behavior, and public paths unchanged.

## Non-Goals

- No public method renames.
- No `fret-imui` dependency or public surface changes.
- No `crates/fret-ui` runtime contract changes.
- No new slider/combo behavior.
