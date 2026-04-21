# ImUi Facade Internal Modularization v1 - M2 Interaction Runtime Slice (2026-04-21)

## Decision

M2 keeps `ecosystem/fret-ui-kit/src/imui/interaction_runtime.rs` as the stable outward hub while
moving its implementation into five private owner modules.

This slice remains structural only:

- no outward helper names changed,
- no model payload shapes changed,
- and no interaction semantics are intentionally widened here.

## What changed

`interaction_runtime.rs` now re-exports the same helper family over five private owner files:

- `ecosystem/fret-ui-kit/src/imui/interaction_runtime/models.rs`
- `ecosystem/fret-ui-kit/src/imui/interaction_runtime/disabled.rs`
- `ecosystem/fret-ui-kit/src/imui/interaction_runtime/lifecycle.rs`
- `ecosystem/fret-ui-kit/src/imui/interaction_runtime/hover.rs`
- `ecosystem/fret-ui-kit/src/imui/interaction_runtime/drag.rs`

Owner split:

- `models.rs` owns the global stores and model lookup helpers.
- `disabled.rs` owns disabled-scope state plus response sanitization.
- `lifecycle.rs` owns activation/deactivation/edit bookkeeping.
- `hover.rs` owns hover-delay timers, shared hover delay state, and long-press timer emission.
- `drag.rs` owns drag thresholds, active-item transitions, and pointer-region / pressable drag
  finishing.

## Why this closes M2

The old flat `interaction_runtime.rs` mixed stores, disabled policy, lifecycle bookkeeping,
hover-delay timers, and drag state transitions in one file.

After M2:

- hover/lifecycle/drag/disabled bookkeeping are reviewable as separate owners,
- the root runtime helper file keeps the existing outward import surface,
- and the remaining large `imui.rs` hub is now the next structural target instead of another
  rewrite of `interaction_runtime.rs`.

## Deferred to M3

- `ecosystem/fret-ui-kit/src/imui.rs`

M3 still owns the root facade hub split after the current `options`, `response`, and
`interaction_runtime` owner decompositions have landed.

## Evidence anchors

- `ecosystem/fret-ui-kit/src/imui/interaction_runtime.rs`
- `ecosystem/fret-ui-kit/src/imui/interaction_runtime/models.rs`
- `ecosystem/fret-ui-kit/src/imui/interaction_runtime/disabled.rs`
- `ecosystem/fret-ui-kit/src/imui/interaction_runtime/lifecycle.rs`
- `ecosystem/fret-ui-kit/src/imui/interaction_runtime/hover.rs`
- `ecosystem/fret-ui-kit/src/imui/interaction_runtime/drag.rs`
- `docs/workstreams/imui-facade-internal-modularization-v1/EVIDENCE_AND_GATES.md`
