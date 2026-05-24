# Retained Public Surface Exit v1 - TODO

Status: Active
Last updated: 2026-05-25

## RPS-M0 - Contract Freeze

- [x] RPS-010 [owner=codex] [deps=none] [scope=docs/adr,docs/workstreams/retained-public-surface-exit-v1]
  Goal: Add ADR 0330 and freeze the retained runtime vs retained authoring boundary.
  Validation: ADR exists, ADR index/alignment include it, and workstream docs name gates.
  Evidence: `docs/adr/0330-retained-runtime-internal-and-compat-surface.md`
  Handoff: Implementation must match the ADR wording: retained runtime stays; retained authoring is compat-only.

## RPS-M1 - Feature-Gate Retained Authoring Root Exports

- [x] RPS-020 [owner=codex] [deps=RPS-010] [scope=crates/fret-ui/src/lib.rs]
  Goal: Keep `Invalidation` and `CommandAvailability` public while gating `Widget/*Cx` behind `compat-retained-widgets`.
  Validation: `cargo test -p fret-ui retained_widget_authoring_exports_are_compat_feature_gated`; `cargo check -p fret-ui`
  Evidence: `crates/fret-ui/src/lib.rs`
  Handoff: If another crate needs retained authoring, it must opt into the compatibility feature explicitly.

## RPS-M2 - Node Compatibility Island Opt-In

- [x] RPS-030 [owner=codex] [deps=RPS-020] [scope=ecosystem/fret-node]
  Goal: Make `fret-node/compat-retained-canvas` enable `fret-ui/compat-retained-widgets` explicitly and update policy wording.
  Validation: `cargo check -p fret-node --features compat-retained-canvas`
  Evidence: `ecosystem/fret-node/Cargo.toml`, `ecosystem/fret-node/src/lib.rs`
  Handoff: The next node slice should replace root retained imports with a named low-level adapter or delete more retained canvas code.

## RPS-M3 - Follow-On Adapter Decision

- [x] RPS-040 [owner=planner] [deps=RPS-030] [scope=docs/workstreams/fret-node-low-level-adapter-v1]
  Goal: Decide whether to continue the active node declarative lane or open a narrower `fret-node` adapter follow-on.
  Validation: Active/next lane has `WORKSTREAM.json` and first task.
  Evidence: `docs/workstreams/fret-node-low-level-adapter-v1/WORKSTREAM.json`
  Handoff: Do not expand this retained public-surface lane into the full node canvas rewrite.

## RPS-M4 - Closeout

- [ ] RPS-050 [owner=planner] [deps=RPS-040] [scope=docs/workstreams/retained-public-surface-exit-v1]
  Goal: Close or mark maintenance after gates are recorded and adapter follow-on is assigned.
  Validation: `python3 tools/check_workstream_catalog.py`; `git diff --check`
  Evidence: `EVIDENCE_AND_GATES.md`, `HANDOFF.md`, `WORKSTREAM.json`
  Handoff: Remaining work belongs in the node adapter lane.
