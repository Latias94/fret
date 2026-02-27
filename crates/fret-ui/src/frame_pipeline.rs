use std::any::TypeId;

use fret_core::{AppWindowId, NodeId, Rect, UiServices};
use fret_runtime::ModelId;

use crate::element::AnyElement;
use crate::elements::ElementContext;
use crate::{UiHost, UiTree, declarative};

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

/// Render a declarative root after applying the provided per-frame change lists.
///
/// This helper exists to make the "propagate changes before mount" ordering hard to get wrong in
/// runner/driver code.
#[allow(clippy::too_many_arguments)]
pub fn render_root_with_changes<H, I>(
    ui: &mut UiTree<H>,
    app: &mut H,
    services: &mut dyn UiServices,
    window: AppWindowId,
    bounds: Rect,
    root_name: &str,
    changed_models: &[ModelId],
    changed_globals: &[TypeId],
    render: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> NodeId
where
    H: UiHost + 'static,
    I: IntoIterator<Item = AnyElement>,
{
    let _ = propagate_changes(ui, app, changed_models, changed_globals);
    declarative::render_root(ui, app, services, window, bounds, root_name, render)
}

/// Convenience wrapper around [`render_root_with_changes`] for base-layer roots.
#[allow(clippy::too_many_arguments)]
pub fn render_base_root_with_changes<H, I>(
    ui: &mut UiTree<H>,
    app: &mut H,
    services: &mut dyn UiServices,
    window: AppWindowId,
    bounds: Rect,
    root_name: &str,
    changed_models: &[ModelId],
    changed_globals: &[TypeId],
    render: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> NodeId
where
    H: UiHost + 'static,
    I: IntoIterator<Item = AnyElement>,
{
    let root = render_root_with_changes(
        ui,
        app,
        services,
        window,
        bounds,
        root_name,
        changed_models,
        changed_globals,
        render,
    );
    ui.set_root(root);
    root
}
