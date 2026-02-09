//! Immediate-mode (`UiWriter`) adapters for `fret-docking`.
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

use fret_authoring::UiWriter;
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
    /// Optional semantics-only drag anchor placed on the dock space tab bar.
    ///
    /// This is intended for scripted diagnostics runs (`fretboard diag`) so scripts can start a
    /// deterministic dock drag without relying on brittle role/label selectors for tabs.
    pub tab_drag_anchor_test_id: Option<&'static str>,
}

impl Default for DockSpaceImUiOptions {
    fn default() -> Self {
        let mut layout = LayoutStyle::default();
        layout.size.width = fret_ui::element::Length::Fill;
        layout.size.height = fret_ui::element::Length::Fill;
        layout.flex.grow = 1.0;
        Self {
            layout,
            test_id: None,
            tab_drag_anchor_test_id: None,
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
    ui: &mut impl UiWriter<H>,
    options: DockSpaceImUiOptions,
    configure: impl FnMut(&mut H, AppWindowId) + 'static,
) {
    let window = ui.with_cx_mut(|cx| cx.window);

    let configure: Rc<RefCell<Box<dyn FnMut(&mut H, AppWindowId)>>> =
        Rc::new(RefCell::new(Box::new(configure)));
    let test_id = options.test_id;
    let tab_drag_anchor_test_id = options.tab_drag_anchor_test_id;

    let props = fret_ui::retained_bridge::RetainedSubtreeProps {
        layout: options.layout,
        factory: fret_ui::retained_bridge::RetainedSubtreeFactory::new::<H>({
            let configure = configure.clone();
            move |ui_tree| {
                build_dock_host(
                    ui_tree,
                    window,
                    test_id,
                    tab_drag_anchor_test_id,
                    configure.clone(),
                )
            }
        }),
    };

    let element = ui.with_cx_mut(|cx| cx.retained_subtree(props));
    ui.add(element);
}

/// Convenience wrapper that uses default options (fill layout, no test id).
#[track_caller]
pub fn dock_space<H: UiHost + 'static>(
    ui: &mut impl UiWriter<H>,
    configure: impl FnMut(&mut H, AppWindowId) + 'static,
) {
    dock_space_with(ui, DockSpaceImUiOptions::default(), configure);
}

fn build_dock_host<H: UiHost + 'static>(
    ui_tree: &mut UiTree<H>,
    window: AppWindowId,
    test_id: Option<&'static str>,
    tab_drag_anchor_test_id: Option<&'static str>,
    configure: Rc<RefCell<Box<dyn FnMut(&mut H, AppWindowId)>>>,
) -> fret_core::NodeId {
    let test_id = test_id.unwrap_or("dock-space");
    let dock_space = create_dock_space_node_with_test_id(ui_tree, window, test_id);
    let tab_drag_anchor = tab_drag_anchor_test_id
        .map(|id| ui_tree.create_node_retained(DockTabDragAnchor { test_id: id }));
    let root = ui_tree.create_node_retained(DockHostRoot::<H> {
        dock_space,
        tab_drag_anchor,
        configure,
    });
    let mut children = vec![dock_space];
    if let Some(anchor) = tab_drag_anchor {
        children.push(anchor);
    }
    ui_tree.set_children(root, children);
    root
}

struct DockTabDragAnchor {
    test_id: &'static str,
}

impl<H: UiHost> Widget<H> for DockTabDragAnchor {
    fn hit_test(&self, _bounds: fret_core::Rect, _position: fret_core::Point) -> bool {
        false
    }

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        cx.set_role(SemanticsRole::Group);
        cx.set_test_id(self.test_id);
    }
}

struct DockHostRoot<H: UiHost> {
    dock_space: fret_core::NodeId,
    tab_drag_anchor: Option<fret_core::NodeId>,
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
        if let Some(anchor) = self.tab_drag_anchor {
            const SIZE_PX: f32 = 12.0;
            const TAB_BAR_H_PX: f32 = 28.0;

            let x = cx.bounds.origin.x.0 + cx.bounds.size.width.0 * 0.25;
            let y = cx.bounds.origin.y.0 + TAB_BAR_H_PX * 0.5;
            let half = SIZE_PX * 0.5;

            let rect = fret_core::Rect::new(
                fret_core::Point::new(
                    fret_core::Px((x - half).max(cx.bounds.origin.x.0)),
                    fret_core::Px((y - half).max(cx.bounds.origin.y.0)),
                ),
                fret_core::Size::new(fret_core::Px(SIZE_PX), fret_core::Px(SIZE_PX)),
            );
            let _ = cx.layout_in(anchor, rect);
        }
        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        cx.paint(self.dock_space, cx.bounds);
    }
}
