# M3 Test Pointer Helpers Module Split

Status: Landed
Date: 2026-05-12

## Decision

Move pointer helper tests out of `ecosystem/fret-code-editor/src/editor/tests/mod.rs` into
`ecosystem/fret-code-editor/src/editor/tests/pointer_helpers.rs`.

The extracted tests cover drag-autoscroll delta direction/clamping and pointer-y to display-row
clamping/rejection.

## Rationale

Pointer helper behavior is an input hot-path foundation for selection, drag autoscroll, and
windowed row hit-testing. These helper tests do not need the larger selection setup, row-geometry
cache fixtures, or platform text-input scaffolding, so keeping them in the monolithic editor test
module made ownership harder to see.

This split gives the low-level pointer helpers a clear owner without changing behavior, public
APIs, or performance thresholds.

## Non-Goals

This slice does not move pointer selection, double-click/triple-click, shift-drag, or navigation
tests. Those remain follow-up slices.

## Evidence

- Pointer helper test owner: `ecosystem/fret-code-editor/src/editor/tests/pointer_helpers.rs`
- Parent test module registration: `ecosystem/fret-code-editor/src/editor/tests/mod.rs`
- Pointer helper implementation: `ecosystem/fret-code-editor/src/editor/mod.rs`

## Gates

```powershell
cargo fmt -p fret-code-editor --check
cargo check -p fret-code-editor
cargo nextest run -p fret-code-editor drag_autoscroll --no-fail-fast
cargo nextest run -p fret-code-editor display_row_for_pointer --no-fail-fast
cargo nextest run -p fret-code-editor --lib --features syntax --no-fail-fast
```
