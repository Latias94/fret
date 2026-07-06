---
type: Work Progress
title: Examples table stress owner helper
timestamp: 2026-07-06T00:00:00Z
git_branch: refactor/examples-table-stress-owner
tags: fret,ui-framework,public-surface,examples,table,raw-model,owner
---

# Summary

`apps/fret-examples/src/table_stress_demo.rs` now routes table stress command writes through a
demo-local `TableStressModelOwner`.

The demo remains a lower-level retained table/perf harness because it owns `UiTree<App>`, retained
`TableState`, a large row set, and frame/allocation diagnostics. The cleanup only removes the
copyable pattern of writing `TableState` and `items_revision` directly from driver command paths.

# Decision

Keep the stress-harness classification. The public app-facing table path should stay
`LocalState`/binding-first, while this harness can keep explicit retained table state for perf and
debugging. The local owner helper makes those shared-model mutations named and source-gated.

# Evidence

- Red proof before implementation:
  `cargo nextest run -p fret-examples --test table_stress_demo_surface table_stress_demo_model_writes_stay_behind_owner_helper --no-fail-fast`
  failed because `TableStressModelOwner` did not exist.
- The same test now requires `TableStressModelOwner::{toggle_sorting,toggle_role_filter,toggle_global_filter,clear_filters,bump_items_revision}(...)`
  and forbids direct, generic, `update_any`, and UFCS `ModelStore` bypasses in production demo
  source.
- `table_stress_model_owner_preserves_command_state_transitions` covers the owner behavior directly:
  sorting asc/desc/none, role/global filters with page reset, clear filters, and wrapping
  `items_revision` increment.
- `cargo nextest run -p fret-examples --test table_stress_demo_surface --no-fail-fast` passes.

# Controls Binding Follow-Up

Branch `refactor/table-stress-controls` keeps the retained stress-harness classification but moves
the table state and item revision models behind `TableStressControls`.

- Startup model allocation now happens in `TableStressControls::new(...)`.
- `TableStressWindowState` stores one `controls` field instead of separate `Model<TableState>` and
  `Model<u64>` fields.
- Keyboard events call semantic controls methods instead of passing model handles through
  `TableStressDriver` helpers.
- Render uses `render_snapshot(...)` for layout subscriptions/readout state and `table_model()`
  only at the retained table component seam.

# Next

Do not mechanically convert retained table stress state to `LocalState`. `TableStressControls` is
the named local contract for this perf harness; future table/data-grid cleanup should add similarly
narrow bindings only when the retained-state semantics stay explicit.
