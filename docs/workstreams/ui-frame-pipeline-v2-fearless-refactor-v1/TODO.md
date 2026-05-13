# TODO

Status: Active
Last updated: 2026-05-13

## P0 Contract Setup

- [x] Create workstream lane for Frame Pipeline v2.
- [x] Add ADR 0327 as the proposed frame-pipeline and boundary contract.
- [ ] Review ADR 0327 and either accept it or revise it before broad code migration.
- [ ] Add an assumptions-first baseline audit of current `UiTree` build/layout/prepaint/paint paths.
- [ ] Add a source map of old paths that are migration candidates:
  - `ViewCacheProps::contained_layout`,
  - view-cache root bookkeeping,
  - paint-cache replay bookkeeping,
  - code-editor-local frame state,
  - layout invalidation propagation,
  - prepaint diagnostics.

## P1 First Vertical Slice: Code Editor Boundary

- [ ] Define the first internal `BoundaryId` / boundary-state shape or a narrower transitional
  equivalent.
- [ ] Make UI Gallery code-editor content root report boundary-level reuse/reject reasons.
- [ ] Move code-editor frame-derived row state toward shared prepaint ownership.
- [ ] Split code-editor paint attribution into boundary prepaint, paint replay, and renderer payload
  buckets.
- [ ] Add or promote a stricter code-editor paint stressor if resize probes are no longer sensitive
  enough.
- [ ] Prove `paint.widget` or total p95/max improves by at least 20-30% on the selected bottleneck.

## P2 Runtime Migration

- [ ] Convert layout containment from a standalone flag into boundary dependency metadata.
- [ ] Convert paint replay into boundary-owned scene-fragment reuse where possible.
- [ ] Make prepaint diagnostics first-class per boundary.
- [ ] Remove duplicated or superseded debug counters after boundary diagnostics cover them.
- [ ] Keep `fret-ui` mechanism-only; move any policy decisions back to ecosystem crates.

## P3 Delete Old Paths

- [ ] Write a deletion audit before closeout.
- [ ] Delete or retire old private paths that v2 replaces.
- [ ] Remove migration-only env knobs that no longer have a diagnostic purpose.
- [ ] Update first-party examples and docs if public authoring guidance changes.
- [ ] Update `docs/adr/IMPLEMENTATION_ALIGNMENT.md` when ADR 0327 is implemented or superseded.

## Always-On Gates

- [ ] `python3 tools/check_layering.py`
- [ ] Focused `fret-ui` unit tests for any boundary/invalidation change.
- [ ] `ui-code-editor-resize-probes` perf gate on macOS M4 for code-editor slices.
- [ ] Worst-bundle `diag stats` attribution for every perf claim.
