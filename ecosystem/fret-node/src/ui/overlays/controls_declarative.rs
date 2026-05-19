use std::sync::Arc;

use fret_core::{Corners, Edges, Px, SemanticsRole, Size, TextOverflow, TextWrap};
use fret_ui::element::{
    AnyElement, ColumnProps, ContainerProps, CrossAlign, Length, PressableProps, SpacingEdges,
    SpacingLength, TextProps,
};
use fret_ui::{ElementContext, UiHost};

use crate::interaction::NodeGraphConnectionMode;
use crate::ui::NodeGraphStyle;

use super::controls_layout::controls_panel_size;
use super::controls_policy::{
    ControlsButton, NodeGraphControlsBindings, controls_button_a11y_label, controls_button_label,
    controls_buttons, resolve_controls_command_id,
};

#[derive(Debug, Clone)]
pub(super) struct NodeGraphControlsOverlayElementProps {
    pub(super) style: NodeGraphStyle,
    pub(super) bindings: NodeGraphControlsBindings,
    pub(super) connection_mode: NodeGraphConnectionMode,
}

pub(super) fn node_graph_controls_overlay_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    props: NodeGraphControlsOverlayElementProps,
) -> AnyElement {
    let panel = controls_panel_size(&props.style);
    let style = props.style;
    let bindings = props.bindings;
    let connection_mode = props.connection_mode;

    cx.container(container_for_size(panel, &style), move |cx| {
        vec![cx.column(column_for_style(&style), move |cx| {
            controls_buttons()
                .iter()
                .copied()
                .map(|button| {
                    cx.keyed(controls_button_slot(button), |cx| {
                        control_button(cx, &style, &bindings, connection_mode, button)
                    })
                })
                .collect::<Vec<_>>()
        })]
    })
}

fn container_for_size(size: Size, style: &NodeGraphStyle) -> ContainerProps {
    let mut props = ContainerProps::default();
    props.layout.size.width = Length::Px(size.width);
    props.layout.size.height = Length::Px(size.height);
    props.padding = SpacingEdges::all(SpacingLength::Px(Px(style.paint.controls_padding.max(0.0))));
    props.background = Some(style.paint.context_menu_background);
    props.border = Edges::all(Px(1.0));
    props.border_color = Some(style.paint.context_menu_border);
    props.corner_radii = Corners::all(Px(style.paint.context_menu_corner_radius.max(0.0)));
    props.snap_to_device_pixels = true;
    props
}

fn column_for_style(style: &NodeGraphStyle) -> ColumnProps {
    let mut props = ColumnProps::default();
    props.layout.size.width = Length::Fill;
    props.layout.size.height = Length::Fill;
    props.gap = SpacingLength::Px(Px(style.paint.controls_gap.max(0.0)));
    props.align = CrossAlign::Center;
    props
}

fn control_button<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    style: &NodeGraphStyle,
    bindings: &NodeGraphControlsBindings,
    connection_mode: NodeGraphConnectionMode,
    button: ControlsButton,
) -> AnyElement {
    let command_id = resolve_controls_command_id(bindings, button);
    let mut props = PressableProps::default();
    props.enabled = command_id.is_some();
    props.layout.size.width = Length::Px(Px(style.paint.controls_button_size.max(10.0)));
    props.layout.size.height = Length::Px(Px(style.paint.controls_button_size.max(10.0)));
    props.a11y.role = Some(SemanticsRole::Button);
    props.a11y.label = Some(Arc::from(controls_button_a11y_label(button)));
    props.a11y.test_id = Some(Arc::from(controls_button_test_id(button)));

    cx.pressable(props, move |cx, _state| {
        if let Some(command_id) = command_id.clone() {
            cx.pressable_on_activate(Arc::new(move |host, action_cx, reason| {
                host.record_pending_command_dispatch_source(action_cx, &command_id, reason);
                host.dispatch_command(Some(action_cx.window), command_id.clone());
            }));
        }

        let mut label = TextProps::new(controls_button_label(button, connection_mode));
        label.style = Some(style.paint.controls_text_style.clone());
        label.color = Some(style.paint.controls_text);
        label.wrap = TextWrap::None;
        label.overflow = TextOverflow::Clip;
        vec![cx.text_props(label)]
    })
}

fn controls_button_slot(button: ControlsButton) -> &'static str {
    match button {
        ControlsButton::ToggleConnectionMode => "controls.toggle_connection_mode",
        ControlsButton::ZoomIn => "controls.zoom_in",
        ControlsButton::ZoomOut => "controls.zoom_out",
        ControlsButton::FrameAll => "controls.frame_all",
        ControlsButton::FrameSelection => "controls.frame_selection",
        ControlsButton::ResetView => "controls.reset_view",
    }
}

