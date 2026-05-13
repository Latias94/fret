# M3 Test Owner Residual Closure

Status: Landed
Date: 2026-05-12

## Decision

Move the remaining concrete tests out of `ecosystem/fret-code-editor/src/editor/tests/mod.rs` into
feature-owned modules:

- `ecosystem/fret-code-editor/src/editor/tests/syntax_window.rs`
- `ecosystem/fret-code-editor/src/editor/tests/paint_guards.rs`
- `ecosystem/fret-code-editor/src/editor/tests/fold_lifecycle.rs`
- `ecosystem/fret-code-editor/src/editor/tests/edit_refresh.rs`

After this slice, `tests/mod.rs` keeps shared fixtures, service doubles, module registration, and
support imports; concrete behavior tests live in owner modules.

## Rationale

The remaining tests were small but belonged to distinct owners: syntax prefetch windowing, paint
hot-path source guards, fold lifecycle selection snapping, and input edit/display-map refresh.
Splitting them finishes the M3 test ownership cleanup without changing behavior or runtime code.

## Non-Goals

This slice does not move shared fixtures into `support.rs`, change syntax prefetch logic, change
paint materialization, change fold behavior, change edit/display-map refresh semantics, or change
public APIs.

## Evidence

- Syntax window test owner: `ecosystem/fret-code-editor/src/editor/tests/syntax_window.rs`
- Paint guard test owner: `ecosystem/fret-code-editor/src/editor/tests/paint_guards.rs`
- Fold lifecycle test owner: `ecosystem/fret-code-editor/src/editor/tests/fold_lifecycle.rs`
- Edit/display-map refresh test owner: `ecosystem/fret-code-editor/src/editor/tests/edit_refresh.rs`
- Parent test module registration and shared fixtures:
  `ecosystem/fret-code-editor/src/editor/tests/mod.rs`

## Gates

```powershell
cargo fmt -p fret-code-editor --check
cargo nextest run -p fret-code-editor syntax_window --features syntax --no-fail-fast
cargo nextest run -p fret-code-editor paint_source --no-fail-fast
cargo nextest run -p fret-code-editor enabling_folds --no-fail-fast
cargo nextest run -p fret-code-editor apply_and_record_edit_refreshes_display_map --no-fail-fast
cargo check -p fret-code-editor --features syntax-rust
cargo nextest run -p fret-code-editor --lib --features syntax --no-fail-fast
```
