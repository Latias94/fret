# M3 Syntax Prefetch Runtime Split

Status: Landed
Date: 2026-05-12

## Decision

The editor syntax prefetch runtime types moved out of
`ecosystem/fret-code-editor/src/editor/mod.rs` into
`ecosystem/fret-code-editor/src/editor/syntax.rs`.

The new owner contains:

- `SyntaxSpan`
- `SyntaxPrefetchKey`
- `SyntaxPrefetchChunk`
- `SyntaxPrefetchRuntimeState`
- `SyntaxPrefetchRuntime`

Existing paint and state call paths are unchanged. `paint/mod.rs` still owns syntax cache
population, invalidation, and highlight materialization for now.

## Rationale

Syntax prefetch state is a cross-frame runtime owner, not widget construction. Keeping it in
`editor/mod.rs` hid the boundary between editor surface state and syntax work scheduling. Moving
the runtime types into `syntax.rs` creates a clear follow-up path for extracting syntax cache and
highlight materialization from the paint module without changing behavior first.

## Non-Goals

This slice does not change syntax highlighting behavior, cache keys, prefetch candidate selection,
pending/ready queue limits, background dispatch priority, paint attribution, or public editor APIs.

The broader `syntax` owner split remains open until syntax cache population, invalidation, and
materialization have a dedicated owner outside the monolithic paint module.

## Evidence

- Syntax runtime owner: `ecosystem/fret-code-editor/src/editor/syntax.rs`
- Surface integration: `ecosystem/fret-code-editor/src/editor/mod.rs`
- Current syntax cache/highlight owner: `ecosystem/fret-code-editor/src/editor/paint/mod.rs`
- Syntax cache regression tests: `ecosystem/fret-code-editor/src/editor/tests/syntax.rs`

## Gates

```powershell
cargo fmt -p fret-code-editor --check
cargo check -p fret-code-editor
cargo check -p fret-code-editor --features syntax-rust
cargo nextest run -p fret-code-editor --lib --features syntax --no-fail-fast
python tools/check_layering.py
git diff --check
```
