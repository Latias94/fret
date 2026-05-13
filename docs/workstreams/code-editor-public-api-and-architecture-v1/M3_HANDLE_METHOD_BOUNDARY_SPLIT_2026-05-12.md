# M3 Handle Method Boundary Split

Status: Landed
Date: 2026-05-12

## Decision

`CodeEditorHandle` now keeps the public struct and constructor in `handle.rs`, while method groups
live in feature-owned child modules:

- `handle/model.rs`: model mutation, selection, buffer replacement, interaction, undo readouts.
- `handle/view.rs`: language, code-font feature policy, text-boundary mode, folds, inlays, soft
  wrap, and code-wrap policy.
- `handle/feature_payloads.rs`: diagnostics, range decorations, gutter markers, semantic tokens,
  and payload snapshots.
- `handle/diagnostics.rs`: cache stats, memory snapshots, paint perf readouts, and diagnostics-only
  cached substring checks.
- `handle/debug.rs`: debug/staging IME helpers, inline-preedit staging toggles, and decorated-line
  text inspection.

This is a source-ownership split only. Public method names, signatures, and root exports are
unchanged.

## Rationale

The initial public surface classification identified `CodeEditorHandle` as the main dumping-ground
risk. Splitting by method owner makes future public API reviews cheaper:

- feature payload changes can stay in the extension-surface module,
- diagnostics/perf hooks can evolve without hiding among model setters,
- debug/staging methods remain visibly separate from stable model/view methods,
- view configuration changes can be reviewed against `DisplayMap` and cache invalidation rules.

## Evidence

- Handle root: `ecosystem/fret-code-editor/src/editor/handle.rs`
- Model methods: `ecosystem/fret-code-editor/src/editor/handle/model.rs`
- View methods: `ecosystem/fret-code-editor/src/editor/handle/view.rs`
- Feature payload methods: `ecosystem/fret-code-editor/src/editor/handle/feature_payloads.rs`
- Diagnostics methods: `ecosystem/fret-code-editor/src/editor/handle/diagnostics.rs`
- Debug/staging methods: `ecosystem/fret-code-editor/src/editor/handle/debug.rs`

## Gates

```powershell
cargo fmt -p fret-code-editor --check
cargo check -p fret-code-editor --features syntax-rust
cargo nextest run -p fret-code-editor --test public_surface --no-fail-fast
cargo nextest run -p fret-code-editor feature_payload --no-fail-fast
cargo nextest run -p fret-code-editor --lib --features syntax --no-fail-fast
python tools/check_layering.py
git diff --check
```
