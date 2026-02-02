//! Immediate-mode (`fret-imui`) adapters for `fret-node`.
//!
//! The node graph UI is implemented as a retained widget subtree. This module provides a small
//! bridge that hosts that subtree inside the declarative element runtime via the
//! `unstable-retained-bridge` feature.

use fret_core::NodeId;
use fret_ui::retained_bridge::RetainedSubtreeProps;
use fret_ui::{UiHost, UiTree};

/// Adds a retained node-graph subtree to an `imui` output list.
///
/// The `build` closure is invoked **only when the subtree is first mounted** (or needs to be
/// recreated after node GC), and must return the retained root node ID.
#[track_caller]
pub fn retained_subtree<H: UiHost + 'static>(
    ui: &mut fret_imui::ImUi<'_, '_, H>,
    build: impl Fn(&mut UiTree<H>) -> NodeId + 'static,
) {
    let element = ui
        .cx_mut()
        .retained_subtree(RetainedSubtreeProps::new(build));
    ui.add(element);
}

/// Adds a retained node-graph subtree to an `imui` output list with explicit layout props.
#[track_caller]
pub fn retained_subtree_with<H: UiHost + 'static>(
    ui: &mut fret_imui::ImUi<'_, '_, H>,
    props: RetainedSubtreeProps,
) {
    let element = ui.cx_mut().retained_subtree(props);
    ui.add(element);
}
