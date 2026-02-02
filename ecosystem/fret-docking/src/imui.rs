//! Immediate-mode (`fret-imui`) adapters for `fret-docking`.
//!
//! This module provides a tiny glue layer that lets imui apps embed a docking host without
//! re-implementing the retained-bridge wiring in every application.
//!
//! Notes:
//! - Docking remains policy-heavy and stateful; this module only provides embedding helpers.
//! - Window creation and dock ops must still be handled by the runner/driver (see
//!   `DockingRuntime` and `runtime::{handle_dock_op, handle_dock_window_created, handle_dock_before_close_window}`).

use std::cell::RefCell;
use std::rc::Rc;

use fret_core::{AppWindowId, SemanticsRole};
use fret_ui::element::LayoutStyle;
use fret_ui::retained_bridge::{LayoutCx, PaintCx, SemanticsCx, UiTreeRetainedExt as _, Widget};
use fret_ui::{UiHost, UiTree};

use crate::{create_dock_space_node_with_test_id, render_and_bind_dock_panels};

/// Options for embedding a dock space in an imui tree.
#[derive(Debug, Clone)]
pub struct DockSpaceImUiOptions {
    /// Layout for the host element (defaults to fill).
    pub layout: LayoutStyle,
    /// Optional semantics test id for the dock space root.
    pub test_id: Option<&'static str>,
}

impl Default for DockSpaceImUiOptions {
    fn default() -> Self {
        let mut layout = LayoutStyle::default();
        layout.size.width = fret_ui::element::Length::Fill;
        layout.size.height = fret_ui::element::Length::Fill;
        Self {
            layout,
            test_id: None,
        }
    }
}

/// Embed a docking host into an imui output list.
///
/// `configure` is called every frame (during layout) before docking renders/binds panel roots.
/// Use it to:
///
/// - ensure panels exist (`DockManager::ensure_panel`),
/// - ensure `DockGraph` window roots are set,
/// - update `ViewportPanel` targets and sizes (if you embed engine viewports).
#[track_caller]
pub fn dock_space_with<H: UiHost + 'static>(
    ui: &mut fret_imui::ImUi<'_, '_, H>,
    options: DockSpaceImUiOptions,
    configure: impl FnMut(&mut H, AppWindowId) + 'static,
) {
    let window = ui.cx_mut().window;

    let configure: Rc<RefCell<Box<dyn FnMut(&mut H, AppWindowId)>>> =
        Rc::new(RefCell::new(Box::new(configure)));
    let test_id = options.test_id;

    let props = fret_ui::retained_bridge::RetainedSubtreeProps {
        layout: options.layout,
        factory: fret_ui::retained_bridge::RetainedSubtreeFactory::new::<H>({
            let configure = configure.clone();
            move |ui_tree| build_dock_host(ui_tree, window, test_id, configure.clone())
        }),
    };

    let element = ui.cx_mut().retained_subtree(props);
    ui.add(element);
}

/// Convenience wrapper that uses default options (fill layout, no test id).
#[track_caller]
pub fn dock_space<H: UiHost + 'static>(
    ui: &mut fret_imui::ImUi<'_, '_, H>,
    configure: impl FnMut(&mut H, AppWindowId) + 'static,
) {
    dock_space_with(ui, DockSpaceImUiOptions::default(), configure);
}

fn build_dock_host<H: UiHost + 'static>(
    ui_tree: &mut UiTree<H>,
    window: AppWindowId,
    test_id: Option<&'static str>,
    configure: Rc<RefCell<Box<dyn FnMut(&mut H, AppWindowId)>>>,
) -> fret_core::NodeId {
    let test_id = test_id.unwrap_or("dock-space");
    let dock_space = create_dock_space_node_with_test_id(ui_tree, window, test_id);
    let root = ui_tree.create_node_retained(DockHostRoot::<H> {
        dock_space,
        configure,
    });
    ui_tree.set_children(root, vec![dock_space]);
    root
}

struct DockHostRoot<H: UiHost> {
    dock_space: fret_core::NodeId,
    configure: Rc<RefCell<Box<dyn FnMut(&mut H, AppWindowId)>>>,
}

impl<H: UiHost + 'static> Widget<H> for DockHostRoot<H> {
    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        cx.set_role(SemanticsRole::Group);
        cx.set_test_id("dock-space-root");
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> fret_core::Size {
        let Some(window) = cx.window else {
            return cx.available;
        };

        (self.configure.borrow_mut())(cx.app, window);

        render_and_bind_dock_panels(
            cx.tree,
            cx.app,
            cx.services,
            window,
            cx.bounds,
            self.dock_space,
        );

        let _ = cx.layout_in(self.dock_space, cx.bounds);
        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        cx.paint(self.dock_space, cx.bounds);
    }
}