fn controls_button_test_id(button: ControlsButton) -> &'static str {
    match button {
        ControlsButton::ToggleConnectionMode => "node_graph.controls.toggle_connection_mode",
        ControlsButton::ZoomIn => "node_graph.controls.zoom_in",
        ControlsButton::ZoomOut => "node_graph.controls.zoom_out",
        ControlsButton::FrameAll => "node_graph.controls.frame_all",
        ControlsButton::FrameSelection => "node_graph.controls.frame_selection",
        ControlsButton::ResetView => "node_graph.controls.reset_view",
    }
}

#[cfg(test)]
mod tests {
    use std::any::{Any, TypeId};
    use std::collections::HashMap;

    use fret_core::{
        AppWindowId, MaterialDescriptor, MaterialId, MaterialRegistrationError, Modifiers,
        MouseButton, PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle,
        Point, PointerId, PointerType, Px, Rect, SemanticsRole, SvgId, SvgService, TextConstraints,
        TextMetrics, TextService,
    };
    use fret_runtime::{
        ClipboardToken, CommandId, CommandRegistry, CommandsHost, DragHost, DragKindId,
        DragSession, Effect, EffectSink, FrameId, GlobalsHost, ImageUploadToken, ModelHost,
        ModelId, ModelStore, ModelsHost, ShareSheetToken, TickId, TimeHost, TimerToken,
    };
    use fret_ui::element::{ElementKind, Length};

    use crate::interaction::NodeGraphConnectionMode;
    use crate::ui::commands::CMD_NODE_GRAPH_ZOOM_IN;
    use crate::ui::overlays::controls_declarative::{
        NodeGraphControlsOverlayElementProps, node_graph_controls_overlay_element,
    };
    use crate::ui::overlays::controls_layout::controls_panel_size;
    use crate::ui::overlays::controls_policy::{
        NodeGraphControlsBindings, NodeGraphControlsCommandBinding,
    };
    use crate::ui::style::NodeGraphStyle;

    #[derive(Default)]
    struct TestUiHost {
        globals: HashMap<TypeId, Box<dyn Any>>,
        models: ModelStore,
        commands: CommandRegistry,
        effects: Vec<Effect>,
        tick_id: TickId,
        frame_id: FrameId,
        next_timer_token: u64,
        next_clipboard_token: u64,
        next_share_sheet_token: u64,
        next_image_upload_token: u64,
    }

    impl GlobalsHost for TestUiHost {
        fn set_global<T: Any>(&mut self, value: T) {
            self.globals.insert(TypeId::of::<T>(), Box::new(value));
        }

        fn global<T: Any>(&self) -> Option<&T> {
            self.globals
                .get(&TypeId::of::<T>())
                .and_then(|v| v.downcast_ref::<T>())
        }

        fn with_global_mut<T: Any, R>(
            &mut self,
            init: impl FnOnce() -> T,
            f: impl FnOnce(&mut T, &mut Self) -> R,
        ) -> R {
            let type_id = TypeId::of::<T>();
            let existing = self.globals.remove(&type_id);
            let mut value = existing
                .and_then(|v| v.downcast::<T>().ok().map(|v| *v))
                .unwrap_or_else(init);
            let out = f(&mut value, self);
            self.globals.insert(type_id, Box::new(value));
            out
        }
    }

    impl ModelHost for TestUiHost {
        fn models(&self) -> &ModelStore {
            &self.models
        }

        fn models_mut(&mut self) -> &mut ModelStore {
            &mut self.models
        }
    }

    impl ModelsHost for TestUiHost {
        fn take_changed_models(&mut self) -> Vec<ModelId> {
            Vec::new()
        }
    }

    impl CommandsHost for TestUiHost {
        fn commands(&self) -> &CommandRegistry {
            &self.commands
        }
    }

    impl EffectSink for TestUiHost {
        fn request_redraw(&mut self, _window: AppWindowId) {}

        fn push_effect(&mut self, effect: Effect) {
            self.effects.push(effect);
        }
    }

    impl TimeHost for TestUiHost {
        fn tick_id(&self) -> TickId {
            self.tick_id
        }

        fn frame_id(&self) -> FrameId {
            self.frame_id
        }

        fn next_timer_token(&mut self) -> TimerToken {
            let out = TimerToken(self.next_timer_token);
            self.next_timer_token = self.next_timer_token.saturating_add(1);
            out
        }

        fn next_clipboard_token(&mut self) -> ClipboardToken {
            let out = ClipboardToken(self.next_clipboard_token);
            self.next_clipboard_token = self.next_clipboard_token.saturating_add(1);
            out
        }

