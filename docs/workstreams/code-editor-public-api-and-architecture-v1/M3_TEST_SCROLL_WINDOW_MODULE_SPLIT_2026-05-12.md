# M3 Test Scroll Window Module Split

Status: Landed
Date: 2026-05-12

## Decision

Move the editor viewport wheel-scroll regression out of
`ecosystem/fret-code-editor/src/editor/tests/mod.rs` into
`ecosystem/fret-code-editor/src/editor/tests/scroll_window.rs`.

The extracted test covers inner windowed-row offset movement, stable viewport bounds, and monotonic
visible-row telemetry after a wheel event.

## Rationale

Scroll window behavior is a UI integration and windowed-surface telemetry owner concern. It depends
on render-flow fixtures and semantics bounds, not on editor model lifecycle, display navigation, or
paint cache materialization. A dedicated module makes this high-level regression easier to find and
extend.

## Non-Goals

This slice does not change wheel handling, windowed rows telemetry, layout bounds, or public APIs.

## Evidence

- Scroll window test owner: `ecosystem/fret-code-editor/src/editor/tests/scroll_window.rs`
- Parent test module registration: `ecosystem/fret-code-editor/src/editor/tests/mod.rs`
- Shared scroll fixture helpers: `ecosystem/fret-code-editor/src/editor/tests/support.rs`
- Windowed rows surface implementation:
  `ecosystem/fret-ui-kit/src/declarative/windowed_rows_surface.rs`

## Gates

```powershell
cargo fmt -p fret-code-editor --check
cargo nextest run -p fret-code-editor editor_viewport_wheel_scroll --no-fail-fast
cargo check -p fret-code-editor --features syntax-rust
cargo nextest run -p fret-code-editor --lib --features syntax --no-fail-fast
```
