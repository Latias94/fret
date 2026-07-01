---
type: Decision
title: U7 non-quad resident partial uploads deferred
tags: fret,u7,renderer,geometry-upload,scene-chunks
timestamp: 2026-07-01
related_plan: ../../plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
git_branch: feat/ui-framework-convergence
subagent_id: 019f1d12-ad00-7f40-b741-5367271c41d8
---

# Decision

Keep real resident partial uploads quad-only for U7 and defer non-quad streams to measured follow-up
work.

# Context

The guarded quad instance path is safe because the retained scene chunk reassembly proof covers the
whole quad instance stream before `Skip` or `Partial(ranges)` can run. The same proof is not yet a
complete dependency closure for non-quad streams.

# Evidence

- `crates/fret-render-wgpu/src/renderer/geometry_upload.rs` now applies real `Full` / `Skip` /
  `Partial(ranges)` only to `quad_instances`; other streams still warm resident diagnostics and use
  full uploads.
- `crates/fret-render-wgpu/src/renderer/scene_chunk_encoding_cache.rs` still blocks material state,
  side tables, and non-quad draw cases in the conservative reassembly path.
- Text streams depend on atlas page/UV/glyph residency and text paint closure details; those belong
  to U8 text/glyph budget work before real text stream partial writes.

# Consequences

- U7 can close with a truthful statement: dirty upload is real for quad instances and diagnostics-only
  for non-quad streams.
- The next high-value lane is U8 text/glyph budgets, not broadening partial writes to text or path
  streams.
- If non-quad upload work resumes, start with a side-table-free `viewport_vertices` subset and add
  negative tests for clip masks, text paint closure gaps, resource generation changes, and coverage
  fallback.

# Citations

- Explorer `019f1d12-ad00-7f40-b741-5367271c41d8`
- Commit `6a45373eac feat(render): enable guarded quad partial uploads`
