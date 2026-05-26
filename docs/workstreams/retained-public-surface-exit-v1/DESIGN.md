# Retained Public Surface Exit v1

Status: Active
Last updated: 2026-05-25

## Why This Lane Exists

`retained-bridge-exit-v1` deleted the old `fret-ui` retained bridge and compatibility facade, but
it intentionally left `Widget` and retained context types available from the stable `fret-ui` root
so the remaining `fret-node/compat-retained-canvas` island could keep compiling. That was a good
bridge-exit step, but it left the wrong long-term signal: retained widget authoring still looked
like the default public authoring model.

This follow-on changes the public-surface stance without deleting the retained runtime. Retained
runtime mechanisms stay inside `fret-ui`; retained widget authoring becomes explicit
compatibility-only API.

## Relevant Authority

- ADRs:
  - `docs/adr/0330-retained-runtime-internal-and-compat-surface.md`
  - `docs/adr/0066-fret-ui-runtime-contract-surface.md`
  - `docs/adr/0028-declarative-elements-and-element-state.md`
  - `docs/adr/0039-component-authoring-model-render-renderonce-and-intoelement.md`
- Existing docs:
  - `docs/shadcn-declarative-progress.md`
  - `docs/architecture.md`
- Related workstreams:
  - `docs/workstreams/retained-bridge-exit-v1/`
  - `docs/workstreams/fret-node-declarative-fearless-refactor-v1/`
  - `docs/workstreams/fearless-architecture-convergence-v1/`

## Problem

Default root exports in `crates/fret-ui` still included:

- `Widget`
- `EventCx`
- `CommandCx`
- `CommandAvailabilityCx`
- `LayoutCx`
- `PrepaintCx`
- `PaintCx`
- `SemanticsCx`

Those names are retained widget authoring hooks. Keeping them in the default root surface conflicts
with the declarative-only component ecosystem boundary and makes it too easy for new ecosystem code
to depend on retained authoring by accident.

At the same time, `Invalidation` and `CommandAvailability` are still shared mechanism data types.
Moving them in the same slice would create churn without proving the retained-authoring boundary.

## Target State

- Default `fret-ui` root keeps `Invalidation` and `CommandAvailability`.
- Default `fret-ui` root does not expose retained widget authoring types.
- `fret-ui/compat-retained-widgets` exposes the retained widget authoring types for explicit
  compatibility islands.
- `fret-node/compat-retained-canvas` enables `fret-ui/compat-retained-widgets` explicitly.
- Source-policy tests lock the boundary.

## In Scope

- `crates/fret-ui/src/lib.rs` export gating.
- `ecosystem/fret-node/Cargo.toml` feature mapping.
- `ecosystem/fret-node/src/lib.rs` policy wording/gates.
- ADR 0330 and alignment/index updates.

## Out Of Scope

- Deleting `UiTree`.
- Deleting `Widget` internally.
- Rewriting the full node graph retained canvas/editor implementation.
- Moving `Invalidation` and `CommandAvailability` to new modules in this slice.
- Introducing a final `CanvasElementAdapter` API in this slice.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| `Widget/*Cx` are retained authoring hooks and should not be default root exports. | Confident | `crates/fret-ui/src/widget.rs`, `docs/adr/0066-fret-ui-runtime-contract-surface.md` | If wrong, ecosystem components would need retained authoring as a stable API and ADR 0330 must be revised. |
| `Invalidation` and `CommandAvailability` are shared mechanism data types and should remain public for now. | Likely | Uses in declarative/binding and command availability surfaces; definitions currently live in `widget.rs`. | If wrong, a later neutral-module move should replace their public path with a migration note. |
| `fret-node/compat-retained-canvas` is the only current external retained authoring consumer that needs the gated exports. | Likely | `rg` usage scan points at `ecosystem/fret-node/src/ui/canvas/widget/**`. | If wrong, `cargo check --workspace` or focused package checks will reveal another explicit compat island. |
| Keeping root paths behind a feature is an acceptable first step before a dedicated compat namespace. | Likely | `retained-bridge-exit-v1` already removed the old facade; this slice minimizes churn while restoring explicit opt-in. | If wrong, next slice should introduce `fret_ui::compat_retained_widgets::*` and migrate node imports. |

## Architecture Direction

The split is:

- Runtime mechanism: retained `UiTree` and `widget.rs` remain inside `fret-ui`.
- Default authoring: declarative element tree and ecosystem policy crates.
- Compatibility island: feature-gated retained widget authoring exports, only for named adapters or
  migration code.

This lets the project answer the retained question precisely: retain retained internally; remove it
from the default public authoring contract.

## Closeout Condition

This lane can close when:

- ADR 0330 is accepted and tracked in the alignment matrix;
- focused `fret-ui` and `fret-node` gates pass;
- docs and source-policy tests no longer call retained root exports stable/default authoring API;
- follow-on adapter work is assigned to the node lane or a new narrow follow-on.
