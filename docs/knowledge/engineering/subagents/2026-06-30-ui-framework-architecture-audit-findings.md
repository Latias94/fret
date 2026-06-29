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
