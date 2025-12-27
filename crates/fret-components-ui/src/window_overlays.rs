use fret_core::{AppWindowId, NodeId, UiServices};
use fret_runtime::{CommandId, Effect};
use fret_ui::{UiHost, tree::UiTree};
use std::cell::RefCell;
use std::rc::Rc;

use crate::context_menu::{CONTEXT_MENU_A11Y_SLOTS, ContextMenuA11yItem, ContextMenuA11yState};
use crate::overlay_policy::{ModalFocusScope, OverlayFocusTarget, OverlayPortal};
use crate::popover::{POPOVER_A11Y_SLOTS, PopoverA11yItem, PopoverA11yState};
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
    command_palette: OverlayPortal,
    dialog: OverlayPortal,
    sheet: OverlayPortal,
    popover: OverlayPortal,
    popover_surface: OverlayPortal,
    context_menu: OverlayPortal,
    modal_focus_scopes: Vec<ModalFocusScope>,
}

impl WindowOverlays {
    pub fn install<H: UiHost>(ui: &mut UiTree<H>) -> Self {
        let tooltip = OverlayPortal::install(ui, TooltipOverlay::new(), false, false, true);
        ui.set_layer_wants_pointer_move_events(tooltip.layer, true);

        // Toasts should be visually on top but pointer-transparent outside toast bounds.
        // The overlay is hit-testable and relies on per-widget hit testing (`Widget::hit_test`).
        let mut toast = OverlayPortal::install(ui, ToastOverlay::new(), false, true, true);
        toast.cleanup_on_close = false;
        ui.set_layer_wants_timer_events(toast.layer, true);

        let command_palette =
            OverlayPortal::install(ui, CommandPaletteOverlay::new(), true, true, false);

        let dialog = OverlayPortal::install(ui, DialogOverlay::new(), true, true, false);

        let sheet = OverlayPortal::install(ui, SheetOverlay::new(), true, true, false);

        // Popovers are non-modal (click-through): they must not install a modal barrier.
        let popover_a11y = Rc::new(RefCell::new(PopoverA11yState::new(POPOVER_A11Y_SLOTS)));
        let popover = OverlayPortal::install(
            ui,
            Popover::new_with_a11y(popover_a11y.clone()),
            false,
            true,
            false,
        );
        let popover_a11y_nodes: Vec<NodeId> = (0..POPOVER_A11Y_SLOTS)
            .map(|slot| ui.create_node(PopoverA11yItem::new(slot, popover_a11y.clone())))
            .collect();
        ui.set_children(popover.root, popover_a11y_nodes);
        ui.set_layer_wants_pointer_down_outside_events(popover.layer, true);

        let mut popover_surface =
            OverlayPortal::install(ui, PopoverSurfaceOverlay::new(), false, true, false);
        popover_surface.cleanup_on_close = false;
        ui.set_layer_wants_pointer_down_outside_events(popover_surface.layer, true);

        // Context menus are non-modal (click-through) but dismiss on outside press.
        let context_menu_a11y = Rc::new(RefCell::new(ContextMenuA11yState::new(
            CONTEXT_MENU_A11Y_SLOTS,
        )));
        let context_menu = OverlayPortal::install(
            ui,
            ContextMenu::new_with_a11y(context_menu_a11y.clone()),
            false,
            true,
            false,
        );
        let a11y_nodes: Vec<NodeId> = (0..CONTEXT_MENU_A11Y_SLOTS)
            .map(|slot| ui.create_node(ContextMenuA11yItem::new(slot, context_menu_a11y.clone())))
            .collect();
        ui.set_children(context_menu.root, a11y_nodes);
        ui.set_layer_wants_pointer_down_outside_events(context_menu.layer, true);

        Self {
            command_palette,
            dialog,
            sheet,
            popover,
            popover_surface,
            context_menu,
            modal_focus_scopes: Vec::new(),
        }
    }

    pub fn command_palette_node(&self) -> NodeId {
        self.command_palette.root
    }

    pub fn sheet_node(&self) -> NodeId {
        self.sheet.root
    }

