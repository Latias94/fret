# M1 Selection Ownership Contract - 2026-05-12

Status: First public-surface ownership move

This slice resolves the open M1 question about where the single-selection model belongs.

## Decision

`Selection` is a buffer-layer public type owned by `fret-code-editor-buffer`.

The `fret-code-editor` crate continues to re-export `Selection` from its root so existing app code
using `fret_code_editor::Selection` does not need to change.

## Rationale

`Selection` is expressed in `TextBuffer` UTF-8 byte offsets and is required by edit transactions,
IME replacement, a11y text ranges, and view/display projection. It does not depend on Fret UI,
layout, paint, focus, overlay, or input policy. That makes it part of the model vocabulary rather
than the widget/controller surface.

This keeps ADR 0185's buffer/view/surface split intact:

- buffer: document identity, edits, revisions, transactions, and byte-index selections,
- view: display-row projection and composed row mapping,
- surface: Fret UI integration, input, IME, a11y, paint, caches, and diagnostics hooks.

## Compatibility

- Preserved: `fret_code_editor::Selection`
- Added canonical owner: `fret_code_editor_buffer::Selection`
- No behavior change: `is_caret`, `normalized`, and `caret` keep the same semantics.
- No migration required for current app callers unless they want to depend directly on the buffer
  crate.

## Evidence

- Canonical type: `ecosystem/fret-code-editor-buffer/src/lib.rs`
- Compatibility re-export: `ecosystem/fret-code-editor/src/lib.rs`
- Surface regression: `ecosystem/fret-code-editor/tests/public_surface.rs`

## Gates

```powershell
cargo fmt -p fret-code-editor-buffer --check
cargo fmt -p fret-code-editor --check
cargo check -p fret-code-editor-buffer
cargo nextest run -p fret-code-editor-buffer --lib --no-fail-fast
cargo check -p fret-code-editor --features syntax-rust
cargo nextest run -p fret-code-editor --lib --features syntax --no-fail-fast
cargo nextest run -p fret-code-editor --test public_surface --no-fail-fast
python tools/check_layering.py
```

## Follow-ups

1. Define a multi-selection collection type only after multi-cursor edit transaction semantics are
   designed.
2. Keep view/display projection helpers accepting buffer-owned selection data rather than inventing
   a parallel UI-surface selection model.
