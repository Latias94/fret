# Retained Public Surface Exit v1 - Handoff

Updated: 2026-05-25

## Current State

The first implementation slice has landed and has a named follow-on owner:

- ADR 0330 defines retained runtime as internal/compat-only authoring surface.
- `crates/fret-ui/src/lib.rs` gates retained widget authoring exports behind
  `compat-retained-widgets`.
- `ecosystem/fret-node/compat-retained-canvas` explicitly enables that feature.
- Node adapter migration continues in `docs/workstreams/fret-node-low-level-adapter-v1/`.

## Recorded Gates

Recorded in `EVIDENCE_AND_GATES.md`:

- `cargo test -p fret-ui retained_widget_authoring_exports_are_compat_feature_gated`
- `cargo check -p fret-ui`
- `cargo check -p fret-node --features compat-retained-canvas`
- `python3 tools/check_layering.py`
- `python3 tools/check_workstream_catalog.py`
- `git diff --check`

## Follow-On

The next code move belongs in `docs/workstreams/fret-node-low-level-adapter-v1/`, likely `NLA-010`.
It should introduce a clearer low-level adapter or delete another retained canvas edge; it should
not reopen this public-surface lane as a broad node graph rewrite.
