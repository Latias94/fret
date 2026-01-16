use std::any::TypeId;

use fret_runtime::ModelId;

use crate::{UiHost, UiTree};

/// Apply all "external" changes that affect the UI tree for the current frame.
///
/// Contract:
/// - Call this before `declarative::render_root` for a given window/frame.
/// - Use the same `changed_*` lists drained from the host/runtime for that frame.
///
/// Rationale:
/// - View-cache subtree reuse relies on correct invalidation flags being set before mount.
/// - This helper makes call sites less error-prone by enforcing a single ordering point.
pub fn propagate_changes<H: UiHost>(
    ui: &mut UiTree<H>,
    app: &mut H,
    changed_models: &[ModelId],
    changed_globals: &[TypeId],
) -> bool {
    let mut did_work = false;
    if !changed_models.is_empty() {
        did_work |= ui.propagate_model_changes(app, changed_models);
    }
    if !changed_globals.is_empty() {
        did_work |= ui.propagate_global_changes(app, changed_globals);
    }
    did_work
}
