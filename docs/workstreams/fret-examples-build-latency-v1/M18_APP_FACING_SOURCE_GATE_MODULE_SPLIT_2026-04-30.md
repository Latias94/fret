# Fret Examples Build Latency v1 - M18 App Facing Source Gate Module Split - 2026-04-30

Status: complete

## Decision

Split app-facing demo source-policy matrices out of `tools/examples_source_tree_policy/gate.py`
into `tools/examples_source_tree_policy/app_facing.py`.

## Rationale

- The stable command entrypoint remains `python tools/gate_examples_source_tree_policy.py`.
- `gate.py` had grown again after M15-M17 and was becoming a new monolithic owner.
- App-facing default-app/capability-first demo checks are a coherent owner group and can evolve
  separately from broad source-tree, advanced roster, local-state bridge, and asset-helper checks.

## Behavior

No source-policy behavior changes are intended in this slice.

The new module owns the marker matrices and runner for:

- query/query-async,
- markdown,
- editor notes,
- todo,
- async playground,
- API workbench lite.

## Evidence

- `tools/examples_source_tree_policy/app_facing.py`
- `tools/examples_source_tree_policy/gate.py`

## Result

`tools/examples_source_tree_policy/gate.py` dropped from 1585 lines to 1285 lines; the new
`app_facing.py` module is 322 lines.

## Gates

```text
python tools/gate_examples_source_tree_policy.py
python -m py_compile tools/gate_examples_source_tree_policy.py tools/examples_source_tree_policy/__init__.py tools/examples_source_tree_policy/gate.py tools/examples_source_tree_policy/app_facing.py
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
