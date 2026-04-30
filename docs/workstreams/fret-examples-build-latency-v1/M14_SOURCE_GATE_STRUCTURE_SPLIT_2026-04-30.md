# Fret Examples Build Latency v1 - M14 Source Gate Structure Split - 2026-04-30

Status: complete

## Decision

Keep `python tools/gate_examples_source_tree_policy.py` as the stable command entrypoint, but move
the large implementation into `tools/examples_source_tree_policy/gate.py`.

## Rationale

- The examples source-tree gate now owns many migrated source-only checks.
- Continuing to add marker matrices to one top-level script would make future slices harder to
  review.
- A package-backed implementation keeps the public command stable while giving future refactors a
  natural place to split owner-specific modules.

## Behavior

No source-policy behavior changes are intended in this slice.

## Gates

```text
python tools/gate_examples_source_tree_policy.py
python -m py_compile tools/gate_examples_source_tree_policy.py tools/examples_source_tree_policy/__init__.py tools/examples_source_tree_policy/gate.py
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
