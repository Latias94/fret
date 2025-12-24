use crate::{
    UiHost,
    tree::{UiLayerId, UiTree},
    widgets::{
        ContextMenu, ContextMenuService, DialogOverlay, DialogService, Popover, PopoverService,
        TooltipOverlay,
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

    pub fn handle_command<H: UiHost>(
        &mut self,
        app: &mut H,
        ui: &mut UiTree<H>,
        text: &mut dyn TextService,
        window: AppWindowId,
        command: &CommandId,
    ) -> bool {
        match command.as_str() {
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
                ui.set_focus(Some(self.popover_node));
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