    pub fn popover_surface_node(&self) -> NodeId {
        self.popover_surface.root
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
            "focus.next" | "focus.previous" => {
                let Some(scope) = self.modal_focus_scopes.last() else {
                    return false;
                };

                let forward = command.as_str() == "focus.next";
                let roots = scope.traversal_roots_in_ui(ui);
                let _ = ui.focus_traverse_in_roots(app, &roots, forward, Some(scope.root()));
                true
            }
            "command_palette.open" => {
                let was_visible = self.command_palette.is_visible(ui);
                let content_root = ui
                    .children(self.command_palette.root)
                    .first()
                    .copied()
                    .unwrap_or(self.command_palette.root);
                self.command_palette.show_with_focus(
                    ui,
                    OverlayFocusTarget::FirstFocusableDescendant {
                        root: content_root,
                        fallback: self.command_palette.root,
                    },
                );
                if !was_visible {
                    self.modal_focus_scopes
                        .push(ModalFocusScope::new(self.command_palette.root));
                }
                app.request_redraw(window);
                true
            }
            "command_palette.close" => {
                self.command_palette.hide(ui, services);
                self.modal_focus_scopes
                    .retain(|s| s.root() != self.command_palette.root);

                app.request_redraw(window);
                true
            }
            "command_palette.toggle" => {
                if self.command_palette.is_visible(ui) {
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

                let was_visible = self.dialog.is_visible(ui);
                self.dialog.show_with_focus(ui, OverlayFocusTarget::Root);
                if !was_visible {
                    self.modal_focus_scopes
                        .push(ModalFocusScope::new(self.dialog.root));
                }
                app.request_redraw(window);
                true
            }
            "dialog.close" => {
                self.dialog.hide(ui, services);
                self.modal_focus_scopes
                    .retain(|s| s.root() != self.dialog.root);

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

                let request_focus = app
                    .global::<SheetService>()
                    .and_then(|s| s.request(window))
                    .map(|(_, req)| req.request_focus)
                    .unwrap_or(true);
                let was_visible = self.sheet.is_visible(ui);
                self.sheet.show_with_focus(
                    ui,
                    if request_focus {
                        OverlayFocusTarget::Root
                    } else {
                        OverlayFocusTarget::None
                    },
                );
                if !was_visible {
                    self.modal_focus_scopes
                        .push(ModalFocusScope::new(self.sheet.root));
                }
                app.request_redraw(window);
                true
            }
            "sheet.close" => {
                self.sheet.hide(ui, services);
                self.modal_focus_scopes
                    .retain(|s| s.root() != self.sheet.root);

                app.with_global_mut(SheetService::default, |service, _app| {
                    service.clear(window);
                });

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

                let request_focus = app
                    .global::<PopoverService>()
                    .and_then(|s| s.request(window))
                    .map(|(_, req)| req.request_focus)
                    .unwrap_or(true);
                self.popover.show_with_focus(
                    ui,
                    if request_focus {
                        OverlayFocusTarget::Root
                    } else {
                        OverlayFocusTarget::None
                    },
                );
                if let Some(scope) = self.modal_focus_scopes.last_mut() {
                    scope.allow_portal(self.popover.root);
                }
                app.request_redraw(window);
                true
            }
            "popover.close" => {
                self.popover.hide(ui, services);
                for scope in &mut self.modal_focus_scopes {
                    scope.disallow_portal(self.popover.root);
                }

                app.with_global_mut(PopoverService::default, |service, _app| {
                    service.clear_request(window);
                });

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

                let request_focus = app
                    .global::<PopoverSurfaceService>()
                    .and_then(|s| s.request(window))
                    .map(|(_, req)| req.request_focus)
                    .unwrap_or(true);
                let req = app
                    .global::<PopoverSurfaceService>()
                    .and_then(|s| s.request(window))
                    .map(|(_, req)| req.clone());
                let focus_target = if !request_focus {
                    OverlayFocusTarget::None
                } else if let Some(req) = req {
                    OverlayFocusTarget::FirstFocusableDescendant {
                        root: req.content_node,
                        fallback: req.content_node,
                    }
                } else {
                    OverlayFocusTarget::Root
                };
                self.popover_surface.show_with_focus(ui, focus_target);
                if let Some(scope) = self.modal_focus_scopes.last_mut() {
                    scope.allow_portal(self.popover_surface.root);
                }

                app.request_redraw(window);
                true
            }
            "popover_surface.close" => {
                self.popover_surface.hide(ui, services);
                for scope in &mut self.modal_focus_scopes {
                    scope.disallow_portal(self.popover_surface.root);
                }

                app.with_global_mut(PopoverSurfaceService::default, |service, _app| {
                    service.clear_request(window);
                });

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

                self.context_menu
                    .show_with_focus(ui, OverlayFocusTarget::Root);
                if let Some(scope) = self.modal_focus_scopes.last_mut() {
                    scope.allow_portal(self.context_menu.root);
                }
                app.request_redraw(window);
                true
            }
            "context_menu.close" => {
                self.context_menu.hide(ui, services);
                for scope in &mut self.modal_focus_scopes {
                    scope.disallow_portal(self.context_menu.root);
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
    use crate::ContextMenuRequest;
    use crate::PopoverSurfaceRequest;
    use crate::combobox::Combobox;
    use crate::select::{Select, SelectOption};
    use crate::{PopoverItem, PopoverRequest};
    use fret_app::App;
    use fret_core::{
        Event, KeyCode, Modifiers, MouseButton, PathCommand, PathConstraints, PathId, PathMetrics,
        PathService, PathStyle, PointerEvent, Px, Rect, Scene, SemanticsRole, Size, TextService,
        geometry::Point,
    };
    use fret_runtime::{CommandId, CommandMeta, Effect, Menu, MenuItem, Model, WhenExpr};
    use fret_ui::widget::Invalidation as UiInvalidation;
    use fret_ui::{
        UiHost, UiTree,
        widget::{LayoutCx, PaintCx, Widget},
    };

    #[derive(Default)]
    struct FakeServices(());

    #[derive(Debug, Default)]
    struct TestContainer;

    impl<H: UiHost> Widget<H> for TestContainer {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            for &child in cx.children {
                cx.layout_in(child, cx.bounds);
            }
            cx.available
        }

        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            for &child in cx.children {
                cx.paint(child, cx.bounds);
            }
        }
    }

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

    #[derive(Debug, Default)]
    struct TwoBoxContainer;

    impl<H: UiHost> Widget<H> for TwoBoxContainer {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            for (i, &child) in cx.children.iter().enumerate() {
                let origin_x = match i {
                    0 => 0.0,
                    1 => 20.0,
                    _ => 0.0,
                };
                let rect = Rect::new(
                    Point::new(Px(origin_x), Px(0.0)),
                    Size::new(Px(10.0), Px(10.0)),
                );
                cx.layout_in(child, rect);
            }
            cx.available
        }

        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            for (i, &child) in cx.children.iter().enumerate() {
                let origin_x = match i {
                    0 => 0.0,
                    1 => 20.0,
                    _ => 0.0,
                };
                let rect = Rect::new(
                    Point::new(Px(origin_x), Px(0.0)),
                    Size::new(Px(10.0), Px(10.0)),
                );
                cx.paint(child, rect);
            }
        }
    }

