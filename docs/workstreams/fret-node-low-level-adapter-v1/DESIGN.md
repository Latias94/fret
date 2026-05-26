# Fret Node Low-Level Adapter v1

Status: Active
Last updated: 2026-05-25

## Why This Lane Exists

`fret-node` is already declarative-first at its public surface, but the remaining retained
canvas/editor island still uses `Widget`, `EventCx`, `LayoutCx`, and `PaintCx` through
`compat-retained-canvas`. ADR 0330 makes that explicit compatibility API, not default authoring
surface. This lane owns the next step: replacing the compatibility island with a named low-level
adapter suitable for canvas/editor-grade interaction.

## Relevant Authority

- `docs/adr/0330-retained-runtime-internal-and-compat-surface.md`
- `docs/adr/0128-canvas-widgets-and-interactive-surfaces.md`
- `docs/workstreams/fret-node-declarative-fearless-refactor-v1/`
- `docs/workstreams/retained-public-surface-exit-v1/`
- `ecosystem/fret-node/src/ui/canvas/widget.rs`

## Target State

- Default `fret-node` authoring remains declarative.
- Low-level canvas/editor integration has a named adapter contract that can own pointer, command,
  focus, layout/prepaint/paint, diagnostics, and viewport transforms without exposing retained
  widget authoring.
- `compat-retained-canvas` becomes a delete-planned oracle or disappears after the adapter proves
  the behavior families.

## Non-goals

- Rewriting graph data models or node styling.
- Moving canvas-specific policy into `crates/fret-ui`.
- Deleting the retained canvas island before an equivalent adapter proof exists.

## First Slice

Audit the retained island and choose one adapter seam. Good candidates:

- paint/prepaint adapter for scene-fragment emission;
- event adapter for pointer routing;
- command adapter for edit/selection commands;
- diagnostics adapter for internals snapshots.

The first slice should move one family behind an adapter trait or function set and keep existing
compat tests green.
