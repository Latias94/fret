# Fearless Architecture Convergence v1 - TODO

Status: Active
Last updated: 2026-05-25

## FAC-M0 - Scope And Contract Freeze

- [x] FAC-010 [owner=planner] [deps=none] [scope=docs/workstreams/fearless-architecture-convergence-v1,docs/adr]
  Goal: Freeze the six-cut owner map and identify which cuts need new follow-ons.
  Validation: `DESIGN.md`, `MILESTONES.md`, `EVIDENCE_AND_GATES.md`, and `WORKSTREAM.json` agree.
  Evidence: `docs/workstreams/fearless-architecture-convergence-v1/DESIGN.md`
  Handoff: Use this coordinator as the first-open map, not as the implementation folder for every cut.

## FAC-M1 - Retained Public Surface Exit

- [x] FAC-020 [owner=codex] [deps=FAC-010] [scope=crates/fret-ui,ecosystem/fret-node,docs/adr,docs/workstreams/retained-public-surface-exit-v1]
  Goal: Make retained widget authoring compat-only by default while keeping the legacy node retained canvas island compiling explicitly.
  Validation: `cargo test -p fret-ui retained_widget_authoring_exports_are_compat_feature_gated`; `cargo check -p fret-node --features compat-retained-canvas`
  Review: Confirm `Invalidation` and `CommandAvailability` remain public mechanism data types.
  Evidence: `docs/adr/0330-retained-runtime-internal-and-compat-surface.md`, `crates/fret-ui/src/lib.rs`, `ecosystem/fret-node/Cargo.toml`
  Handoff: Continue remaining adapter migration in the retained/node follow-on lanes.

## FAC-M2 - Split Follow-On Lanes

- [x] FAC-030 [owner=planner] [deps=FAC-020] [scope=docs/workstreams/fret-node-low-level-adapter-v1]
  Goal: Decide whether the node low-level adapter work continues in the existing active node lane or starts a narrower follow-on.
  Validation: Updated node workstream `WORKSTREAM.json` or new follow-on exists.
  Evidence: `docs/workstreams/fret-node-low-level-adapter-v1/WORKSTREAM.json`
  Handoff: First task should target one adapter seam, not the whole retained canvas deletion.

- [x] FAC-040 [owner=planner] [deps=FAC-020] [scope=docs/workstreams/fret-ui-kit-taxonomy-boundaries-v1]
  Goal: Open a kit taxonomy follow-on that names style/headless/primitives/declarative/imui/recipes owners and chooses one first owner move.
  Validation: New workstream `WORKSTREAM.json` and gate inventory.
  Evidence: `docs/workstreams/fret-ui-kit-taxonomy-boundaries-v1/WORKSTREAM.json`
  Handoff: Keep public API widening out unless the first slice proves it.

- [x] FAC-050 [owner=planner] [deps=FAC-020] [scope=docs/workstreams/ui-overlay-focus-dismissal-oracle-v1]
  Goal: Split overlay/focus/dismissal oracle work from the shipped runtime dispatch snapshot lane.
  Validation: Follow-on lane or updated active lane with oracle fixtures and commands.
  Evidence: `docs/workstreams/ui-overlay-focus-dismissal-oracle-v1/WORKSTREAM.json`
  Handoff: Policy oracle lives above runtime mechanisms.

- [x] FAC-060 [owner=planner] [deps=FAC-020] [scope=docs/workstreams/ui-frame-pipeline-v2-phase-contract-followon-v1]
  Goal: Start a narrow Frame Pipeline v2 follow-on for the next explicit phase-contract proof.
  Validation: Follow-on names one surface and one perf/correctness gate.
  Evidence: `docs/workstreams/ui-frame-pipeline-v2-phase-contract-followon-v1/WORKSTREAM.json`
  Handoff: Do not reopen the closed broad lane.

- [x] FAC-070 [owner=planner] [deps=FAC-020] [scope=docs/workstreams/fret-launch-root-surface-convergence-v1]
  Goal: Decide whether launch root-surface convergence is a maintenance update or a new follow-on.
  Validation: Gate inventory covers direct `WinitAppDriver` example regressions and `FnDriver` posture.
  Evidence: `docs/workstreams/fret-launch-root-surface-convergence-v1/WORKSTREAM.json`
  Handoff: Prefer gate/doc posture before widening launch APIs.

## FAC-M3 - Closeout

- [ ] FAC-080 [owner=planner] [deps=FAC-030,FAC-040,FAC-050,FAC-060,FAC-070] [scope=docs/workstreams/fearless-architecture-convergence-v1]
  Goal: Close this coordinator or leave it as maintenance once every cut has an owner lane.
  Validation: `python3 tools/check_workstream_catalog.py`; `python3 tools/check_layering.py`
  Evidence: `EVIDENCE_AND_GATES.md`, `HANDOFF.md`, `WORKSTREAM.json`
  Handoff: Remaining work must be in narrow owner lanes, not this coordinator.
