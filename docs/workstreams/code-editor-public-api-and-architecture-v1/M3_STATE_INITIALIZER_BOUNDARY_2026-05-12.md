# M3 State Initializer Boundary

Status: Landed
Date: 2026-05-12

## Decision

The default editor-state initializer moved from `CodeEditorHandle::new` into
`CodeEditorState::new` in `ecosystem/fret-code-editor/src/editor/state.rs`.

`CodeEditorHandle::new` now owns only public handle construction concerns:

- create a valid `TextBuffer` from app-provided text,
- fall back to an empty valid buffer if construction fails,
- wrap the state in `Rc<RefCell<_>>`.

The state module owns default display-map, cache, diagnostics, paint-perf, syntax-cache, and
feature-payload initialization.

## Coverage

The moved initializer covers:

- initial `DisplayMap`,
- text boundary and interaction defaults,
- undo/cache defaults,
- row text/geometry/scene cache defaults,
- paint-frame and paint-perf defaults,
- syntax-only cache defaults,
- feature payload store defaults tied to the initial buffer revision.

## Non-Goals

This slice does not split the public `CodeEditorHandle` method surface, change public API shape,
change default state values, change cache behavior, or change diagnostics/perf schemas.

## Evidence

- State initializer: `ecosystem/fret-code-editor/src/editor/state.rs`
- Public handle constructor: `ecosystem/fret-code-editor/src/editor/mod.rs`
- Public surface regression: `ecosystem/fret-code-editor/tests/public_surface.rs`
- Default-state behavior regression: `ecosystem/fret-code-editor/src/editor/tests/mod.rs`

## Gates

```powershell
cargo fmt -p fret-code-editor --check
cargo check -p fret-code-editor --features syntax-rust
cargo nextest run -p fret-code-editor --test public_surface --no-fail-fast
cargo nextest run -p fret-code-editor --lib --features syntax --no-fail-fast
```
