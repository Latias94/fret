# Fearless Architecture Convergence v1 - TODO

Status: Closed
Last updated: 2026-07-02

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

- [x] FAC-080 [owner=planner] [deps=FAC-030,FAC-040,FAC-050,FAC-060,FAC-070] [scope=docs/workstreams/fearless-architecture-convergence-v1]
  Goal: Close this coordinator or leave it as maintenance once every cut has an owner lane.
  Validation: `python3 tools/check_workstream_catalog.py`; `python3 tools/check_layering.py`
  Evidence: `EVIDENCE_AND_GATES.md`, `HANDOFF.md`, `WORKSTREAM.json`
  Handoff: Closed by `CLOSEOUT_AUDIT_2026-07-02.md`; remaining work must be in narrow owner lanes,
  not this coordinator.

## FAC-M4 - 2026 UI Framework Convergence

- [x] FAC-090 [owner=codex] [deps=FAC-080] [scope=docs/golden-architecture.md,docs/runtime-contract-matrix.md,docs/ui-closure-map.md,docs/adr,docs/plans]
  Goal: Freeze the 2026 convergence owner map from the implementation-ready plan without reopening
  closed broad lanes.
  Validation: `python3 tools/check_layering.py`; `python3 tools/check_workstream_catalog.py`;
  `git diff --check`
  Evidence: `docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md`,
  `docs/golden-architecture.md`, `docs/runtime-contract-matrix.md`, `docs/ui-closure-map.md`,
  `docs/adr/IMPLEMENTATION_ALIGNMENT.md`
  Handoff: Execute the plan as narrow units. Start with source-policy gates, then identity/dirty
  graph metrics before deleting compatibility paths.

- [x] FAC-100 [owner=codex] [deps=FAC-090] [scope=tools,docs/dependency-policy.md]
  Goal: Add a source-policy gate that catches mechanism/policy drift and default app import leaks
  that dependency layering cannot see.
  Validation: `python3 tools/check_surface_policy.py`; focused unit tests for the checker; existing
  layering/profile gates.
  Evidence: `tools/check_surface_policy.py`, `tools/test_check_surface_policy.py`
  Handoff: Keep this gate heuristic and allowlist-backed in v1; do not scan the whole repo with
  blanket `fret_ui`/`fret_core` denies.

- [x] FAC-110 [owner=codex] [deps=FAC-090,FAC-100] [scope=docs/workstreams/fearless-architecture-convergence-v1]
  Goal: Close the 2026 implementation-ready convergence plan with evidence for U1-U9 and explicit
  retained/deferred follow-ons.
  Validation: `python3 tools/check_layering.py`; `python3 tools/check_surface_policy.py`;
  `python3 tools/check_consumption_profiles.py`; `python3 tools/check_workstream_catalog.py`;
  `git diff --check`
  Evidence: `docs/workstreams/fearless-architecture-convergence-v1/CLOSEOUT_AUDIT_2026-07-02.md`
  Handoff: Future work starts from the closeout audit's retained/deferred table.

## FAC-M5 - UI Framework Phase 2 Fearless Refactor

- [x] FAC-120 [owner=codex] [deps=FAC-110] [scope=docs/plans,docs/workstreams/fearless-architecture-convergence-v1,docs/knowledge/engineering]
  Goal: Close `docs/plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md` with evidence for
  U1-U14 and explicit retained bridges instead of reopening the broad coordinator.
  Validation: `cargo fmt --all --check`; `python3 tools/check_layering.py`;
  `python3 tools/check_surface_policy.py`; `python3 tools/check_consumption_profiles.py`;
  `python3 tools/check_execution_surface.py`; `python3 tools/check_workstream_catalog.py`;
  `python3 tools/check_adr_numbers.py`; `git diff --check`
  Evidence: `docs/plans/2026-07-02-001-refactor-ui-framework-phase2-closeout.md`
  Handoff: Future work starts from the closeout retained-bridge table. Do not treat Phase 2 as a
  claim that flat `Scene`, parent repair, full text shaping closure, or all non-quad partial upload
  work has been deleted.

## Deferred Follow-Ons

- Stable handle deletion after U4 observability.
- Entity-first `ViewId` ownership after the current v1 boundary-node bridge.
- Per-boundary ownership for currently window/layer-forest products only where cross-layer behavior
  can be preserved.
- Retained mechanism vocabulary audits for `Roving*` and explicit resizable module paths.
- Full second-hour starter expansion beyond the shipped `workbench-lite` scaffold.
- `workbench-lite` settings dialog / real async-mutation submit diagnostics.
- Advanced/manual allowlist cleanup as public wrappers replace proof surfaces.
- Renderer output migration beyond the flat `Scene` compatibility bridge.
- Non-quad resident partial uploads after side-table/material/text closure proofs.
- Full-blob text helper deletion after chunk-local text resource closure.
- Full aggregate pre-release runs when release scope needs them; duplicate ADR ID `0324` was
  resolved by renumbering the later a11y state-description ADR to `0332`.
- Phase 2 retained bridges: parent-pointer repair normal-path deletion, retained-tree GC liveness
  shrink, flat launch input deletion, full shaping-aware text chunk closure, per-stream non-quad
  partial upload gates, advanced/manual quarantine shrink, and historical observation-collapse perf
  key retirement.
