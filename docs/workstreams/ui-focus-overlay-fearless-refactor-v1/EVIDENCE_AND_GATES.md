# Evidence and Gates

## Minimum gates (local)

- `cargo fmt`
- `cargo nextest run -p fret-ui`
- `python3 tools/check_layering.py`

## Existing regression coverage (anchors)

- Outside press routing: `crates/fret-ui/src/tree/tests/outside_press.rs`
- Escape dismissal: `crates/fret-ui/src/tree/tests/escape_dismiss.rs`
- Focus scope trap: `crates/fret-ui/src/tree/tests/focus_scope.rs`
- Declarative dismissible interactions: `crates/fret-ui/src/declarative/tests/interactions/dismissible.rs`

## New artifacts (Phase A/B)

- Unit test: stale parent pointers do not break outside-press branch exclusion.
- Unit tests: outside-press default focus clearing vs `prevent_default` suppression.