    #[derive(Debug, Default)]
    struct FocusOnPointerDown;

    impl<H: UiHost> Widget<H> for FocusOnPointerDown {
        fn is_focusable(&self) -> bool {
            true
        }

        fn event(&mut self, cx: &mut fret_ui::EventCx<'_, H>, event: &Event) {
            if let Event::Pointer(PointerEvent::Down { position, .. }) = event
                && cx.bounds.contains(*position)
            {
                cx.request_focus(cx.node);
            }
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

        let root = ui.create_node(TestContainer);
        ui.set_root(root);

        let mut overlays = WindowOverlays::install(&mut ui);

        let palette_root = overlays.command_palette_node();
        let content_root = ui.create_node(TestContainer);
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
    fn popover_click_outside_is_click_through_and_does_not_override_focused_target() {
        let mut host = App::new();
        let mut services = FakeServices::default();

        let window = AppWindowId::default();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(TwoBoxContainer);
        ui.set_root(root);

        let trigger = ui.create_node(Focusable);
        let other = ui.create_node(FocusOnPointerDown);
        ui.add_child(root, trigger);
        ui.add_child(root, other);

        ui.set_focus(Some(trigger));

        let mut overlays = WindowOverlays::install(&mut ui);
        run_frame(&mut ui, &mut host, &mut services);

        let anchor = ui
            .debug_node_bounds(trigger)
            .expect("expected trigger bounds");
        host.with_global_mut(PopoverService::default, |service, _app| {
            service.set_request(
                window,
                PopoverRequest {
                    owner: trigger,
                    anchor,
                    items: vec![PopoverItem::new("A")],
                    selected: None,
                    request_focus: true,
                },
            );
        });

        let handled = overlays.handle_command(
            &mut host,
            &mut ui,
            &mut services,
            window,
            &CommandId::from("popover.open"),
        );
        assert!(handled);
        run_frame(&mut ui, &mut host, &mut services);
        assert!(ui.is_layer_visible(overlays.popover.layer));

        // Click the "other" widget. This should both:
        // - dismiss the non-modal popover via outside-press observer,
        // - focus the hit-tested target via normal dispatch.
        ui.dispatch_event(
            &mut host,
            &mut services,
            &Event::Pointer(PointerEvent::Down {
                position: Point::new(Px(25.0), Px(5.0)),
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
            }),
        );

        let _ = pump_commands(&mut overlays, &mut host, &mut ui, &mut services, window);
        assert!(!ui.is_layer_visible(overlays.popover.layer));
        assert_eq!(ui.focus(), Some(other));
    }

    #[test]
    fn popover_semantics_exposes_list_items() {
        let mut host = App::new();
        let mut services = FakeServices::default();

        let window = AppWindowId::default();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(TwoBoxContainer);
        ui.set_root(root);

        let trigger = ui.create_node(Focusable);
        let other = ui.create_node(Focusable);
        ui.add_child(root, trigger);
        ui.add_child(root, other);

        ui.set_focus(Some(trigger));

        let mut overlays = WindowOverlays::install(&mut ui);
        run_frame(&mut ui, &mut host, &mut services);

        let anchor = ui
            .debug_node_bounds(trigger)
            .expect("expected trigger bounds");
        host.with_global_mut(PopoverService::default, |service, _app| {
            service.set_request(
                window,
                PopoverRequest {
                    owner: trigger,
                    anchor,
                    items: vec![
                        PopoverItem::new("A"),
                        PopoverItem::new("B"),
                        PopoverItem::new("C").disabled(),
                    ],
                    selected: Some(1),
                    request_focus: true,
                },
            );
        });

        let handled = overlays.handle_command(
            &mut host,
            &mut ui,
            &mut services,
            window,
            &CommandId::from("popover.open"),
        );
        assert!(handled);
        run_frame(&mut ui, &mut host, &mut services);

        ui.request_semantics_snapshot();
        run_frame(&mut ui, &mut host, &mut services);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let popover_root = snap
            .nodes
            .iter()
            .find(|n| n.id == overlays.popover.root)
            .expect("popover semantics node");
        assert_eq!(popover_root.role, SemanticsRole::List);

        let a = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ListItem && n.label.as_deref() == Some("A"))
            .expect("A item semantics node");
        assert!(!a.flags.disabled);
        assert!(!a.flags.selected);

        let b = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ListItem && n.label.as_deref() == Some("B"))
            .expect("B item semantics node");
        assert!(!b.flags.disabled);
        assert!(b.flags.selected);

