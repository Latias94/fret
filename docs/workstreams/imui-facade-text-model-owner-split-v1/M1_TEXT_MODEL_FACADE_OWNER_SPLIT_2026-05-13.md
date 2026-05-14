# M1 Text Model Facade Owner Split - 2026-05-13

Status: text model facade owner split landed

## Change

Moved the text and textarea model-backed inherent `ImUiFacade` wrappers into:

- `ecosystem/fret-ui-kit/src/imui/facade_writer/text_models.rs`

`facade_writer.rs` now declares `mod text_models;` and keeps the public trait surface plus the
remaining root glue.

## Evidence

| File | Before | After |
| --- | ---: | ---: |
| `ecosystem/fret-ui-kit/src/imui/facade_writer.rs` | 1632 lines | 1537 lines |
| `ecosystem/fret-ui-kit/src/imui/facade_writer/text_models.rs` | n/a | 99 lines |

## Contract Check

- The moved methods remain inherent methods on `ImUiFacade`.
- No public method names changed.
- No `fret::imui` re-export path changed.
- No `fret-imui` dependency or public surface changed.
- No `crates/fret-ui` runtime contract changed.
