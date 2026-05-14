# M1 Container Facade Owner Split - 2026-05-13

Status: container facade owner split landed

## Change

Moved the structural container inherent `ImUiFacade` wrappers into:

- `ecosystem/fret-ui-kit/src/imui/facade_writer/container_wrappers.rs`

`facade_writer.rs` now declares `mod container_wrappers;` and keeps the public trait surface plus
the remaining root glue.

## Evidence

| File | Before | After |
| --- | ---: | ---: |
| `ecosystem/fret-ui-kit/src/imui/facade_writer.rs` | 1275 lines | 1113 lines |
| `ecosystem/fret-ui-kit/src/imui/facade_writer/container_wrappers.rs` | n/a | 166 lines |

## Contract Check

- The moved methods remain inherent methods on `ImUiFacade`.
- No public method names changed.
- No `fret::imui` re-export path changed.
- No `fret-imui` dependency or public surface changed.
- No `crates/fret-ui` runtime contract changed.
