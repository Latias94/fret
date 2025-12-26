use fret_core::{AppWindowId, NodeId, UiServices};
use fret_runtime::{CommandId, Effect};
use fret_ui::{
    UiHost,
    tree::{UiLayerId, UiTree},
};

use crate::{
    CommandPaletteOverlay, ContextMenu, ContextMenuService, DialogOverlay, DialogService, Popover,
    PopoverService, PopoverSurfaceOverlay, PopoverSurfaceService, SheetOverlay, SheetService,
    ToastOverlay, TooltipOverlay,
};

/// Standard window-level UI overlays (tooltips, popovers, context menus).
///
/// This helper exists to reduce per-app boilerplate: many apps want the same overlay widgets wired
/// into `UiTree` layers with consistent open/close + focus restoration behavior.
///
/// Design note:
/// - This type lives in `fret-components-ui` (not `fret-ui`) so higher-level policy and shadcn-style
///   composition can evolve without bloating the runtime crate (see ADR 0037, MVP 48).
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

    sheet_layer: UiLayerId,
    sheet_node: NodeId,
    sheet_previous_focus: Option<NodeId>,

    popover_layer: UiLayerId,
    popover_node: NodeId,
    popover_previous_focus: Option<NodeId>,

    popover_surface_layer: UiLayerId,
    popover_surface_node: NodeId,
    popover_surface_previous_focus: Option<NodeId>,

    context_menu_layer: UiLayerId,
    context_menu_node: NodeId,
    context_menu_previous_focus: Option<NodeId>,
}

