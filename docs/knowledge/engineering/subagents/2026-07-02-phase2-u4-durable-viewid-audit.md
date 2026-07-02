---
type: Subagent Finding
title: Phase 2 U4 durable ViewId audit
tags: fret,ui,view-boundary,viewid,phase2,subagent
timestamp: 2026-07-02
related_plan: docs/plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md
git_branch: feat/ui-framework-phase2-refactor
subagent_id: 019f2398-2204-7751-ab10-69c9a6f7a44e
---

# Finding

The read-only audit confirmed that U4's boundary store migration had prepared entity-first storage
but production `ViewId` allocation still came from live `NodeId` through a quarantine helper.
`GlobalElementId` is already stored on `Node` and maintained through the live element index, so it
is the right stable seed for declarative boundary roots.

# Evidence

- Declarative mount creates/reuses nodes from `GlobalElementId` and then sets view-cache flags in
  `crates/fret-ui/src/declarative/mount.rs`.
- `Node` stores `element: Option<GlobalElementId>` and `element_binding_generation` in
  `crates/fret-ui/src/tree/node_storage.rs`.
- `ElementNodeIndex` already maintains live element lookup and stale/detached validation in
  `crates/fret-ui/src/tree/identity.rs`.
- Before this slice, `ViewBoundaryStore` had `BoundaryId`/`ViewId` indexes but
  `ensure_view_boundary_state` still derived `ViewId` from live `NodeId`.

# Recommendation

Use the next U4 cut for durable `ViewId` allocation:

- Allocate/preserve `ViewId` from a stable boundary key, preferring `GlobalElementId`.
- Keep no-element runtime boundaries explicit instead of pretending they have declarative identity.
- Add test accessors that query the store's current view rather than converting `ViewId` back to
  `NodeId`.
- Treat observation fanout as a later slice.

# Disposition

Implemented in the durable ViewId lifecycle slice. `ViewBoundaryStore` now owns
`ViewBoundaryKey -> BoundaryId -> ViewId`, structural detach preserves records with
`live_node: None`, and the live-`NodeId` derived ViewId helper is deleted. Remaining work is the
layout-only dirty-view live projection and observation fanout.
