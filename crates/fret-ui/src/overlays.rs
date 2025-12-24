use crate::{
    UiHost,
    tree::{UiLayerId, UiTree},
    widgets::{
        CommandPaletteOverlay, ContextMenu, ContextMenuService, DialogOverlay, DialogService,
        Popover, PopoverService, ToastOverlay, TooltipOverlay,
    },
};
use fret_core::{AppWindowId, NodeId, TextService};
use fret_runtime::{CommandId, Effect};

/// Standard window-level UI overlays (tooltips, popovers, context menus).
///
/// This helper exists to reduce per-app boilerplate: many apps want the same overlay widgets wired
/// into `UiTree` layers with consistent open/close + focus restoration behavior.
#[derive(Debug)]
pub struct WindowOverlays {
    _tooltip_node: NodeId,
    _toast_node: NodeId,

    command_palette_layer: UiLayerId,
    command_palette_node: NodeId,
    command_palette_previous_focus: Option<NodeId>,

    dialog_layer: UiLayerId,
    dialog_node: NodeId,
    dialog_previous_focus: Option<NodeId>,

    popover_layer: UiLayerId,
    popover_node: NodeId,
    popover_previous_focus: Option<NodeId>,

    context_menu_layer: UiLayerId,
    context_menu_node: NodeId,
    context_menu_previous_focus: Option<NodeId>,
}

impl WindowOverlays {
    pub fn install<H: UiHost>(ui: &mut UiTree<H>) -> Self {
        let tooltip_node = ui.create_node(TooltipOverlay::new());
        let _ = ui.push_overlay_root_ex(tooltip_node, false, false);

        // Toasts should be visually on top but pointer-transparent outside toast bounds.
        // The overlay is hit-testable and relies on per-widget hit testing (`Widget::hit_test`).
        let toast_node = ui.create_node(ToastOverlay::new());
        let _ = ui.push_overlay_root_ex(toast_node, false, true);

        let command_palette_node = ui.create_node(CommandPaletteOverlay::new());
        let command_palette_layer = ui.push_overlay_root(command_palette_node, true);
        ui.set_layer_visible(command_palette_layer, false);

        let dialog_node = ui.create_node(DialogOverlay::new());
        let dialog_layer = ui.push_overlay_root(dialog_node, true);
        ui.set_layer_visible(dialog_layer, false);

        let popover_node = ui.create_node(Popover::new());
        let popover_layer = ui.push_overlay_root(popover_node, true);
        ui.set_layer_visible(popover_layer, false);

        let context_menu_node = ui.create_node(ContextMenu::new());
        let context_menu_layer = ui.push_overlay_root(context_menu_node, true);
        ui.set_layer_visible(context_menu_layer, false);

        Self {
            _tooltip_node: tooltip_node,
            _toast_node: toast_node,
            command_palette_layer,
            command_palette_node,
            command_palette_previous_focus: None,
            dialog_layer,
            dialog_node,
            dialog_previous_focus: None,
            popover_layer,
            popover_node,
            popover_previous_focus: None,
            context_menu_layer,
            context_menu_node,
            context_menu_previous_focus: None,
        }
    }

    pub fn command_palette_node(&self) -> NodeId {
        self.command_palette_node
    }

