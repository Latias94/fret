# Fearless Architecture Convergence v1

Status: Active
Last updated: 2026-05-25

## Why This Lane Exists

The six architecture cuts are related, but they are not one implementation problem. Treating them
as one giant refactor would blur ownership between `crates/fret-ui`, `ecosystem/*`, `fret-node`,
Frame Pipeline v2, and launch/facade posture. This lane is the coordinator: it records the target
shape, assigns each cut to the right owner lane, and keeps the first executable slice small enough
to prove.

## Relevant Authority

- ADRs:
  - `docs/adr/0066-fret-ui-runtime-contract-surface.md`
  - `docs/adr/0330-retained-runtime-internal-and-compat-surface.md`
  - `docs/adr/0327-frame-pipeline-v2-and-view-boundaries.md`
  - `docs/adr/0328-product-language-and-ecosystem-positioning.md`
- Existing docs:
  - `docs/architecture.md`
  - `docs/shadcn-declarative-progress.md`
  - `docs/repo-structure.md`
- Related workstreams:
  - `docs/workstreams/retained-public-surface-exit-v1/`
  - `docs/workstreams/fret-node-declarative-fearless-refactor-v1/`
  - `docs/workstreams/ui-focus-overlay-fearless-refactor-v1/`
  - `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/`
  - `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/`
  - `docs/workstreams/imui-kit-owner-split-v1/`

## Problem

The current codebase has enough retained substrate, declarative authoring, IMUI helpers, overlay
policy, node-graph compatibility code, frame-pipeline work, and launch facade work to build real
surfaces. The architecture risk is not missing effort. The risk is that old public surfaces keep
teaching the wrong owner boundary:

- retained `Widget/*Cx` looks like the public authoring model;
- `fret-node` still needs a retained island but should become a low-level canvas/viewport adapter;
- `fret-ui-kit` contains several owner categories that should be named and kept separate;
- overlay/focus/dismissal semantics are implemented but lack a shared headless policy oracle;
- Frame Pipeline v2 is accepted but follow-on work must not regress into helper-only paths;
- launch examples are mostly fixed, but root-surface posture still needs a maintenance gate.

## Target State

When this convergence lane closes:

- retained runtime is internal/compat-only by default, with ADR 0330 and gates proving the boundary;
- node graph low-level retained canvas work is behind a named adapter/compat lane, not the ecosystem
  authoring story;
- `fret-ui-kit` owner taxonomy is explicit enough that style, headless engines, primitives,
  declarative adapters, IMUI helpers, and recipes do not accidentally trade responsibilities;
- overlay/focus/dismissal behavior has a reusable oracle surface that policy crates can test
  against;
- Frame Pipeline v2 follow-ons target named build/layout/prepaint/paint/commit phase contracts;
- launch/facade posture keeps `FnDriver` as the preferred advanced route and treats direct
  `WinitAppDriver` examples as regressions unless explicitly justified.

## In Scope

- Workstream split and sequencing for the six cuts.
- ADR creation or updates when a public or hard-to-change contract changes.
- First implementation slice: retained public surface exit.
- Gate inventory for the next five cuts.

## Out Of Scope

- Rewriting all retained runtime internals.
- Completing the full declarative node graph adapter in this coordinator lane.
- Moving Radix/shadcn/Base UI policy into `crates/fret-ui`.
- Reopening closed broad lanes such as Frame Pipeline v2 closeout or IMUI owner split closeout.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Retained runtime should stay, but retained widget authoring should not be the default public API. | Confident | `docs/adr/0066-fret-ui-runtime-contract-surface.md`, `crates/fret-ui/src/lib.rs`, `docs/adr/0330-retained-runtime-internal-and-compat-surface.md` | If wrong, ADR 0330 would over-constrain low-level authors and must be revised before further deletion. |
| `fret-node/compat-retained-canvas` is the right first compatibility island to prove explicit opt-in. | Confident | `ecosystem/fret-node/Cargo.toml`, `ecosystem/fret-node/src/ui/canvas/widget.rs`, `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md` | If wrong, retained exports may need a different adapter crate before gating root exports. |
| Closed lanes should stay closed and new work should start as narrow follow-ons. | Confident | `docs/workstreams/standalone/workstream-state-v1.md`, `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/WORKSTREAM.json` | If wrong, evidence would be fragmented across old closeout docs and current implementation docs. |
| Overlay/focus/dismissal needs a policy oracle outside the runtime mechanism layer. | Likely | `docs/workstreams/ui-focus-overlay-fearless-refactor-v1/README.md`, Radix/Base UI reference stack in `docs/reference-stack-ui-behavior.md` | If wrong, policy may keep drifting across recipes and diagnostics scripts. |
| Launch posture is mostly a maintenance-gate problem after the previous lane. | Likely | `docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/FINAL_STATUS.md` | If wrong, a follow-on must audit root exports and examples before code changes. |

## Architecture Direction

Use a coordinator-plus-follow-ons model:

- this lane owns the six-cut map and ordering;
- `retained-public-surface-exit-v1` owns the first hard contract and code slice;
- active existing lanes continue only when their `WORKSTREAM.json` says they are active or
  maintenance;
- closed lanes provide evidence but are not reopened unless a new explicit state change says so.

## Six-Cut Owner Map

| Cut | Owner lane | First proof |
| --- | --- | --- |
| Retained public surface exit | `retained-public-surface-exit-v1` | `fret-ui` gates `Widget/*Cx` behind `compat-retained-widgets`; `fret-node` opts in explicitly. |
| `fret-node` low-level adapter | Existing `fret-node-declarative-fearless-refactor-v1`, likely follow-on | Replace retained canvas/editor island with a named canvas/viewport adapter surface. |
| `fret-ui-kit` taxonomy deepening | New follow-on after current IMUI closeouts | Source audit names style/headless/primitives/declarative/imui/recipes owners and moves one confused owner. |
| Overlay/focus/dismissal oracle | Follow-on from `ui-focus-overlay-fearless-refactor-v1` | Headless oracle fixtures for outside press, focus restore, modal barrier, nested scopes. |
| Frame Pipeline v2 follow-on | Narrow follow-on from closed Frame Pipeline v2 lane | One additional surface proves explicit phase contracts without helper-only shortcuts. |
| Launch root surface convergence | Follow-on from `fret-launch-app-surface-fearless-refactor-v1` | Root export/example gate keeps `FnDriver` preferred and direct `WinitAppDriver` exceptional. |

## Closeout Condition

This lane can close when each cut has either:

- a completed implementation lane with gates and evidence, or
- a narrow active follow-on with a clear first task, owner, and validation command.