        let c = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ListItem && n.label.as_deref() == Some("C"))
            .expect("C item semantics node");
        assert!(c.flags.disabled);
        assert!(!c.flags.selected);
    }

    #[test]
    fn combobox_semantics_role_and_expanded_follow_popover() {
        let mut host = App::new();
        let mut services = FakeServices::default();

        let window = AppWindowId::default();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(TestContainer);
        ui.set_root(root);

        let items = host
            .models_mut()
            .insert(vec!["Apple".to_string(), "Banana".to_string()]);
        let selection = host.models_mut().insert::<Option<usize>>(None);
        let query = host.models_mut().insert("ap".to_string());

        let combobox = ui.create_node(Combobox::new(items, selection, query));
        ui.add_child(root, combobox);
        ui.set_focus(Some(combobox));

        let mut overlays = WindowOverlays::install(&mut ui);
        run_frame(&mut ui, &mut host, &mut services);

        // Open popover explicitly to focus this test on semantics wiring.
        let anchor = ui
            .debug_node_bounds(combobox)
            .expect("expected combobox bounds");
        host.with_global_mut(PopoverService::default, |service, _app| {
            service.set_request(
                window,
                PopoverRequest {
                    owner: combobox,
                    anchor,
                    items: vec![PopoverItem::new("Apple")],
                    selected: Some(0),
                    request_focus: false,
                },
            );
        });
        let handled = overlays.handle_command(
            &mut host,
            &mut ui,
            &mut services,
            window,
            &CommandId::from("popover.open"),
        );
        assert!(handled);
        run_frame(&mut ui, &mut host, &mut services);

        ui.request_semantics_snapshot();
        run_frame(&mut ui, &mut host, &mut services);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let node = snap
            .nodes
            .iter()
            .find(|n| n.id == combobox)
            .expect("combobox semantics node");
        assert_eq!(node.role, SemanticsRole::ComboBox);
        assert_eq!(node.value.as_deref(), Some("ap"));
        assert!(node.flags.expanded);

        // Close explicitly.
        let handled = overlays.handle_command(
            &mut host,
            &mut ui,
            &mut services,
            window,
            &CommandId::from("popover.close"),
        );
        assert!(handled);
        run_frame(&mut ui, &mut host, &mut services);

        ui.request_semantics_snapshot();
        run_frame(&mut ui, &mut host, &mut services);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let node = snap
            .nodes
            .iter()
            .find(|n| n.id == combobox)
            .expect("combobox semantics node");
        assert_eq!(node.role, SemanticsRole::ComboBox);
        assert_eq!(node.value.as_deref(), Some("ap"));
        assert!(!node.flags.expanded);
    }

    #[test]
    fn command_palette_escape_closes_and_restores_focus() {
        let mut host = App::new();
        let mut services = FakeServices::default();

        let window = AppWindowId::default();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(TestContainer);
        ui.set_root(root);

        let mut overlays = WindowOverlays::install(&mut ui);

        let prev_focus = ui.create_node(Focusable);
        ui.add_child(root, prev_focus);
        ui.set_focus(Some(prev_focus));

        let palette_root = overlays.command_palette_node();
        let content_root = ui.create_node(TestContainer);
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

        let root = ui.create_node(TestContainer);
        ui.set_root(root);

        let mut overlays = WindowOverlays::install(&mut ui);

        let prev_focus = ui.create_node(Focusable);
        ui.add_child(root, prev_focus);
        ui.set_focus(Some(prev_focus));

        let palette_root = overlays.command_palette_node();
        let content_root = ui.create_node(TestContainer);
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

        let root = ui.create_node(TestContainer);
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
        assert_eq!(ui.focus(), Some(overlays.dialog.root));

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

        let root = ui.create_node(TestContainer);
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

        let root = ui.create_node(TestContainer);
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
        assert_eq!(ui.focus(), Some(overlays.sheet.root));

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

        let root = ui.create_node(TestContainer);
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
    fn focus_traversal_is_trapped_in_topmost_modal_overlay() {
        let mut host = App::new();
        let mut services = FakeServices::default();

        let window = AppWindowId::default();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(TestContainer);
        ui.set_root(root);

        let mut overlays = WindowOverlays::install(&mut ui);

        // Command palette modal content (has a focusable descendant).
        let palette_root = overlays.command_palette_node();
        let palette_content_root = ui.create_node(TestContainer);
        ui.add_child(palette_root, palette_content_root);
        let palette_focus = ui.create_node(Focusable);
        ui.add_child(palette_content_root, palette_focus);

        // Sheet modal content (two focusables so we can observe traversal).
        let sheet_root = overlays.sheet_node();
        let sheet_content_root = ui.create_node(TestContainer);
        ui.add_child(sheet_root, sheet_content_root);
        let sheet_focus_a = ui.create_node(Focusable);
        let sheet_focus_b = ui.create_node(Focusable);
        ui.add_child(sheet_content_root, sheet_focus_a);
        ui.add_child(sheet_content_root, sheet_focus_b);

        // Open command palette, then open sheet. Both are "modal" layers in the runtime sense.
        let handled = overlays.handle_command(
            &mut host,
            &mut ui,
            &mut services,
            window,
            &CommandId::from("command_palette.open"),
        );
        assert!(handled);
        assert_eq!(ui.focus(), Some(palette_focus));

        host.with_global_mut(SheetService::default, |service, _app| {
            service.set_request(window, crate::SheetRequest::new(palette_focus));
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

        // Focus traversal must be trapped in the topmost modal (sheet), not the earlier modal (palette).
        let handled = overlays.handle_command(
            &mut host,
            &mut ui,
            &mut services,
            window,
            &CommandId::from("focus.next"),
        );
        assert!(handled);
        assert_eq!(ui.focus(), Some(sheet_focus_a));

        let handled = overlays.handle_command(
            &mut host,
            &mut ui,
            &mut services,
            window,
            &CommandId::from("focus.next"),
        );
        assert!(handled);
        assert_eq!(ui.focus(), Some(sheet_focus_b));

        let handled = overlays.handle_command(
            &mut host,
            &mut ui,
            &mut services,
            window,
            &CommandId::from("focus.next"),
        );
        assert!(handled);
        assert_eq!(ui.focus(), Some(sheet_focus_a));

        assert_ne!(ui.focus(), Some(palette_focus));
    }

    #[test]
    fn context_menu_initial_selection_skips_disabled_and_space_activates() {
        let mut host = App::new();
        let mut services = FakeServices::default();

        let window = AppWindowId::default();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(TestContainer);
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
        assert_eq!(ui.focus(), Some(overlays.context_menu.root));

        run_frame(&mut ui, &mut host, &mut services);
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
    fn context_menu_semantics_exposes_menu_items() {
        let mut host = App::new();
        let mut services = FakeServices::default();

        let window = AppWindowId::default();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(TestContainer);
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
            CommandMeta::new("A").with_when(WhenExpr::parse("false").unwrap()),
        );
        host.commands_mut()
            .register(cmd_b.clone(), CommandMeta::new("B"));
        host.commands_mut()
            .register(cmd_c.clone(), CommandMeta::new("C"));

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
        run_frame(&mut ui, &mut host, &mut services);

        ui.request_semantics_snapshot();
        run_frame(&mut ui, &mut host, &mut services);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let menu_root = snap
            .nodes
            .iter()
            .find(|n| n.id == overlays.context_menu.root)
            .expect("context menu semantics node");
        assert_eq!(menu_root.role, SemanticsRole::Menu);

        let a = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("A"))
            .expect("A menu item semantics node");
        assert!(a.flags.disabled);
        assert!(!a.flags.selected);

        let b = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("B"))
            .expect("B menu item semantics node");
        assert!(!b.flags.disabled);
        assert!(b.flags.selected);

        let c = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("C"))
            .expect("C menu item semantics node");
        assert!(!c.flags.disabled);
        assert!(!c.flags.selected);
    }

    #[test]
    fn context_menu_home_end_select_first_and_last_enabled() {
        let mut host = App::new();
        let mut services = FakeServices::default();

        let window = AppWindowId::default();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(TestContainer);
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
        assert_eq!(ui.focus(), Some(overlays.context_menu.root));

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
        assert_eq!(ui.focus(), Some(overlays.context_menu.root));

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
    fn context_menu_typeahead_selects_item_and_activates() {
        let mut host = App::new();
        let mut services = FakeServices::default();

        let window = AppWindowId::default();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(TestContainer);
        ui.set_root(root);

        let mut overlays = WindowOverlays::install(&mut ui);

        let trigger = ui.create_node(Focusable);
        ui.add_child(root, trigger);
        ui.set_focus(Some(trigger));

        let cmd_a = CommandId::from("test.menu.alpha");
        let cmd_b = CommandId::from("test.menu.bravo");
        let cmd_c = CommandId::from("test.menu.charlie");

        host.commands_mut()
            .register(cmd_a.clone(), CommandMeta::new("Alpha"));
        host.commands_mut()
            .register(cmd_b.clone(), CommandMeta::new("Bravo"));
        host.commands_mut()
            .register(cmd_c.clone(), CommandMeta::new("Charlie"));

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
        assert_eq!(ui.focus(), Some(overlays.context_menu.root));

        run_frame(&mut ui, &mut host, &mut services);

        ui.dispatch_event(
            &mut host,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::KeyC,
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
        assert_eq!(app_commands, vec![cmd_c]);
        assert_eq!(ui.focus(), Some(trigger));
    }

    #[test]
    fn context_menu_typeahead_cycles_on_repeated_char() {
        let mut host = App::new();
        let mut services = FakeServices::default();

        let window = AppWindowId::default();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(TestContainer);
        ui.set_root(root);

        let mut overlays = WindowOverlays::install(&mut ui);

        let trigger = ui.create_node(Focusable);
        ui.add_child(root, trigger);
        ui.set_focus(Some(trigger));

        let cmd_a1 = CommandId::from("test.menu.alpha");
        let cmd_a2 = CommandId::from("test.menu.alpine");

        host.commands_mut()
            .register(cmd_a1.clone(), CommandMeta::new("Alpha"));
        host.commands_mut()
            .register(cmd_a2.clone(), CommandMeta::new("Alpine"));

        let menu = Menu {
            title: "Menu".into(),
            items: vec![
                MenuItem::Command {
                    command: cmd_a1.clone(),
                    when: None,
                },
                MenuItem::Command {
                    command: cmd_a2.clone(),
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
        assert_eq!(ui.focus(), Some(overlays.context_menu.root));

        run_frame(&mut ui, &mut host, &mut services);

        ui.dispatch_event(
            &mut host,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::KeyA,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        ui.dispatch_event(
            &mut host,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::KeyA,
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
        assert_eq!(app_commands, vec![cmd_a1]);
        assert_eq!(ui.focus(), Some(trigger));
    }

    #[test]
    fn context_menu_page_down_up_moves_selection_by_page() {
        let mut host = App::new();
        let mut services = FakeServices::default();

        let window = AppWindowId::default();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(TestContainer);
        ui.set_root(root);

        let mut overlays = WindowOverlays::install(&mut ui);

        let trigger = ui.create_node(Focusable);
        ui.add_child(root, trigger);
        ui.set_focus(Some(trigger));

        let cmd0 = CommandId::from("test.menu.item.0");
        let cmd1 = CommandId::from("test.menu.item.1");
        let cmd2 = CommandId::from("test.menu.item.2");

        host.commands_mut()
            .register(cmd0.clone(), CommandMeta::new("Item 0"));
        host.commands_mut()
            .register(cmd1.clone(), CommandMeta::new("Item 1"));
        host.commands_mut()
            .register(cmd2.clone(), CommandMeta::new("Item 2"));

        let menu = Menu {
            title: "Menu".into(),
            items: vec![
                MenuItem::Command {
                    command: cmd0.clone(),
                    when: None,
                },
                MenuItem::Command {
                    command: cmd1.clone(),
                    when: None,
                },
                MenuItem::Command {
                    command: cmd2.clone(),
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
        assert_eq!(ui.focus(), Some(overlays.context_menu.root));

        run_frame(&mut ui, &mut host, &mut services);

        ui.dispatch_event(
            &mut host,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::PageDown,
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

        let commands = pump_commands(&mut overlays, &mut host, &mut ui, &mut services, window);
        assert_eq!(commands, vec![cmd2.clone()]);
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
        assert_eq!(ui.focus(), Some(overlays.context_menu.root));

        run_frame(&mut ui, &mut host, &mut services);

        ui.dispatch_event(
            &mut host,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::PageUp,
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

        let commands = pump_commands(&mut overlays, &mut host, &mut ui, &mut services, window);
        assert_eq!(commands, vec![cmd0]);
        assert_eq!(ui.focus(), Some(trigger));
    }

    #[test]
    fn context_menu_submenu_arrow_right_opens_and_arrow_left_closes() {
        let mut host = App::new();
        let mut services = FakeServices::default();

        let window = AppWindowId::default();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(TestContainer);
        ui.set_root(root);

        let mut overlays = WindowOverlays::install(&mut ui);

        let trigger = ui.create_node(Focusable);
        ui.add_child(root, trigger);
        ui.set_focus(Some(trigger));

        let cmd_disabled = CommandId::from("test.menu.sub.disabled");
        let cmd_enabled = CommandId::from("test.menu.sub.enabled");
        host.commands_mut().register(
            cmd_disabled.clone(),
            CommandMeta::new("Alpha").with_when(WhenExpr::parse("false").unwrap()),
        );
        host.commands_mut()
            .register(cmd_enabled.clone(), CommandMeta::new("Beta"));

        let menu = Menu {
            title: "Menu".into(),
            items: vec![MenuItem::Submenu {
                title: "Sub".into(),
                when: None,
                items: vec![
                    MenuItem::Command {
                        command: cmd_disabled,
                        when: None,
                    },
                    MenuItem::Command {
                        command: cmd_enabled.clone(),
                        when: None,
                    },
                ],
            }],
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
        assert_eq!(ui.focus(), Some(overlays.context_menu.root));

        run_frame(&mut ui, &mut host, &mut services);

        ui.dispatch_event(
            &mut host,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::ArrowRight,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        run_frame(&mut ui, &mut host, &mut services);
        ui.dispatch_event(
            &mut host,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::ArrowLeft,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        run_frame(&mut ui, &mut host, &mut services);

        let commands = pump_commands(&mut overlays, &mut host, &mut ui, &mut services, window);
        assert_eq!(commands, Vec::<CommandId>::new());
        assert_eq!(ui.focus(), Some(overlays.context_menu.root));

        ui.dispatch_event(
            &mut host,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::Space,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        run_frame(&mut ui, &mut host, &mut services);

        // First Space re-opens the submenu (no command yet).
        let commands = pump_commands(&mut overlays, &mut host, &mut ui, &mut services, window);
        assert_eq!(commands, Vec::<CommandId>::new());
        assert_eq!(ui.focus(), Some(overlays.context_menu.root));

        ui.dispatch_event(
            &mut host,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::Space,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        // Second Space activates the first enabled item (skipping the disabled one) and closes.
        let commands = pump_commands(&mut overlays, &mut host, &mut ui, &mut services, window);
        assert_eq!(commands, vec![cmd_enabled]);
        assert_eq!(ui.focus(), Some(trigger));
    }

    #[test]
    fn context_menu_submenu_enter_opens_and_space_activates_first_enabled() {
        let mut host = App::new();
        let mut services = FakeServices::default();

        let window = AppWindowId::default();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(TestContainer);
        ui.set_root(root);

        let mut overlays = WindowOverlays::install(&mut ui);

        let trigger = ui.create_node(Focusable);
        ui.add_child(root, trigger);
        ui.set_focus(Some(trigger));

        let cmd_disabled = CommandId::from("test.menu.sub.disabled");
        let cmd_enabled = CommandId::from("test.menu.sub.enabled");
        host.commands_mut().register(
            cmd_disabled.clone(),
            CommandMeta::new("Alpha").with_when(WhenExpr::parse("false").unwrap()),
        );
        host.commands_mut()
            .register(cmd_enabled.clone(), CommandMeta::new("Beta"));

        let menu = Menu {
            title: "Menu".into(),
            items: vec![MenuItem::Submenu {
                title: "Sub".into(),
                when: None,
                items: vec![
                    MenuItem::Command {
                        command: cmd_disabled,
                        when: None,
                    },
                    MenuItem::Command {
                        command: cmd_enabled.clone(),
                        when: None,
                    },
                ],
            }],
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
        assert_eq!(ui.focus(), Some(overlays.context_menu.root));

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
        run_frame(&mut ui, &mut host, &mut services);

        let commands = pump_commands(&mut overlays, &mut host, &mut ui, &mut services, window);
        assert_eq!(commands, Vec::<CommandId>::new());
        assert_eq!(ui.focus(), Some(overlays.context_menu.root));

        ui.dispatch_event(
            &mut host,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::Space,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        let commands = pump_commands(&mut overlays, &mut host, &mut ui, &mut services, window);
        assert_eq!(commands, vec![cmd_enabled]);
        assert_eq!(ui.focus(), Some(trigger));
    }

    #[test]
    fn select_keyboard_opens_popover_and_selects_enabled_item() {
        let mut host = App::new();
        let mut services = FakeServices::default();

        let window = AppWindowId::default();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(TestContainer);
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
        assert_eq!(ui.focus(), Some(overlays.popover.root));

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
    fn select_typeahead_jumps_to_matching_option() {
        let mut host = App::new();
        let mut services = FakeServices::default();

        let window = AppWindowId::default();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(TestContainer);
        ui.set_root(root);

        let mut overlays = WindowOverlays::install(&mut ui);

        let model: Model<usize> = host.models_mut().insert(0usize);
        let select = ui.create_node(Select::new(
            model,
            vec![
                SelectOption::new("Apple").disabled(),
                SelectOption::new("Banana"),
                SelectOption::new("Cherry"),
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
        assert_eq!(ui.focus(), Some(overlays.popover.root));

        run_frame(&mut ui, &mut host, &mut services);

        ui.dispatch_event(
            &mut host,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::KeyC,
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
        assert_eq!(host.models().get(model).copied(), Some(2));
    }

    #[test]
    fn popover_surface_open_focuses_first_focusable_descendant_and_close_restores_focus() {
        let mut host = App::new();
        let mut services = FakeServices::default();

        let window = AppWindowId::default();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(TestContainer);
        ui.set_root(root);

        let mut overlays = WindowOverlays::install(&mut ui);

        let prev_focus = ui.create_node(Focusable);
        ui.add_child(root, prev_focus);
        ui.set_focus(Some(prev_focus));

        let surface_root = overlays.popover_surface_node();
        let content_root = ui.create_node(TestContainer);
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
