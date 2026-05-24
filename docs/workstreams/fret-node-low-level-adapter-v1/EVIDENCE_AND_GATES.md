# Fret Node Low-Level Adapter v1 - Evidence and Gates

Status: Active
Last updated: 2026-05-25

## Canonical Gates

- `cargo check -p fret-node`
- `cargo check -p fret-node --features compat-retained-canvas`
- `cargo test -p fret-node --features compat-retained-canvas retained_compatibility_surface_stays_declarative_only`
- `python3 tools/check_layering.py`

## Evidence Anchors

- `ecosystem/fret-node/Cargo.toml`
- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/canvas/widget.rs`
- `docs/workstreams/retained-public-surface-exit-v1/EVIDENCE_AND_GATES.md`
