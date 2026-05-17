# shadcn Menu/Select Policy Follow-on v1 — Evidence And Gates

Status: Active
Last updated: 2026-05-17

## Baseline Observation

Observed during ASF-060:

```bash
cargo test -p fret-ui-shadcn --locked --test select_keyboard_navigation -j 1
```

Result:

- Failed after running the test.
- Failure: pointer-open + ArrowDown + Enter selected `"apple"`; the test expected `"banana"`.
- Interpretation: this is an unresolved select interaction contract, not a layering failure.

## Gate Set

```bash
cargo test -p fret-ui-headless --locked --lib entry_focus -j 1
cargo test -p fret-ui-kit --locked --lib initial_focus -j 1
cargo test -p fret-ui-shadcn --locked --test select_keyboard_navigation -j 1
cargo test -p fret-ui-shadcn --locked --test dropdown_menu_keyboard_navigation -j 1
cargo test -p fret-ui-shadcn --locked --test context_menu_keyboard_navigation -j 1
python tools/check_layering.py
```

Use narrower gates while iterating, but record exact commands next to each shipped slice.

## Evidence Anchors

- `docs/workstreams/architecture-surface-fearless-refactor-v1/EVIDENCE_AND_GATES.md`
- `ecosystem/fret-ui-headless/src/entry_focus.rs`
- `ecosystem/fret-ui-kit/src/primitives/menu/root.rs`
- `ecosystem/fret-ui-kit/src/primitives/select.rs`
- `ecosystem/fret-ui-shadcn/src/select.rs`
- `ecosystem/fret-ui-shadcn/tests/select_keyboard_navigation.rs`
