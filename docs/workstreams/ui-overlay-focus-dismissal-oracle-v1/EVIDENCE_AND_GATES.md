# UI Overlay Focus Dismissal Oracle v1 - Evidence and Gates

Status: Active
Last updated: 2026-05-25

## Canonical Gates

- `cargo test -p fret-ui outside_press`
- `cargo test -p fret-ui focus_scope`
- `cargo test -p fret-ui dismissible`
- `python3 tools/check_layering.py`

## Evidence Anchors

- `docs/workstreams/ui-focus-overlay-fearless-refactor-v1/EVIDENCE_AND_GATES.md`
- `crates/fret-ui/src/tree/tests/outside_press.rs`
- `crates/fret-ui/src/declarative/tests/interactions/dismissible.rs`
- `tools/diag-scripts/ui-gallery/overlay/`