    pub fn handle_command<H: UiHost>(
        &mut self,
        app: &mut H,
        ui: &mut UiTree<H>,
        text: &mut dyn TextService,
        window: AppWindowId,
        command: &CommandId,
    ) -> bool {
        match command.as_str() {
            "command_palette.open" => {
                let visible = ui.is_layer_visible(self.command_palette_layer);
                if !visible {
                    self.command_palette_previous_focus = ui.focus();
                    ui.set_layer_visible(self.command_palette_layer, true);
                }

                let focus = ui
                    .children(self.command_palette_node)
                    .first()
                    .copied()
                    .and_then(|root| ui.first_focusable_descendant(root))
                    .unwrap_or(self.command_palette_node);
                ui.set_focus(Some(focus));
                app.request_redraw(window);
                true
            }
            "command_palette.close" => {
                if ui.is_layer_visible(self.command_palette_layer) {
                    ui.cleanup_subtree(text, self.command_palette_node);
                    ui.set_layer_visible(self.command_palette_layer, false);
                }

                if let Some(prev) = self.command_palette_previous_focus.take() {
                    ui.set_focus(Some(prev));
                }

                app.request_redraw(window);
                true
            }
            "command_palette.toggle" => {
                if ui.is_layer_visible(self.command_palette_layer) {
                    return self.handle_command(
                        app,
                        ui,
                        text,
                        window,
                        &CommandId::from("command_palette.close"),
                    );
                }
                self.handle_command(
                    app,
                    ui,
                    text,
                    window,
                    &CommandId::from("command_palette.open"),
                )
            }
            "dialog.open" => {
                let has_request = app
                    .global::<DialogService>()
                    .and_then(|s| s.request(window))
                    .is_some();
                if !has_request {
                    return true;
                }

                let visible = ui.is_layer_visible(self.dialog_layer);
                if !visible {
                    self.dialog_previous_focus = ui.focus();
                    ui.set_layer_visible(self.dialog_layer, true);
                }
                ui.set_focus(Some(self.dialog_node));
                app.request_redraw(window);
                true
            }
            "dialog.close" => {
                if ui.is_layer_visible(self.dialog_layer) {
                    ui.cleanup_subtree(text, self.dialog_node);
                    ui.set_layer_visible(self.dialog_layer, false);
                }

                app.with_global_mut(DialogService::default, |service, app| {
                    let action = service.take_pending_action(window);
                    service.clear(window);
                    if let Some(command) = action {
                        app.push_effect(Effect::Command {
                            window: Some(window),
                            command,
                        });
                    }
                });

                if let Some(prev) = self.dialog_previous_focus.take() {
                    ui.set_focus(Some(prev));
                }

                app.request_redraw(window);
                true
            }
            "popover.open" => {
                let has_request = app
                    .global::<PopoverService>()
                    .and_then(|s| s.request(window))
                    .is_some();
                if !has_request {
                    return true;
                }

                let visible = ui.is_layer_visible(self.popover_layer);
                if !visible {
                    self.popover_previous_focus = ui.focus();
                    ui.set_layer_visible(self.popover_layer, true);
                }

                let request_focus = app
                    .global::<PopoverService>()
                    .and_then(|s| s.request(window))
                    .map(|(_, req)| req.request_focus)
                    .unwrap_or(true);
                if request_focus {
                    ui.set_focus(Some(self.popover_node));
                }
                app.request_redraw(window);
                true
            }
            "popover.close" => {
                if ui.is_layer_visible(self.popover_layer) {
                    ui.cleanup_subtree(text, self.popover_node);
                    ui.set_layer_visible(self.popover_layer, false);
                }

                app.with_global_mut(PopoverService::default, |service, _app| {
                    service.clear_request(window);
                });

                if let Some(prev) = self.popover_previous_focus.take() {
                    ui.set_focus(Some(prev));
                }

                app.request_redraw(window);
                true
            }
            "context_menu.open" => {
                let has_request = app
                    .global::<ContextMenuService>()
                    .and_then(|s| s.request(window))
                    .is_some();
                if !has_request {
                    return true;
                }

                let visible = ui.is_layer_visible(self.context_menu_layer);
                if !visible {
                    self.context_menu_previous_focus = ui.focus();
                    ui.set_layer_visible(self.context_menu_layer, true);
                }
                ui.set_focus(Some(self.context_menu_node));
                app.request_redraw(window);
                true
            }
            "context_menu.close" => {
                if ui.is_layer_visible(self.context_menu_layer) {
                    ui.cleanup_subtree(text, self.context_menu_node);
                    ui.set_layer_visible(self.context_menu_layer, false);
                }

                app.with_global_mut(ContextMenuService::default, |service, app| {
                    let action = service.take_pending_action(window);
                    service.clear(window);
                    if let Some(command) = action {
                        app.push_effect(Effect::Command {
                            window: Some(window),
                            command,
                        });
                    }
                });

                if let Some(prev) = self.context_menu_previous_focus.take() {
                    ui.set_focus(Some(prev));
                }

                app.request_redraw(window);
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_host::TestHost;
    use crate::widget::{LayoutCx, PaintCx, Widget};
    use fret_core::{Px, Rect, Scene, Size, TextConstraints, TextMetrics, TextService, TextStyle};

    #[derive(Default)]
    struct FakeTextService;

    impl TextService for FakeTextService {
        fn prepare(
            &mut self,
            _text: &str,
            _style: TextStyle,
            _constraints: TextConstraints,
        ) -> (fret_core::TextBlobId, TextMetrics) {
            (
                fret_core::TextBlobId::default(),
                TextMetrics {
                    size: Size::new(Px(10.0), Px(10.0)),
                    baseline: Px(8.0),
                },
            )
        }

        fn release(&mut self, _blob: fret_core::TextBlobId) {}
    }

    struct Focusable;

    impl<H: UiHost> Widget<H> for Focusable {
        fn is_focusable(&self) -> bool {
            true
        }

        fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
            Size::new(Px(10.0), Px(10.0))
        }

        fn paint(&mut self, _cx: &mut PaintCx<'_, H>) {}
    }

    #[test]
    fn command_palette_open_focuses_first_focusable_descendant() {
        let mut host = TestHost::new();
        let mut text = FakeTextService::default();

        let window = AppWindowId::default();
        let mut ui = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(crate::widgets::Stack::new());
        ui.set_root(root);

        let mut overlays = WindowOverlays::install(&mut ui);

        let palette_root = overlays.command_palette_node();
        let content_root = ui.create_node(crate::widgets::Column::new());
        ui.add_child(palette_root, content_root);

        let focus_target = ui.create_node(Focusable);
        ui.add_child(content_root, focus_target);

        let handled = overlays.handle_command(
            &mut host,
            &mut ui,
            &mut text,
            window,
            &CommandId::from("command_palette.open"),
        );
        assert!(handled);
        assert_eq!(ui.focus(), Some(focus_target));

        let mut scene = Scene::default();
        ui.layout_all(
            &mut host,
            &mut text,
            Rect::new(
                fret_core::Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(800.0), Px(600.0)),
            ),
            1.0,
        );
        ui.paint_all(
            &mut host,
            &mut text,
            Rect::new(
                fret_core::Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(800.0), Px(600.0)),
            ),
            &mut scene,
            1.0,
        );
    }
}
