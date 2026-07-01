---
type: Subagent Finding
title: U9 AppUi shell split audit
tags: fret,u9,facade,app-ui,subagent
timestamp: 2026-07-02
related_plan: docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
subagent_id: 019f1ffb-5277-79a2-ac4f-519bddec460c
---

# Finding

Explorer `019f1ffb-5277-79a2-ac4f-519bddec460c` recommended making
`ecosystem/fret/src/view/shell.rs` the owner of the `AppUi` shell, not only the impl-method holder.
The clean boundary is: `view.rs` keeps module declarations, re-exports, and tests; `shell.rs` owns
`AppUi`, its documentation, the no-`Deref` boundary note, and the core shell helper impl.

# Evidence

Sibling modules access `AppUi` internals directly:

- `view/layout_query.rs` carries action handler fields across nested layout-query regions.
- `view/effects.rs` reads `action_root` and `cx`.
- `view/bridges.rs` forwards through the underlying `ElementContext`.
- `view/data.rs` reaches the underlying `ElementContext` for selector/query/mutation substrate calls.

Moving the struct without changing field visibility would trigger Rust privacy errors. The
recommended visibility is `pub(super)` for internal fields and `pub(super)` for helper methods used
by sibling modules, while preserving `pub(crate)` for `new`, `watch_local`, and
`take_action_handlers` where existing source-shape tests expect that boundary.

# Recommendation

Move the `AppUi` struct and shell impl together into `view/shell.rs`, add `pub use shell::AppUi` in
`view.rs`, keep source-shape tests aggregating `SHELL_RS_SOURCE`, and verify with `cargo check -p
fret --lib` before broader gates because privacy errors surface there first.

# Disposition

Integrated into the U9 AppUi shell split.
