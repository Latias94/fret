---
type: Subagent Finding
title: Phase 3 U13 cookbook quarantine readonly audit
tags: fret,phase3,u13,subagent,cookbook,quarantine
timestamp: 2026-07-03
subagent_id: 019f29e6-cc07-7a63-9a77-7aad3adb17b7
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
---

# Finding

Readonly explorer `019f29e6-cc07-7a63-9a77-7aad3adb17b7` audited remaining U13 advanced/manual
records and reported four high-value follow-ons.

# Evidence

- IMUI action/editor examples still expose `fret_runtime::{CommandId, CommandMeta, CommandScope,
  Model}`, `fret_core::Color`, and low-level `fret_ui::element::ColumnProps`.
- IMUI debug draw and plot examples still expose `fret_core` geometry/color types, raw
  `Model<T>`, and `ColumnProps`.
- `data_table_basics.rs` is already classified default clean but still imports
  `fret_runtime::Model`; the source-policy gate currently catches `Model<` only for unclassified
  public examples, not every default-clean record.
- `async_inbox_basics.rs` remains a true host/effect action case because dispatcher wakeups and
  async inbox state are not yet wrapped by an app-facing action helper.

# Recommendation

- Prefer IMUI facade work next: add explicit `fret::commands` / `LocalState` adapters and hide
  `ColumnProps` behind IMUI panel helpers without widening `fret::app::prelude::*`.
- Then migrate `data_table_basics.rs` or add a narrow table output facade before tightening
  default-clean policy against `use fret_runtime::Model` / `Model<`.
- Keep `async_inbox_basics.rs` in advanced/manual quarantine until a host-effect action helper can
  encode dispatcher wakeup semantics without reopening the raw action host.

# Disposition

The undo portion was handled immediately in the main thread by deleting its over-conservative RAF
path and moving it to `LocalStateTxn` action helpers. Async remains deferred because it has a real
host-effect dependency.

# Citations

- [Phase 3 plan](../../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md)
- [Undo facade cookbook migration](../progress/2026-07-03-phase3-u13-undo-facade-cookbook-migration.md)
