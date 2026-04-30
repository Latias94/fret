# Fret Examples Build Latency v1 - M17 API Workbench Source Gate - 2026-04-30

Status: complete

## Decision

Move the API workbench lite source-only marker check out of the monolithic `fret-examples` Rust
unit-test module and into `python tools/gate_examples_source_tree_policy.py`.

## Migrated Check

- `api_workbench_lite_demo_uses_query_for_sqlite_reads_and_mutation_for_explicit_submit`

## Behavior

The examples source-tree policy gate now owns:

- API workbench default app surface markers,
- generic `AppRenderContext` shell helper markers,
- capability-first dialog/element landing markers,
- SQLite history query/mutation ownership markers,
- retry and mutation completion flow markers,
- legacy raw `cx.elements()` and direct history-owner regressions.

No runtime behavior change is intended. This slice only moves pure source scanning away from Rust
unit tests that require compiling `fret-examples`.

## Evidence

- `tools/examples_source_tree_policy/gate.py`
- `apps/fret-examples/src/lib.rs`

## Result

The Rust `#[test]` count in `apps/fret-examples/src/lib.rs` dropped from 71 to 70.

## Gates

```text
python tools/gate_examples_source_tree_policy.py
python -m py_compile tools/gate_examples_source_tree_policy.py tools/examples_source_tree_policy/__init__.py tools/examples_source_tree_policy/gate.py
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
