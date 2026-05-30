# Material 3 Canvas Draw Region Diagnostics v1

Status: Closed
Last updated: 2026-05-28

## Why This Lane Exists

The Material 3 component alignment sweep proved broad selector coverage, but ProgressIndicator and
Slider still draw important chrome inside `cx.canvas` as anonymous `SceneOp::Quad` entries. That
makes diagnostics good at finding the component root, but weak at naming the actual painted regions
such as progress track, active track, slider handle, state layer, ticks, and stop indicators.

This lane classifies that gap before changing architecture. The expected answer is split:

- exact named draw regions inside a canvas are a `fret-ui`/diagnostics mechanism gap because
  `SceneOp` carries no label or metadata and diagnostics bundles only expose scene counts,
  fingerprints, and paint hotspots;
- Material3 recipe code can still expose stable layout-only diagnostic anchors for a bounded set of
  painted parts where the geometry is already known at render time;
- scene/golden tests remain the correct gate for pixel-level or arc/segment output that cannot be
  represented as simple rectangular anchors.

## Target State

- ProgressIndicator and Slider have an explicit parity packet that separates recipe test gaps from
  a true canvas diagnostics mechanism gap.
- Material3 exposes stable, hidden diagnostic anchors only for rectangular painted regions whose
  layout can be represented without changing paint, hit-testing, focus, or accessibility outcomes.
- Circular progress arcs and animated indeterminate segments stay covered by scene/golden evidence
  until a generic named scene-op diagnostics contract exists.
- Any future `crates/*` mechanism work is recorded as a narrow follow-on with ADR/contract evidence,
  not slipped into Material recipe code.

## Source Precedence

- Material Design 3: visual intent for progress indicator tracks and slider track/handle/state
  layer/tick/stop indicator geometry.
- Compose Material3: toolkit-style semantics, slider state behavior, and touch target expectations.
- MUI Material UI / Material Web: web-facing slider/progress defaults and draw-part taxonomy.
- In-tree shadcn/Radix work: Fret-side `test_id` stamping and diagnostic gate conventions only.

## Layer Ownership

- `ecosystem/fret-ui-material3/src/progress_indicator.rs`: progress recipe geometry, hidden part
  anchors where rectangular and deterministic, and scene/golden proof.
- `ecosystem/fret-ui-material3/src/slider.rs`: slider/range slider recipe geometry, hidden part
  anchors for track/active/handle/state-layer/tick-marker surfaces where deterministic.
- `ecosystem/fret-ui-material3/src/foundation`: reusable Material test-id/diagnostic anchor helpers
  when multiple recipes share the same layout-only marker pattern.
- `ecosystem/fret-bootstrap` diagnostics: script-level proof and bounded bundles only.
- `crates/fret-ui` / `crates/fret-core`: out of scope for this lane unless the packet proves a
  generic, design-system-agnostic named scene-op diagnostics contract is necessary.

## In Scope

- Audit current ProgressIndicator/Slider canvas drawing and existing gates.
- Add a compact packet artifact naming foundation vs recipe vs diagnostics mechanism gaps.
- Add or refine recipe-level diagnostic anchors only where the anchor can match a rectangular
  painted region without changing user-facing behavior.
- Add focused automation-surface tests and, if useful, a UI Gallery diagnostics script for the
  anchored parts.
- Keep existing headless scene/golden gates green.

## Out Of Scope

- Adding labels/metadata to `SceneOp`.
- Replacing canvas rendering with many layout elements purely for testability.
- Full circular arc hit-testing or named per-segment diagnostics.
- Broad slider behavior parity not directly tied to draw-region observability.

## Closeout Condition

This lane can close when:

- the owner classification is recorded,
- recipe-level rectangular anchors are either implemented and gated or explicitly rejected with
  evidence,
- ProgressIndicator/Slider headless gates still pass,
- any precise named scene-op diagnostics work is split into a mechanism follow-on,
- and the broader Material3 goal has a clear next component packet.

Closeout result on 2026-05-28: closed. Linear progress and slider/range-slider rectangular
track/active-track/handle anchors are implemented and gated. Circular progress arcs, animated
segments, tick markers, stop indicators, and state-layer paint remain scene/golden evidence unless a
future mechanism lane adds generic named scene-op diagnostics.
