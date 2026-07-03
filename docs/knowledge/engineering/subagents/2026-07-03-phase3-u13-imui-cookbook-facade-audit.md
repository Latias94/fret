---
type: Subagent Finding
title: Phase 3 U13 IMUI cookbook facade audits
tags: fret,phase3,u13,imui,cookbook,facade,subagent
timestamp: 2026-07-03
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
git_branch: feat/ui-framework-phase2-refactor
subagent_ids:
  - 019f2a19-950c-7141-8271-574b19ca596e
  - 019f2a19-d0a9-77e1-b1e7-218660bd1cc5
---

# Finding

Two readonly explorers audited the remaining IMUI cookbook facade gaps.

- `imui_action_basics.rs` and `imui_editor_controls_basics.rs` are app-facing IMUI teaching
  examples, but they were still correctly quarantined while their sources exposed raw
  `fret_runtime::Model`, `fret_ui::element::ColumnProps`, and `fret_core::Color`.
- The command vocabulary does not need a new facade: default app code can import
  `fret::commands::{CommandId, CommandMeta, CommandScope}` explicitly.
- The IMUI/editor state gap should be solved with narrow `LocalState<T>` adapters under
  `fret::imui` / `fret::imui::editor`, not with a crate-wide `IntoModel<T>` or by re-exporting raw
  runtime handles through `fret::app::prelude::*`.
- `imui_debug_draw_basics.rs` is not an advanced/manual quarantine surface today; it is a debug draw
  facade proof. Its remaining app-facing leak is direct `ColumnProps` use.
- `imui_plot_basics.rs` should stay in advanced/manual quarantine until `fret-plot` gains a
  plot-specific handle/binding that hides raw `Model<LinePlotModel>`, `PlotState`, `PlotOutput`,
  and host layout plumbing.

# Recommendation

- Migrate `imui_action_basics.rs` by deleting the mixed GenUI panel instead of creating a broad
  GenUI state facade for one example. Keep the lesson focused on declarative + IMUI action dispatch.
- Migrate `imui_editor_controls_basics.rs` next by adding editor-control `LocalState<T>` adapters
  plus an app-facing color export; do not widen app prelude.
- Remove `ColumnProps` from `imui_debug_draw_basics.rs` with an app-facing host wrapper or safe
  `imui(...)`/`ui::v_flex(...)` shape.
- Keep plot migration separate and solve it in `fret-plot` with a plot-specific binding/handle.

# Disposition

The action example recommendation was implemented immediately: the GenUI panel was deleted, the
example now uses `LocalState<String>`, `fret::commands`, and `imui_in(...)`, and the source-policy
quarantine record was removed. Editor, debug draw, and plot remain follow-up slices.

# Citations

- [Phase 3 plan](../../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md)
- [IMUI action local-state migration](../progress/2026-07-03-phase3-u13-imui-action-local-state.md)
