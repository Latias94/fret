---
type: Subagent Finding
title: UI framework architecture audit findings
tags: fret,architecture,performance,dx,gpui
timestamp: 2026-06-30
related_plan: docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
---

# Finding

Four audit lanes converged on the same target architecture.
Fret should not replace its current direction; it should make the existing direction enforceable through contracts and gates.

# Evidence

- Repository boundary audit: `crates/fret-ui` has clean dependency posture, but public vocabulary and example paths risk turning the mechanism layer into a policy/component layer.
- GPUI/Zed comparison: copy per-frame element trees, externalized state, `notify`, dirty views, prepaint frame products, dispatch snapshots, and cached frame-product reuse; avoid GPUI platform coupling and keep policy outside `fret-ui`.
- Framework consumer audit: the user journey reaches todo but lacks a promoted second-hour app ladder for settings, command palette, data table, workspace shell, and canvas/node graph.
- Performance audit: Fret already has retained layout, dispatch snapshots, bounds tree, renderer cache, and text atlas infrastructure, but identity, dirty graph, scene encoding, GPU upload, and text/glyph cache budgets are not yet unified enough for editor scale.

# Recommendation

Use `docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md` as the next execution contract.
Start with contract alignment and source-policy gates, then add architecture metrics before stable handle, dirty graph, scene chunk, and text/glyph budget migrations.

# Disposition

Integrated into the plan as U1 through U9.
No code was changed by the subagents.

## Follow-up Audits During Execution

Two older read-only audit agents were closed during U3 execution:

- `019f1440-5447-75e2-b450-6700e61df8c6` found performance hot spots to feed U4/U7/U8: layout fast paths still pay root/dirty scans, subtree invalidation clones child vectors, paint-cache replay rebases descendants after movement, text layout keys clone/compare large objects, text input copies full strings, atlas revision invalidates scene encoding too broadly, text residency scans whole scenes, virtual list range allocation and rerender spikes need gates, managed-surface unplaced-child checks are quadratic, and diagnostics/HUD can pollute perf observations.
- `019f1440-7fc5-7330-83d1-b51dae3f56f3` found ecosystem refactor risks for U6/U9: shadcn public/raw surface is too wide, dropdown/select are giant policy files, select scroll buttons should become reusable policy, table code and tests freeze implementation shape, headless table row model is too monolithic, docking-specific diagnostics leak into runtime, docking tests are oversized, chart/node/plot state and panels need narrower controllers and fixtures.

U4 explorer `019f1627-edfc-7001-be3e-8ef1d98c076e` recommends starting with counters before identity handle migration:

- Add `UiDebugFrameStats` fields for identity seeded hit/stale, fallback scans/nodes/hit/miss, parent pointer repair, GC reachability/stale removals, dirty frontier breadth, dispatch snapshot cache hit/miss/build/invalidation, and model/global observation churn.
- Instrument first in `tree/layout/state.rs`, `declarative/mount.rs`, `tree/dispatch_snapshot.rs`, `tree/observation.rs`, and debug stats export paths.
- Do not remove fallback scans or introduce `StableNodeHandle` in the first U4 slice. Keep fallback scans as counted temporary correctness paths, then gate them toward zero after warmup.
