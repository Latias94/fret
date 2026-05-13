# M1 Value Model Facade Owner Split - 2026-05-13

Status: value model facade owner split landed

## Change

Moved the slider/combo model inherent `ImUiFacade` wrappers into:

- `ecosystem/fret-ui-kit/src/imui/facade_writer/value_models.rs`

`facade_writer.rs` now declares `mod value_models;` and keeps the public trait surface plus the
remaining root glue.

## Evidence

| File | Before | After |
| --- | ---: | ---: |
| `ecosystem/fret-ui-kit/src/imui/facade_writer.rs` | 1474 lines | 1426 lines |
| `ecosystem/fret-ui-kit/src/imui/facade_writer/value_models.rs` | n/a | 53 lines |

## Contract Check

- The moved methods remain inherent methods on `ImUiFacade`.
- No public method names changed.
- No `fret::imui` re-export path changed.
- No `fret-imui` dependency or public surface changed.
- No `crates/fret-ui` runtime contract changed.
