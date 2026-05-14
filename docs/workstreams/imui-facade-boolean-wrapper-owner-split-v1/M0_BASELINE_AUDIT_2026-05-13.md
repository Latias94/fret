# M0 Baseline Audit - 2026-05-13

Status: baseline captured

## Current Source Shape

Before M1:

| File | Baseline |
| --- | ---: |
| `ecosystem/fret-ui-kit/src/imui/facade_writer.rs` | 1537 lines before M1 |
| `ecosystem/fret-ui-kit/src/imui/facade_writer/boolean_wrappers.rs` | n/a |

The boolean wrapper cluster still lived in `facade_writer.rs`:

- `checkbox_model(...)`
- `checkbox_model_with_options(...)`
- `radio(...)`
- `radio_with_options(...)`
- `switch_model(...)`
- `switch_model_with_options(...)`

## Decision

Move only those inherent wrappers to a private `facade_writer/boolean_wrappers.rs` owner. Leave
trait methods, boolean control behavior, and public paths unchanged.

## Non-Goals

- No public method renames.
- No `fret-imui` dependency or public surface changes.
- No `crates/fret-ui` runtime contract changes.
- No new boolean control behavior.
