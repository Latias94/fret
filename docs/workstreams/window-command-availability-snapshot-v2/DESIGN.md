# Window Command Availability Snapshot v2

Status: Active
Last updated: 2026-06-20

## Problem

Window runtime command/action availability publication must be bounded, observable, and
frame-scoped. It is a runtime contract because menus, command palettes, shortcut overlays, and
platform integration layers consume the same per-window availability snapshot.

The failure mode this lane prevents is coupling pointer movement to a full widget command
availability recomputation. Overlay-heavy UI can move the pointer frequently; those frames must
remain dispatch-bound by pointer routing and hit testing, not by command publication.

## Scope

Owned here:

- `crates/fret-ui` command availability traversal and window runtime snapshot publication.
- Frame-local debug records for command availability routes and command IDs.
- `fret-bootstrap` debug snapshot projection for command availability hotspots.
- `fret-diag stats` parsing/reporting for the hotspot payload.
- The overlay pointer-move perf gate that proves pointer movement stays below the initial dispatch
  threshold.

Not owned here:

- Component policy for whether an action should be enabled. That remains in ecosystem layers.
- Generic perf registry, diff, trace, and DevTools reporting infrastructure. Those stay in
  `diag-perf-attribution-v1` and `diag-perf-profiling-infra-v1`.
- Broad UI smoothness baselines. Those stay in `ui-perf-*` lanes.

## Target Contract

- Pointer-move dispatch publishes the input-context snapshot only; it must not synchronously publish
  the full widget command availability snapshot.
- Full command availability publication remains frame-scoped and uses the focused/default route
  first, then explicit fallback routes when needed.
- Every command availability route records a bounded frame-local debug hotspot when diagnostics are
  enabled.
- Any frame whose command availability publication exceeds 500us must have per-command/per-route
  attribution available in `debug.command_availability_hotspots` and surfaced by `diag stats`.
- The initial overlay pointer-move gate keeps worst pointer dispatch under 1.5ms and worst pointer
  hit test under 500us.

## Route Vocabulary

The current runtime routes are intentionally explicit:

- `focused_or_default`
- `default_root_fallback`
- `focus_traversal_snapshot`
- `action_route_fallback_roots`
- `subtree_no_focus_fallback`

These route names are diagnostics contract data. Rename only with a migration/audit note.

## Design Notes

The core traversal still returns `CommandAvailability`, but now also returns the resolved node when
one handled the command. Timed wrappers record:

- command ID,
- route,
- start node/element,
- resolved node/element,
- outcome,
- elapsed time.

Records are cleared at frame reset, sorted by elapsed time, and capped. This keeps the payload
small while preserving enough evidence to explain slow command availability frames.

Runtime recording stays low-overhead: `fret-ui` records command/route/node/element IDs in the hot
publication path, while `fret-bootstrap` resolves diagnostic debug paths during snapshot projection.
This avoids making command availability timing measure the diagnostic string construction.

The attribution threshold is deliberately lower than the public perf gate: 500us is the point where
a frame should be explainable even if it is not yet user-visible jank.

## Focus Traversal Availability Cache

`focus.next` and `focus.previous` share the same availability predicate: whether the current
focus-routing snapshot has at least one traversable focus target. Before this slice, the publication
path kept that result in a local `Option`, so repeated publication in the same frame recomputed the
same traversal.

The result now lives in a `UiTree` frame-level cache. The cache key includes:

- frame id,
- dispatch snapshot generation,
- window,
- active layer roots,
- barrier root,
- requested and resolved scope root,
- command availability revision,
- layout readiness for the resolved scope root,
- inspection mode.

This keeps the cache in the mechanism layer: it only coalesces a pure traversal availability query
for the same authoritative frame/snapshot state. It does not change component policy or command
enablement rules. When the dispatch snapshot or command availability revision changes, the cache is
cleared; when layout readiness changes after final layout, the key changes and the query recomputes.

Diagnostics attribution is intentionally recorded only on cache miss. A same-frame cache hit should
not create another `focus_traversal_snapshot` hotspot, because no traversal work happened.

## Action Route Fallback Root Timing

The timed `action_route_fallback_roots` route must not do extra fallback-root discovery just to
label diagnostics. The route resolves the explicit fallback roots once, evaluates availability from
that same resolved node stream, and reuses the first live fallback root as the hotspot start node.
This keeps route attribution faithful to the work being timed while avoiding duplicate
`ElementRuntime` fallback-root collection and live-node resolution in diagnostics-enabled builds.

## No-Focus Subtree Fallback

The no-focus, no-barrier subtree fallback exists for first-open discovery of custom widget commands.
It does not make focus-bound `text.*` / `edit.*` commands available by scanning arbitrary unfocused
text surfaces. Those commands must be answered by the focused/default route or explicit action-route
fallback roots. This keeps editor/clipboard commands from turning first-open command publication
into full-window subtree traversal.

## Current Evidence

- Source anchors:
  - `crates/fret-ui/src/tree/commands.rs`
  - `crates/fret-ui/src/tree/ui_tree_debug/record.rs`
  - `ecosystem/fret-bootstrap/src/ui_diagnostics/layout_paint_hotspot_diagnostics.rs`
  - `crates/fret-diag/src/stats/bundle_stats_snapshot.rs`
  - `crates/fret-diag/src/stats/bundle_stats_report.inc.rs`
- Latest perf evidence:
  - `target/fret-diag/codex-command-availability-pointer-after-focus-cache-20260516/1778905245660/bundle.schema2.json`
  - `target/fret-diag/codex-command-availability-pointer-after-focus-cache-20260516/stats.command_availability.top10.json`
  - `target/fret-diag/codex-command-availability-pointer-after-focus-cache-20260516/focus_traversal_snapshot.by_frame.json`
- Focused test evidence:
  - `cargo nextest run -p fret-ui action_availability_snapshot_reuses_focus_traversal_within_frame action_availability_snapshot_refreshes_focus_traversal_on_next_frame action_availability_snapshot_publishes_focus_traversal_gating --no-fail-fast`
  - `cargo nextest run -p fret-ui window_command_action_availability_snapshot --no-fail-fast`
  - `cargo nextest run -p fret-diag bundle_stats_projects_command_availability_hotspots --no-fail-fast`
