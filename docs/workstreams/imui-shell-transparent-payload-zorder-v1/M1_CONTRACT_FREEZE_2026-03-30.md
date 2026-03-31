# M1 Contract Freeze — 2026-03-30

This note freezes the first contract assumptions for
`docs/workstreams/imui-shell-transparent-payload-zorder-v1/`.

## Decision

The next shell-preview lane should focus on transparent moving-window overlap behavior, not on
reopening payload ghost visibility.

## Frozen decisions

### 1. Owner split

The intended owner split remains:

- runner/runtime diagnostics own `moving_window`, transparent payload application, hit-test
  passthrough, and under-window routing truth,
- `ecosystem/fret-docking` owns docking-specific preview continuity expectations under that truth,
- generic `fret-ui-kit::recipes` and `fret-ui-kit::imui` do not absorb overlap/z-order policy.

### 2. Proof surface

The first proof surface remains:

- `apps/fret-examples/src/docking_arbitration_demo.rs`

with these first script gates:

- `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-transparent-payload-zorder-switch.json`
- `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-large-transparent-payload-zorder-switch.json`

### 3. Minimum diagnostics package

The lane should start from existing diagnostics before inventing new fields:

- `transparent_payload_applied`
- `transparent_payload_hit_test_passthrough_applied`
- `moving_window`
- `window_under_moving_window`
- `window_under_moving_window_source`
- `current_window`
- `payload_ghost_visible`

Only if launched proof still cannot explain failures should this lane add new diagnostics fields.

### 4. Explicit non-goals

This lane does not own:

- generic cross-window preview ownership,
- same-window payload ghost behavior,
- native external drag images,
- aggregate multi-item previews.

## Immediate execution consequence

From this point forward:

1. treat payload ghost visibility as already closed by the predecessor lane,
2. use launched transparent-payload overlap scripts as the first proof artifact,
3. avoid widening generic recipe or `imui` contracts before launched evidence demands it.