impl WindowOverlays {
    pub fn install<H: UiHost>(ui: &mut UiTree<H>) -> Self {
        let tooltip_node = ui.create_node(TooltipOverlay::new());
        let tooltip_layer = ui.push_overlay_root_ex(tooltip_node, false, false);
        ui.set_layer_wants_pointer_move_events(tooltip_layer, true);

        // Toasts should be visually on top but pointer-transparent outside toast bounds.
        // The overlay is hit-testable and relies on per-widget hit testing (`Widget::hit_test`).
        let toast_node = ui.create_node(ToastOverlay::new());
        let toast_layer = ui.push_overlay_root_ex(toast_node, false, true);
        ui.set_layer_wants_timer_events(toast_layer, true);

        let command_palette_node = ui.create_node(CommandPaletteOverlay::new());
        let command_palette_layer = ui.push_overlay_root(command_palette_node, true);
        ui.set_layer_visible(command_palette_layer, false);

        let dialog_node = ui.create_node(DialogOverlay::new());
        let dialog_layer = ui.push_overlay_root(dialog_node, true);
        ui.set_layer_visible(dialog_layer, false);

        let sheet_node = ui.create_node(SheetOverlay::new());
        let sheet_layer = ui.push_overlay_root(sheet_node, true);
        ui.set_layer_visible(sheet_layer, false);

        let popover_node = ui.create_node(Popover::new());
        let popover_layer = ui.push_overlay_root(popover_node, true);
        ui.set_layer_visible(popover_layer, false);

        let popover_surface_node = ui.create_node(PopoverSurfaceOverlay::new());
        let popover_surface_layer = ui.push_overlay_root(popover_surface_node, true);
        ui.set_layer_visible(popover_surface_layer, false);

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
            sheet_layer,
            sheet_node,
            sheet_previous_focus: None,
            popover_layer,
            popover_node,
            popover_previous_focus: None,
            popover_surface_layer,
            popover_surface_node,
            popover_surface_previous_focus: None,
            context_menu_layer,
            context_menu_node,
            context_menu_previous_focus: None,
        }
    }

    pub fn command_palette_node(&self) -> NodeId {
        self.command_palette_node
    }

    pub fn sheet_node(&self) -> NodeId {
        self.sheet_node
    }

    pub fn popover_surface_node(&self) -> NodeId {
        self.popover_surface_node
    }

    pub fn handle_command<H: UiHost>(
        &mut self,
        app: &mut H,
        ui: &mut UiTree<H>,
        services: &mut dyn UiServices,
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
                    ui.cleanup_subtree(services, self.command_palette_node);
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
                        services,
                        window,
                        &CommandId::from("command_palette.close"),
                    );
                }
                self.handle_command(
                    app,
                    ui,
                    services,
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
                    ui.cleanup_subtree(services, self.dialog_node);
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
            "sheet.open" => {
                let has_request = app
                    .global::<SheetService>()
                    .and_then(|s| s.request(window))
                    .is_some();
                if !has_request {
                    return true;
                }

                let visible = ui.is_layer_visible(self.sheet_layer);
                if !visible {
                    self.sheet_previous_focus = ui.focus();
                    ui.set_layer_visible(self.sheet_layer, true);
                }
                let request_focus = app
                    .global::<SheetService>()
                    .and_then(|s| s.request(window))
                    .map(|(_, req)| req.request_focus)
                    .unwrap_or(true);
                if request_focus {
                    ui.set_focus(Some(self.sheet_node));
                }
                app.request_redraw(window);
                true
            }
            "sheet.close" => {
                if ui.is_layer_visible(self.sheet_layer) {
                    ui.cleanup_subtree(services, self.sheet_node);
                    ui.set_layer_visible(self.sheet_layer, false);
                }

                app.with_global_mut(SheetService::default, |service, _app| {
                    service.clear(window);
                });

                if let Some(prev) = self.sheet_previous_focus.take() {
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
                    ui.cleanup_subtree(services, self.popover_node);
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
            "popover_surface.open" => {
                let has_request = app
                    .global::<PopoverSurfaceService>()
                    .and_then(|s| s.request(window))
                    .is_some();
                if !has_request {
                    return true;
                }

                let visible = ui.is_layer_visible(self.popover_surface_layer);
                if !visible {
                    self.popover_surface_previous_focus = ui.focus();
                    ui.set_layer_visible(self.popover_surface_layer, true);
                }

                let request_focus = app
                    .global::<PopoverSurfaceService>()
                    .and_then(|s| s.request(window))
                    .map(|(_, req)| req.request_focus)
                    .unwrap_or(true);
                if request_focus {
                    let req = app
                        .global::<PopoverSurfaceService>()
                        .and_then(|s| s.request(window))
                        .map(|(_, req)| req.clone());
                    if let Some(req) = req {
                        let focus = ui
                            .first_focusable_descendant(req.content_node)
                            .unwrap_or(req.content_node);
                        ui.set_focus(Some(focus));
                    } else {
                        ui.set_focus(Some(self.popover_surface_node));
                    }
                }

                app.request_redraw(window);
                true
            }
            "popover_surface.close" => {
                if ui.is_layer_visible(self.popover_surface_layer) {
                    ui.set_layer_visible(self.popover_surface_layer, false);
                }

                app.with_global_mut(PopoverSurfaceService::default, |service, _app| {
                    service.clear_request(window);
                });

                if let Some(prev) = self.popover_surface_previous_focus.take() {
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
                    ui.cleanup_subtree(services, self.context_menu_node);
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
    use crate::PopoverSurfaceRequest;
    use crate::select::{Select, SelectOption};
    use fret_app::App;
    use fret_core::{
        Event, KeyCode, Modifiers, MouseButton, PathCommand, PathConstraints, PathId, PathMetrics,
        PathService, PathStyle, PointerEvent, Px, Rect, Scene, Size, TextService, geometry::Point,
    };
    use fret_runtime::{CommandId, CommandMeta, Effect, Menu, MenuItem, Model, WhenExpr};
    use fret_ui::widget::Invalidation as UiInvalidation;
    use fret_ui::{
        ContextMenuRequest, UiTree,
        widget::{LayoutCx, PaintCx, Widget},
    };

    #[derive(Default)]
    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _text: &str,
            _style: fret_core::TextStyle,
            _constraints: fret_core::TextConstraints,
        ) -> (fret_core::TextBlobId, fret_core::TextMetrics) {
            (
                fret_core::TextBlobId::default(),
                fret_core::TextMetrics {
                    size: Size::new(Px(10.0), Px(10.0)),
                    baseline: Px(8.0),
                },
            )
        }

        fn release(&mut self, _blob: fret_core::TextBlobId) {}
    }

    impl PathService for FakeServices {
        fn prepare(
            &mut self,
            _commands: &[PathCommand],
            _style: PathStyle,
            _constraints: PathConstraints,
        ) -> (PathId, PathMetrics) {
            (PathId::default(), PathMetrics::default())
        }

        fn release(&mut self, _path: PathId) {}
    }

    impl fret_core::SvgService for FakeServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
            fret_core::SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
            false
        }
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

    fn run_frame(ui: &mut UiTree<App>, host: &mut App, services: &mut FakeServices) {
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let mut scene = Scene::default();
        ui.layout_all(host, services, bounds, 1.0);
        ui.paint_all(host, services, bounds, &mut scene, 1.0);
    }

    fn pump_commands(
        overlays: &mut WindowOverlays,
        host: &mut App,
        ui: &mut UiTree<App>,
        services: &mut FakeServices,
        window: AppWindowId,
    ) -> Vec<CommandId> {
        let mut app_commands: Vec<CommandId> = Vec::new();
        loop {
            let effects = host.flush_effects();
            if effects.is_empty() {
                break;
            }
            for effect in effects {
                if let Effect::Command { command, .. } = effect {
                    let handled = overlays.handle_command(host, ui, services, window, &command);
                    if !handled {
                        app_commands.push(command);
                    }
                }
            }
        }
        app_commands
    }

    #[test]
    fn command_palette_open_focuses_first_focusable_descendant() {
        let mut host = App::new();
        let mut services = FakeServices::default();

        let window = AppWindowId::default();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(fret_ui::primitives::Stack::new());
        ui.set_root(root);

        let mut overlays = WindowOverlays::install(&mut ui);

        let palette_root = overlays.command_palette_node();
        let content_root = ui.create_node(fret_ui::primitives::Column::new());
        ui.add_child(palette_root, content_root);

        let focus_target = ui.create_node(Focusable);
        ui.add_child(content_root, focus_target);

        let handled = overlays.handle_command(
            &mut host,
            &mut ui,
            &mut services,
            window,
            &CommandId::from("command_palette.open"),
        );
        assert!(handled);
        assert_eq!(ui.focus(), Some(focus_target));

        let mut scene = Scene::default();
        ui.layout_all(
            &mut host,
            &mut services,
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(800.0), Px(600.0)),
            ),
            1.0,
        );
        ui.paint_all(
            &mut host,
            &mut services,
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(800.0), Px(600.0)),
            ),
            &mut scene,
            1.0,
        );
    }

    #[test]
    fn command_palette_escape_closes_and_restores_focus() {
        let mut host = App::new();
        let mut services = FakeServices::default();

        let window = AppWindowId::default();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(fret_ui::primitives::Stack::new());
        ui.set_root(root);

        let mut overlays = WindowOverlays::install(&mut ui);

        let prev_focus = ui.create_node(Focusable);
        ui.add_child(root, prev_focus);
        ui.set_focus(Some(prev_focus));

        let palette_root = overlays.command_palette_node();
        let content_root = ui.create_node(fret_ui::primitives::Column::new());
        ui.add_child(palette_root, content_root);

        let focus_target = ui.create_node(Focusable);
        ui.add_child(content_root, focus_target);

        let handled = overlays.handle_command(
            &mut host,
            &mut ui,
            &mut services,
            window,
            &CommandId::from("command_palette.open"),
        );
        assert!(handled);
        assert_eq!(ui.focus(), Some(focus_target));

        run_frame(&mut ui, &mut host, &mut services);

        ui.dispatch_event(
            &mut host,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::Escape,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        let app_commands = pump_commands(&mut overlays, &mut host, &mut ui, &mut services, window);
        assert!(app_commands.is_empty());
        assert_eq!(ui.focus(), Some(prev_focus));
    }

    #[test]
    fn command_palette_click_outside_closes_and_restores_focus() {
        let mut host = App::new();
        let mut services = FakeServices::default();

        let window = AppWindowId::default();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(fret_ui::primitives::Stack::new());
        ui.set_root(root);

        let mut overlays = WindowOverlays::install(&mut ui);

        let prev_focus = ui.create_node(Focusable);
        ui.add_child(root, prev_focus);
        ui.set_focus(Some(prev_focus));

        let palette_root = overlays.command_palette_node();
        let content_root = ui.create_node(fret_ui::primitives::Column::new());
        ui.add_child(palette_root, content_root);

        let focus_target = ui.create_node(Focusable);
        ui.add_child(content_root, focus_target);

        let handled = overlays.handle_command(
            &mut host,
            &mut ui,
            &mut services,
            window,
            &CommandId::from("command_palette.open"),
        );
        assert!(handled);
        run_frame(&mut ui, &mut host, &mut services);

        ui.dispatch_event(
            &mut host,
            &mut services,
            &Event::Pointer(PointerEvent::Down {
                position: Point::new(Px(0.0), Px(0.0)),
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
            }),
        );

        let app_commands = pump_commands(&mut overlays, &mut host, &mut ui, &mut services, window);
        assert!(app_commands.is_empty());
        assert_eq!(ui.focus(), Some(prev_focus));
    }

    #[test]
    fn dialog_escape_closes_dispatches_cancel_and_restores_focus() {
        let mut host = App::new();
        let mut services = FakeServices::default();

        let window = AppWindowId::default();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(fret_ui::primitives::Stack::new());
        ui.set_root(root);

        let mut overlays = WindowOverlays::install(&mut ui);

        let prev_focus = ui.create_node(Focusable);
        ui.add_child(root, prev_focus);
        ui.set_focus(Some(prev_focus));

        host.with_global_mut(DialogService::default, |service, _app| {
            service.set_request(
                window,
                crate::DialogRequest {
                    owner: prev_focus,
                    title: "Test".into(),
                    message: "Body".into(),
                    actions: vec![crate::DialogAction::cancel("Cancel")],
                    default_action: None,
                    cancel_command: Some(CommandId::from("test.dialog.cancel")),
                },
            );
        });

        let handled = overlays.handle_command(
            &mut host,
            &mut ui,
            &mut services,
            window,
            &CommandId::from("dialog.open"),
        );
        assert!(handled);
        assert_eq!(ui.focus(), Some(overlays.dialog_node));

        run_frame(&mut ui, &mut host, &mut services);

        ui.dispatch_event(
            &mut host,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::Escape,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        let app_commands = pump_commands(&mut overlays, &mut host, &mut ui, &mut services, window);
        assert_eq!(app_commands, vec![CommandId::from("test.dialog.cancel")]);
        assert_eq!(ui.focus(), Some(prev_focus));

        let still_open = host
            .global::<DialogService>()
            .and_then(|s| s.request(window))
            .is_some();
        assert!(!still_open, "expected dialog request to be cleared");
    }

    #[test]
    fn dialog_click_outside_closes_and_restores_focus() {
        let mut host = App::new();
        let mut services = FakeServices::default();

        let window = AppWindowId::default();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(fret_ui::primitives::Stack::new());
        ui.set_root(root);

        let mut overlays = WindowOverlays::install(&mut ui);

        let prev_focus = ui.create_node(Focusable);
        ui.add_child(root, prev_focus);
        ui.set_focus(Some(prev_focus));

        host.with_global_mut(DialogService::default, |service, _app| {
            service.set_request(
                window,
                crate::DialogRequest {
                    owner: prev_focus,
                    title: "Test".into(),
                    message: "Body".into(),
                    actions: vec![crate::DialogAction::cancel("Cancel")],
                    default_action: None,
                    cancel_command: Some(CommandId::from("test.dialog.cancel")),
                },
            );
        });

        let handled = overlays.handle_command(
            &mut host,
            &mut ui,
            &mut services,
            window,
            &CommandId::from("dialog.open"),
        );
        assert!(handled);
        run_frame(&mut ui, &mut host, &mut services);

        ui.dispatch_event(
            &mut host,
            &mut services,
            &Event::Pointer(PointerEvent::Down {
                position: Point::new(Px(0.0), Px(0.0)),
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
            }),
        );

        let app_commands = pump_commands(&mut overlays, &mut host, &mut ui, &mut services, window);
        assert_eq!(app_commands, vec![CommandId::from("test.dialog.cancel")]);
        assert_eq!(ui.focus(), Some(prev_focus));
    }

    #[test]
    fn sheet_escape_closes_and_restores_focus() {
        let mut host = App::new();
        let mut services = FakeServices::default();

        let window = AppWindowId::default();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(fret_ui::primitives::Stack::new());
        ui.set_root(root);

        let mut overlays = WindowOverlays::install(&mut ui);

        let prev_focus = ui.create_node(Focusable);
        ui.add_child(root, prev_focus);
        ui.set_focus(Some(prev_focus));

        host.with_global_mut(SheetService::default, |service, _app| {
            service.set_request(window, crate::SheetRequest::new(prev_focus));
        });

        let handled = overlays.handle_command(
            &mut host,
            &mut ui,
            &mut services,
            window,
            &CommandId::from("sheet.open"),
        );
        assert!(handled);
        assert_eq!(ui.focus(), Some(overlays.sheet_node));

        run_frame(&mut ui, &mut host, &mut services);

        ui.dispatch_event(
            &mut host,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::Escape,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        let app_commands = pump_commands(&mut overlays, &mut host, &mut ui, &mut services, window);
        assert!(app_commands.is_empty());
        assert_eq!(ui.focus(), Some(prev_focus));

        let still_open = host
            .global::<SheetService>()
            .and_then(|s| s.request(window))
            .is_some();
        assert!(!still_open, "expected sheet request to be cleared");
    }

    #[test]
    fn sheet_click_outside_closes_and_restores_focus() {
        let mut host = App::new();
        let mut services = FakeServices::default();

        let window = AppWindowId::default();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(fret_ui::primitives::Stack::new());
        ui.set_root(root);

        let mut overlays = WindowOverlays::install(&mut ui);

        let prev_focus = ui.create_node(Focusable);
        ui.add_child(root, prev_focus);
        ui.set_focus(Some(prev_focus));

        host.with_global_mut(SheetService::default, |service, _app| {
            service.set_request(
                window,
                crate::SheetRequest::new(prev_focus).side(crate::SheetSide::Right),
            );
        });

        let handled = overlays.handle_command(
            &mut host,
            &mut ui,
            &mut services,
            window,
            &CommandId::from("sheet.open"),
        );
        assert!(handled);
        run_frame(&mut ui, &mut host, &mut services);

        ui.dispatch_event(
            &mut host,
            &mut services,
            &Event::Pointer(PointerEvent::Down {
                position: Point::new(Px(0.0), Px(0.0)),
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
            }),
        );

        let app_commands = pump_commands(&mut overlays, &mut host, &mut ui, &mut services, window);
        assert!(app_commands.is_empty());
        assert_eq!(ui.focus(), Some(prev_focus));
    }

    #[test]
    fn context_menu_arrow_keys_skip_disabled_and_space_activates() {
        let mut host = App::new();
        let mut services = FakeServices::default();

        let window = AppWindowId::default();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(fret_ui::primitives::Stack::new());
        ui.set_root(root);

        let mut overlays = WindowOverlays::install(&mut ui);

        let trigger = ui.create_node(Focusable);
        ui.add_child(root, trigger);
        ui.set_focus(Some(trigger));

        let cmd_a = CommandId::from("test.menu.a");
        let cmd_b = CommandId::from("test.menu.b");
        let cmd_c = CommandId::from("test.menu.c");

        host.commands_mut().register(
            cmd_a.clone(),
            CommandMeta::new("B").with_when(WhenExpr::parse("false").unwrap()),
        );

        let menu = Menu {
            title: "Menu".into(),
            items: vec![
                MenuItem::Command {
                    command: cmd_a.clone(),
                    when: None,
                },
                MenuItem::Command {
                    command: cmd_b.clone(),
                    when: None,
                },
                MenuItem::Command {
                    command: cmd_c.clone(),
                    when: None,
                },
            ],
        };

        host.with_global_mut(ContextMenuService::default, |service, _app| {
            service.set_request(
                window,
                ContextMenuRequest {
                    position: Point::new(Px(10.0), Px(10.0)),
                    menu,
                    input_ctx: Default::default(),
                    menu_bar: None,
                },
            );
        });

        let handled = overlays.handle_command(
            &mut host,
            &mut ui,
            &mut services,
            window,
            &CommandId::from("context_menu.open"),
        );
        assert!(handled);
        assert_eq!(ui.focus(), Some(overlays.context_menu_node));

        run_frame(&mut ui, &mut host, &mut services);

        ui.dispatch_event(
            &mut host,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::ArrowDown,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        ui.dispatch_event(
            &mut host,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::Space,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        let app_commands = pump_commands(&mut overlays, &mut host, &mut ui, &mut services, window);
        assert_eq!(app_commands, vec![cmd_b]);
        assert_eq!(ui.focus(), Some(trigger));
    }

    #[test]
    fn context_menu_home_end_select_first_and_last_enabled() {
        let mut host = App::new();
        let mut services = FakeServices::default();

        let window = AppWindowId::default();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(fret_ui::primitives::Stack::new());
        ui.set_root(root);

        let mut overlays = WindowOverlays::install(&mut ui);

        let trigger = ui.create_node(Focusable);
        ui.add_child(root, trigger);
        ui.set_focus(Some(trigger));

        let cmd_a = CommandId::from("test.menu.a");
        let cmd_b = CommandId::from("test.menu.b");
        let cmd_c = CommandId::from("test.menu.c");

        host.commands_mut().register(
            cmd_a.clone(),
            CommandMeta::new("B").with_when(WhenExpr::parse("false").unwrap()),
        );

        let menu = Menu {
            title: "Menu".into(),
            items: vec![
                MenuItem::Command {
                    command: cmd_a.clone(),
                    when: None,
                },
                MenuItem::Command {
                    command: cmd_b.clone(),
                    when: None,
                },
                MenuItem::Command {
                    command: cmd_c.clone(),
                    when: None,
                },
            ],
        };

        host.with_global_mut(ContextMenuService::default, |service, _app| {
            service.set_request(
                window,
                ContextMenuRequest {
                    position: Point::new(Px(10.0), Px(10.0)),
                    menu: menu.clone(),
                    input_ctx: Default::default(),
                    menu_bar: None,
                },
            );
        });

        let handled = overlays.handle_command(
            &mut host,
            &mut ui,
            &mut services,
            window,
            &CommandId::from("context_menu.open"),
        );
        assert!(handled);
        assert_eq!(ui.focus(), Some(overlays.context_menu_node));

        run_frame(&mut ui, &mut host, &mut services);

        ui.dispatch_event(
            &mut host,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::End,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        ui.dispatch_event(
            &mut host,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::Space,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        let app_commands = pump_commands(&mut overlays, &mut host, &mut ui, &mut services, window);
        assert_eq!(app_commands, vec![cmd_c.clone()]);
        assert_eq!(ui.focus(), Some(trigger));

        host.with_global_mut(ContextMenuService::default, |service, _app| {
            service.set_request(
                window,
                ContextMenuRequest {
                    position: Point::new(Px(10.0), Px(10.0)),
                    menu,
                    input_ctx: Default::default(),
                    menu_bar: None,
                },
            );
        });

        let handled = overlays.handle_command(
            &mut host,
            &mut ui,
            &mut services,
            window,
            &CommandId::from("context_menu.open"),
        );
        assert!(handled);
        assert_eq!(ui.focus(), Some(overlays.context_menu_node));

        run_frame(&mut ui, &mut host, &mut services);

        ui.dispatch_event(
            &mut host,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::Home,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        ui.dispatch_event(
            &mut host,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::Space,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        let app_commands = pump_commands(&mut overlays, &mut host, &mut ui, &mut services, window);
        assert_eq!(app_commands, vec![cmd_b]);
        assert_eq!(ui.focus(), Some(trigger));
    }

    #[test]
    fn select_keyboard_opens_popover_and_selects_enabled_item() {
        let mut host = App::new();
        let mut services = FakeServices::default();

        let window = AppWindowId::default();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(fret_ui::primitives::Stack::new());
        ui.set_root(root);

        let mut overlays = WindowOverlays::install(&mut ui);

        let model: Model<usize> = host.models_mut().insert(0usize);
        let select = ui.create_node(Select::new(
            model,
            vec![
                SelectOption::new("A").disabled(),
                SelectOption::new("B"),
                SelectOption::new("C"),
            ],
        ));
        ui.add_child(root, select);
        ui.set_focus(Some(select));

        run_frame(&mut ui, &mut host, &mut services);

        ui.dispatch_event(
            &mut host,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::Enter,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        let _ = pump_commands(&mut overlays, &mut host, &mut ui, &mut services, window);
        assert_eq!(ui.focus(), Some(overlays.popover_node));

        run_frame(&mut ui, &mut host, &mut services);

        ui.dispatch_event(
            &mut host,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::ArrowDown,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        ui.dispatch_event(
            &mut host,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::Enter,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        let _ = pump_commands(&mut overlays, &mut host, &mut ui, &mut services, window);
        assert_eq!(ui.focus(), Some(select));

        ui.invalidate(select, UiInvalidation::Layout);
        run_frame(&mut ui, &mut host, &mut services);
        assert_eq!(host.models().get(model).copied(), Some(1));
    }

    #[test]
    fn popover_surface_open_focuses_first_focusable_descendant_and_close_restores_focus() {
        let mut host = App::new();
        let mut services = FakeServices::default();

        let window = AppWindowId::default();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(fret_ui::primitives::Stack::new());
        ui.set_root(root);

        let mut overlays = WindowOverlays::install(&mut ui);

        let prev_focus = ui.create_node(Focusable);
        ui.add_child(root, prev_focus);
        ui.set_focus(Some(prev_focus));

        let surface_root = overlays.popover_surface_node();
        let content_root = ui.create_node(fret_ui::primitives::Column::new());
        ui.add_child(surface_root, content_root);

        let focus_target = ui.create_node(Focusable);
        ui.add_child(content_root, focus_target);

        host.with_global_mut(PopoverSurfaceService::default, |service, _app| {
            service.set_request(
                window,
                PopoverSurfaceRequest::new(
                    prev_focus,
                    Rect::new(
                        Point::new(Px(10.0), Px(10.0)),
                        Size::new(Px(20.0), Px(20.0)),
                    ),
                    content_root,
                ),
            );
        });

        let handled = overlays.handle_command(
            &mut host,
            &mut ui,
            &mut services,
            window,
            &CommandId::from("popover_surface.open"),
        );
        assert!(handled);
        assert_eq!(ui.focus(), Some(focus_target));

        let mut scene = Scene::default();
        ui.layout_all(
            &mut host,
            &mut services,
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(800.0), Px(600.0)),
            ),
            1.0,
        );
        ui.paint_all(
            &mut host,
            &mut services,
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(800.0), Px(600.0)),
            ),
            &mut scene,
            1.0,
        );

        let handled = overlays.handle_command(
            &mut host,
            &mut ui,
            &mut services,
            window,
            &CommandId::from("popover_surface.close"),
        );
        assert!(handled);
        assert_eq!(ui.focus(), Some(prev_focus));

        let still_open = host
            .global::<PopoverSurfaceService>()
            .and_then(|s| s.request(window))
            .is_some();
        assert!(
            !still_open,
            "expected popover surface request to be cleared"
        );
    }
}
