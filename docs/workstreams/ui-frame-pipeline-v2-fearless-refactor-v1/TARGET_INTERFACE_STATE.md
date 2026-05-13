# Target Interface State

Status: Draft target
Last updated: 2026-05-13

This file describes the target shape. ADR 0327 is accepted as the contract freeze; internal and
public names may still evolve through follow-on ADR/workstream notes while preserving the accepted
phase and boundary outcomes.

## Runtime Pipeline

Target phases:

```text
schedule / dirty propagation
build
request_layout
layout
prepaint
paint
renderer_prepare_encode_upload_present
```

The important contract is not the exact function names. The important contract is that each phase
has one owner, one output shape, and diagnostics that can explain reuse or invalidation.

## Boundary State

The target runtime boundary should contain state equivalent to:

```rust,ignore
struct ViewBoundaryState {
    id: BoundaryId,
    parent: Option<BoundaryId>,
    identity: BoundaryIdentity,
    dirty: BoundaryDirtyFlags,
    dependencies: BoundaryDependencyKeys,
    layout: BoundaryLayoutState,
    prepaint: BoundaryPrepaintState,
    paint: BoundaryPaintState,
    diagnostics: BoundaryDiagnostics,
}
```

The public or internal names may change. The required capabilities should not.

## Dirty Flags

Minimum dirty classes:

- `build`
- `layout`
- `prepaint`
- `paint`
- `hit_test`
- `semantics`
- `renderer_resources`

Dirty propagation must be boundary-aware. A child paint invalidation should not become ancestor
layout invalidation unless a declared dependency requires it.

## Dependency Keys

Minimum dependency keys:

- parent available width / height,
- scale factor,
- theme/style revision where relevant,
- text/font revision where relevant,
- scroll position or windowed range,
- model/global observation revision,
- interaction state relevant to layout/paint.

Keys should be explicit enough that diagnostics can answer:

- reused,
- rejected because key changed,
- rejected because dirty flag was set,
- rejected because inspection/picking disables reuse,
- rejected because an escape hatch requested full rebuild.

## Prepaint State

Prepaint should own geometry-derived state:

- visible window / row range,
- editor frame state,
- scroll extents,
- overlay anchors,
- hitbox inputs,
- resource touch plans,
- scene-fragment replay plans.

Paint should consume this state rather than recomputing it.

## Scene Fragment

Boundary-owned paint reuse should converge on:

```rust,ignore
struct SceneFragment {
    ops: SceneRecording,
    text_blob_ids: Vec<TextBlobId>,
    resource_ids: BoundaryResourceIds,
    fingerprint: SceneFingerprint,
    local_bounds: Rect,
}
```

Replay may translate or transform when dependency keys allow it.

## Migration-Only Surfaces

The following current concepts should be treated as candidates for migration, consolidation, or
deletion:

- `ViewCacheProps::contained_layout` as a standalone knob,
- separate view-cache root and paint-cache root bookkeeping when a boundary can own both,
- env-only perf knobs that become unnecessary once boundary diagnostics are explicit,
- code-editor-local prepaint-like staging that can move to the shared prepaint phase,
- the transitional node-scoped `PrepaintOutputs` carrier currently used for code-editor replay
  plans,
- duplicate cache rejection counters that can become boundary diagnostics.

## Public Surface Stance

The v2 boundary model should be primarily internal to `crates/fret-ui`.

Public app authors should see simpler concepts:

- view/root boundaries through the app-facing `fret` lane,
- optional component-level hints in ecosystem builders,
- diagnostics and perf evidence through `fretboard`.

They should not be forced to manually manage phase internals.
