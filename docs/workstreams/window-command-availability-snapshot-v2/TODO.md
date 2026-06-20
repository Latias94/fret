# TODO

Status: Active
Last updated: 2026-06-20

## Current Slice

- [x] Confirm pointer movement no longer synchronously publishes full command availability.
- [x] Add route-aware command availability debug hotspots in `fret-ui`.
- [x] Project command availability hotspots through `fret-bootstrap` debug snapshots.
- [x] Parse and report command availability hotspots in `fret-diag stats`.
- [x] Add focused stats JSON coverage for command availability hotspot attribution.
- [x] Sync the perf key registry doc required by the current diag profiling infra gate.
- [x] Re-run changed-package tests after the workstream docs are added.
- [x] Re-run the overlay pointer-move perf gate and record the latest bundle.
- [x] Run `diag stats` on the latest bundle and confirm hotspot output/JSON shape.
- [x] Cache/coalesce `focus_traversal_snapshot` availability within the same frame.
- [x] Add direct `fret-ui` coverage proving same-frame focus traversal cache reuse and next-frame
  recompute.
- [x] Re-run the overlay pointer-move perf gate after the cache and confirm the latest bundle has
  no frame with more than one `focus_traversal_snapshot` hotspot.

## Follow-Up Slices

- [x] Add a direct `fret-ui` unit test that proves `PointerEvent::Move` publishes only input-context
  state and not widget command availability.
- [ ] Promote a first-party check that fails when command availability publication exceeds 500us and
  no hotspots are present.
- [ ] Consider a dedicated `diag stats` gate flag for command availability attribution presence.
- [x] Audit whether `command_availability_in_action_route_fallback_roots` should avoid resolving
  fallback roots twice when diagnostics are enabled. The timed route now resolves fallback roots once and reuses the
  first resolved root as the debug hotspot start node; focused coverage:
  `action_availability_snapshot_uses_explicit_action_route_fallback_root`.
- [x] Audit repeated `focus_traversal_snapshot` publication within one frame; the latest evidence
  shows the redundant repeated 99-105us samples are removed by the frame-level cache.
- [x] Skip focus-bound `text.*` / `edit.*` commands in no-focus subtree fallback after explicit
  action-route fallback roots fail; focused coverage:
  `action_availability_no_focus_subtree_fallback_skips_focus_bound_edit_commands`.
- [x] Cache subtree-interest summaries for no-focus subtree fallback within one publication so
  repeated widget commands can reuse subtree pruning metadata; focused coverage:
  `action_availability_no_focus_subtree_fallback_reuses_subtree_interest_across_commands`.

## Exit Criteria

- The changed-package test set passes.
- The overlay pointer-move perf gate passes with worst dispatch below 1.5ms.
- `diag stats --json` exposes `top[].command_availability_hotspots[]` with command, route,
  elapsed time, node IDs, element IDs, and debug paths.
- Same-frame repeated command availability publication does not record more than one
  `focus_traversal_snapshot` hotspot for the same frame/snapshot state.
- Workstream gates and evidence paths are current.
