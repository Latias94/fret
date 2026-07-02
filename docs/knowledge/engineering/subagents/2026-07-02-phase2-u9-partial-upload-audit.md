---
type: Subagent Finding
title: Phase 2 U9 partial upload guard audit
tags: fret,phase2,u9,renderer,subagent
timestamp: 2026-07-02
related_plan: docs/plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md
subagent_id: 019f2498-a433-76a2-a5a1-c7189341c570
git_branch: feat/ui-framework-phase2-refactor
---

# Finding

The first safe non-quad partial upload slice should be `viewport_vertices`, but only for
`VertexColor` draws. `viewport_vertices` also carries `Image` and `ViewportSurface` draws, whose
resource and render-target dependencies are not closed enough for dirty-range writes.

# Evidence

The existing protection chain has three layers:

- Payload-plan alignment requires chunk payload shape and stream fingerprints to match the flat
  render-plan segment.
- Resident stream coverage requires safe segment ranges to cover the whole GPU stream before a
  partial plan is accepted.
- Upload execution still falls back to full writes when the stream is unsupported, uninitialized,
  resized, coverage-incomplete, or cannot be sliced as POD.

The existing per-stream upload counters already distinguish quad and viewport writes through
`renderer_geometry_upload_quad_instance_{bytes,write_count}` and
`renderer_geometry_upload_viewport_vertex_{bytes,write_count}`.

# Recommendation

Open only the `VertexColor` viewport vertex path in U9. Keep these streams on full upload until a
future closure slice proves them:

- `Image` and `ViewportSurface` contributions to `viewport_vertices`
- text glyph instances, text vertices, and text paints
- path vertices and path paints
- mask/text-vertex draws, material-dependent draws, clip side tables, mask side tables, and effect
  side tables

# Disposition

Adopted for the U9 implementation. The production gate checks render-plan flags before including
viewport vertex ranges in the resident partial-upload signatures, and tests prove Image and
ViewportSurface remain full upload.

# Citations

- Subagent `019f2498-a433-76a2-a5a1-c7189341c570`
- [U9 progress](../progress/2026-07-02-phase2-u9-viewport-partial-upload.md)
