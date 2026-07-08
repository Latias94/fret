// This file is part of the docking UI implementation.
//
// It is intentionally `pub(super)` only; the public API lives in `dock/mod.rs`.

mod diagnostics;
mod floating_hit;
mod intent;
mod target;
mod transaction;

pub(super) use diagnostics::{compute_dock_drop_resolve_diagnostics, dock_drop_target_diagnostics};
pub(super) use intent::{
    DockPanelDropDrag, DockTabsDropDrag, resolve_dock_drop_intent_panel,
    resolve_dock_drop_intent_tabs,
};
pub(super) use target::resolve_dock_drop_target;
pub(super) use transaction::{
    ResolvedDockDropTransaction, apply_resolved_dock_drop_transaction,
    dock_drop_transaction_debug_kind, resolve_dock_drop_transaction,
    validate_dock_drop_transaction_commit,
};
