# Fret Node Paint Root Cached Edge Key Helper v1 - TODO

Status: Closed
Last updated: 2026-05-25

## CEKH-M0 - Scope Freeze

- [x] CEKH-010 [owner=codex] [deps=none] [scope=docs/workstreams/fret-node-paint-root-cached-edge-key-helper-v1]
  Goal: Open a narrow follow-on for cached edge key-field helper ownership.
  Validation: `python3 -m json.tool docs/workstreams/fret-node-paint-root-cached-edge-key-helper-v1/WORKSTREAM.json`
  Evidence: `DESIGN.md`, `EVIDENCE_AND_GATES.md`
  Handoff: Keep cache invalidation, cache lifetime, scope string changes, and route adapters out of
  scope.

## CEKH-M1 - Cached Edge Key Helper

- [x] CEKH-020 [owner=codex] [deps=CEKH-010] [scope=ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_edges/keys.rs]
  Goal: Move shared cached edge key-field writes behind one helper without changing key semantics.
  Validation: `cargo test -p fret-node --features compat-retained-canvas paint_root_cached_edge_key_helper`
  Evidence: `keys.rs`, `ecosystem/fret-node/src/lib.rs`
  Handoff: Key functions preserve names, inputs, scope strings, and rect-origin behavior.

## CEKH-M2 - Closeout

- [x] CEKH-030 [owner=codex] [deps=CEKH-020] [scope=docs/workstreams/fret-node-paint-root-cached-edge-key-helper-v1]
  Goal: Close the lane and keep invalidation/lifetime/key semantic changes separate.
  Validation: `python3 tools/check_workstream_catalog.py`; `git diff --check`
  Evidence: `CLOSEOUT_AUDIT_2026-05-25.md`
  Handoff: Start a separate follow-on for key input API changes or cache invalidation work.
