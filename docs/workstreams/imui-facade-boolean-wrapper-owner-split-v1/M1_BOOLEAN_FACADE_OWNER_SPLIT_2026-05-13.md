# M1 Boolean Facade Owner Split - 2026-05-13

Status: boolean facade owner split landed

## Change

Moved the checkbox/radio/switch inherent `ImUiFacade` wrappers into:

- `ecosystem/fret-ui-kit/src/imui/facade_writer/boolean_wrappers.rs`

`facade_writer.rs` now declares `mod boolean_wrappers;` and keeps the public trait surface plus the
remaining root glue.

## Evidence

| File | Before | After |
| --- | ---: | ---: |
| `ecosystem/fret-ui-kit/src/imui/facade_writer.rs` | 1537 lines | 1474 lines |
| `ecosystem/fret-ui-kit/src/imui/facade_writer/boolean_wrappers.rs` | n/a | 67 lines |

## Contract Check

- The moved methods remain inherent methods on `ImUiFacade`.
- No public method names changed.
- No `fret::imui` re-export path changed.
- No `fret-imui` dependency or public surface changed.
- No `crates/fret-ui` runtime contract changed.
