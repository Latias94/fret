# M3 Test Display Navigation Module Split

Status: Landed
Date: 2026-05-12

## Decision

Move display-row navigation tests out of `ecosystem/fret-code-editor/src/editor/tests/mod.rs` into
`ecosystem/fret-code-editor/src/editor/tests/display_navigation.rs`.

The extracted tests cover wrapped vertical movement, Home/End movement, Shift+navigation
extension, page movement, code-wrap policy row boundaries, and fold/inlay row-map hit-testing for
vertical movement.

## Rationale

Display-row navigation is an input navigation owner concern that depends on the view/display map
contract. Keeping these tests together makes the wrap/fold/inlay caret semantics easier to audit
without mixing them with buffer lifecycle, paint materialization, platform text input, or scroll
window tests.

## Non-Goals

This slice does not change caret movement behavior, display-map construction, fold/inlay
composition, page scroll semantics, or public APIs.

## Evidence

- Display navigation test owner: `ecosystem/fret-code-editor/src/editor/tests/display_navigation.rs`
- Parent test module registration: `ecosystem/fret-code-editor/src/editor/tests/mod.rs`
- Input navigation implementation: `ecosystem/fret-code-editor/src/editor/input/navigation.rs`
- Display map implementation: `ecosystem/fret-code-editor-view/src/lib.rs`

## Gates

```powershell
cargo fmt -p fret-code-editor --check
cargo nextest run -p fret-code-editor "move_caret_vertical|home_end|page_down" --no-fail-fast
cargo check -p fret-code-editor --features syntax-rust
cargo nextest run -p fret-code-editor --lib --features syntax --no-fail-fast
```
