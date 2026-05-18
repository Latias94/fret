# Pressable Clean Geometry Propagation v1 - Handoff

Status: Closed
Last updated: 2026-05-18

## Current State

The lane has been opened as a narrow follow-on to the closed
`retained-layout-orchestration-v1` workstream. PGP-010 through PGP-050 are complete. The minimal
runtime slice added `Pressable` to the clean-geometry execution allowlist and turned the focused
layout proof green.

PGP-020 audited the `Pressable` geometry and side-effect surfaces. PGP-030 added a focused RED test
showing the current gap: small width-only resize skipped the root Taffy solve but still reran
`Pressable` wrapper layout (`layout_nodes_performed=2`). PGP-040 closed that gap with the smallest
possible allowlist change.

PGP-050 captured fresh UI Gallery resize-jitter evidence. `Pressable` moved off the worst-frame
layout hotspot list; `ViewCache`, `Scroll`, and a small `Flex` owner remain. This lane is closed.

## Active Task

- None. The lane is closed after PGP-050.

## Decisions Since Last Update

- Do not reopen `retained-layout-orchestration-v1`; it is closed after the `Semantics` slice.
- Do not add `Pressable` to the clean-geometry execution allowlist until source audit and RED tests
  prove the side-effect model.
- Treat `Scroll` and `ViewCache` as separate future lanes, not part of this `Pressable` proof.
- Keep this work in `fret-ui` mechanism code and tests; no component policy changes are in scope.
- PGP-020 found no audited side effect that requires rerunning `Pressable` layout during clean
  width-only bounds propagation.
- PGP-030 RED failure is specific: `layout_engine_solves=0`, no clean-geometry rejection noise, but
  `layout_nodes_performed=2`.
- PGP-040 added `Pressable` to the execution allowlist and preserved all audited interaction gates.
- PGP-050 confirms `Pressable` is no longer the local worst-frame layout hotspot in the shared
  resize-jitter repro. Do not expand this lane to `ViewCache` or `Scroll`.

## Blockers

- None.

## Next Recommended Action

Open a new narrow lane for `ViewCache` or `Scroll` only with fresh attribution and a source audit of
their cache, viewport, clipping, and input semantics.