        fn next_share_sheet_token(&mut self) -> ShareSheetToken {
            let out = ShareSheetToken(self.next_share_sheet_token);
            self.next_share_sheet_token = self.next_share_sheet_token.saturating_add(1);
            out
        }

        fn next_image_upload_token(&mut self) -> ImageUploadToken {
            let out = ImageUploadToken(self.next_image_upload_token);
            self.next_image_upload_token = self.next_image_upload_token.saturating_add(1);
            out
        }
    }

    #[derive(Default)]
    struct FakeUiServices;

    impl TextService for FakeUiServices {
        fn prepare(
            &mut self,
            _input: &fret_core::TextInput,
            _constraints: TextConstraints,
        ) -> (fret_core::TextBlobId, TextMetrics) {
            (
                fret_core::TextBlobId::default(),
                TextMetrics {
                    size: fret_core::Size::new(Px(10.0), Px(10.0)),
                    baseline: Px(8.0),
                },
            )
        }

        fn release(&mut self, _blob: fret_core::TextBlobId) {}
    }

    impl PathService for FakeUiServices {
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

    impl SvgService for FakeUiServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
            SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: SvgId) -> bool {
            true
        }
    }

    impl fret_core::MaterialService for FakeUiServices {
        fn register_material(
            &mut self,
            _desc: MaterialDescriptor,
        ) -> Result<MaterialId, MaterialRegistrationError> {
            Err(MaterialRegistrationError::Unsupported)
        }

        fn unregister_material(&mut self, _id: MaterialId) -> bool {
            false
        }
    }

    impl DragHost for TestUiHost {
        fn drag(&self, _pointer_id: PointerId) -> Option<&DragSession> {
            None
        }

        fn drag_mut(&mut self, _pointer_id: PointerId) -> Option<&mut DragSession> {
            None
        }

        fn cancel_drag(&mut self, _pointer_id: PointerId) {}

        fn any_drag_session(&self, _predicate: impl FnMut(&DragSession) -> bool) -> bool {
            false
        }

        fn find_drag_pointer_id(
            &self,
            _predicate: impl FnMut(&DragSession) -> bool,
        ) -> Option<PointerId> {
            None
        }

        fn cancel_drag_sessions(
            &mut self,
            _predicate: impl FnMut(&DragSession) -> bool,
        ) -> Vec<PointerId> {
            Vec::new()
        }

        fn begin_drag_with_kind<T: Any>(
            &mut self,
            _pointer_id: PointerId,
            _kind: DragKindId,
            _source_window: AppWindowId,
            _start: Point,
            _payload: T,
        ) {
        }

        fn begin_cross_window_drag_with_kind<T: Any>(
            &mut self,
            _pointer_id: PointerId,
            _kind: DragKindId,
            _source_window: AppWindowId,
            _start: Point,
            _payload: T,
        ) {
        }
    }

    fn render_controls(
        style: NodeGraphStyle,
        bindings: NodeGraphControlsBindings,
        mode: NodeGraphConnectionMode,
    ) -> fret_ui::element::AnyElement {
        let mut host = TestUiHost::default();
        let mut runtime = fret_ui::ElementRuntime::new();
        let window = AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(300.0)),
        );
        let mut cx = fret_ui::ElementContext::new_for_root_name(
            &mut host,
            &mut runtime,
            window,
            bounds,
            "root",
        );
        node_graph_controls_overlay_element(
            &mut cx,
            NodeGraphControlsOverlayElementProps {
                style,
                bindings,
                connection_mode: mode,
            },
        )
    }

    #[test]
    fn controls_declarative_composition_builds_panel_and_button_tree_without_retained_widget() {
        let mut style = NodeGraphStyle::default();
        style.paint.controls_button_size = 20.0;
        style.paint.controls_padding = 4.0;
        style.paint.controls_gap = 2.0;
        let expected = controls_panel_size(&style);

        let root = render_controls(
            style,
            NodeGraphControlsBindings::default(),
            NodeGraphConnectionMode::Strict,
        );

        let ElementKind::Container(panel) = &root.kind else {
            panic!("controls root should be a declarative container");
        };
        assert_eq!(panel.layout.size.width, Length::Px(expected.width));
        assert_eq!(panel.layout.size.height, Length::Px(expected.height));
        assert_eq!(root.children.len(), 1);

        let ElementKind::Column(_) = &root.children[0].kind else {
            panic!("controls root should contain a declarative column");
        };
        assert_eq!(root.children[0].children.len(), 6);

        let first = &root.children[0].children[0];
        let ElementKind::Pressable(props) = &first.kind else {
            panic!("control entries should be declarative pressables");
        };
        assert_eq!(props.a11y.role, Some(SemanticsRole::Button));
        assert_eq!(props.a11y.label.as_deref(), Some("Toggle connection mode"));
        assert_eq!(
            props.a11y.test_id.as_deref(),
            Some("node_graph.controls.toggle_connection_mode")
        );
        let ElementKind::Text(label) = &first.children[0].kind else {
            panic!("control button should contain a text label");
        };
        assert_eq!(label.text.as_ref(), "S");
    }

    #[test]
    fn controls_declarative_composition_honors_mode_and_disabled_bindings() {
        let mut bindings = NodeGraphControlsBindings::default();
        bindings.zoom_in = NodeGraphControlsCommandBinding::Disabled;

        let root = render_controls(
            NodeGraphStyle::default(),
            bindings,
            NodeGraphConnectionMode::Loose,
        );
        let column = &root.children[0];

        let toggle = &column.children[0];
        let ElementKind::Text(toggle_label) = &toggle.children[0].kind else {
            panic!("toggle button should contain a text label");
        };
        assert_eq!(toggle_label.text.as_ref(), "L");

        let zoom_in = &column.children[1];
        let ElementKind::Pressable(zoom_in_props) = &zoom_in.kind else {
            panic!("zoom in should be a pressable");
        };
        assert!(!zoom_in_props.enabled);
        assert_eq!(
            zoom_in_props.a11y.test_id.as_deref(),
            Some("node_graph.controls.zoom_in")
        );

        let zoom_in_text = &zoom_in.children[0];
        let ElementKind::Text(label) = &zoom_in_text.kind else {
            panic!("zoom in button should contain a text label");
        };
        assert_eq!(label.text.as_ref(), "+");

        let zoom_out = &column.children[2];
        let ElementKind::Pressable(zoom_out_props) = &zoom_out.kind else {
            panic!("zoom out should be a pressable");
        };
        assert!(zoom_out_props.enabled);
        assert_ne!(
            zoom_out_props.a11y.test_id.as_deref(),
            Some(CMD_NODE_GRAPH_ZOOM_IN)
        );
    }

    #[test]
    fn controls_declarative_activation_dispatches_commands_and_honors_disabled_bindings() {
        let mut host = TestUiHost::default();
        let mut ui = fret_ui::UiTree::<TestUiHost>::new();
        let mut services = FakeUiServices;
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(120.0), Px(220.0)),
        );
        let custom_zoom = CommandId::from("node_graph.custom.zoom_in");
        let mut bindings = NodeGraphControlsBindings::default();
        bindings.zoom_in = NodeGraphControlsCommandBinding::Command(custom_zoom.clone());
        bindings.zoom_out = NodeGraphControlsCommandBinding::Disabled;

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut host,
            &mut services,
            window,
            bounds,
            "controls-activation",
            |cx| {
                vec![node_graph_controls_overlay_element(
                    cx,
                    NodeGraphControlsOverlayElementProps {
                        style: NodeGraphStyle::default(),
                        bindings,
                        connection_mode: NodeGraphConnectionMode::Strict,
                    },
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut host, &mut services, bounds, 1.0);

        let panel = ui.children(root)[0];
        let column = ui.children(panel)[0];
        let zoom_in = ui.children(column)[1];
        click_node(&mut ui, &mut host, &mut services, zoom_in);
        assert!(
            host.effects.iter().any(|effect| {
                matches!(
                    effect,
                    Effect::Command { window: effect_window, command }
                        if *effect_window == Some(window) && command == &custom_zoom
                )
            }),
            "custom zoom-in command should dispatch from the declarative controls pressable"
        );

        host.effects.clear();
        let zoom_out = ui.children(column)[2];
        click_node(&mut ui, &mut host, &mut services, zoom_out);
        assert!(
            !host
                .effects
                .iter()
                .any(|effect| matches!(effect, Effect::Command { .. })),
            "disabled controls binding should suppress declarative command activation: {:?}",
            host.effects
        );
    }

    fn click_node(
        ui: &mut fret_ui::UiTree<TestUiHost>,
        host: &mut TestUiHost,
        services: &mut FakeUiServices,
        node: fret_core::NodeId,
    ) {
        let rect = ui.debug_node_bounds(node).expect("node should be laid out");
        let position = Point::new(Px(rect.origin.x.0 + 1.0), Px(rect.origin.y.0 + 1.0));
        ui.dispatch_event(
            host,
            services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );
        ui.dispatch_event(
            host,
            services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                position,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                click_count: 1,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );
    }
}
