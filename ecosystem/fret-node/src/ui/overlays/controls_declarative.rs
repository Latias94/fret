use std::sync::Arc;

use fret_core::{Corners, Edges, Px, SemanticsRole, Size, TextOverflow, TextWrap};
use fret_ui::action::PressablePointerDownResult;
use fret_ui::element::{
    AnyElement, ColumnProps, ContainerProps, CrossAlign, Length, PointerRegionProps,
    PressableProps, SemanticsProps, SpacingEdges, SpacingLength, TextProps,
};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::interaction::NodeGraphConnectionMode;
use crate::ui::NodeGraphStyle;

use super::controls_host_policy::plan_controls_declarative_panel_pointer_down;
use super::controls_interaction_policy::{
    ControlsInteractionState, ControlsKeyboardInteractionPlan, plan_controls_keyboard_interaction,
    plan_controls_pointer_down_interaction,
};
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
    pub(super) focus_target: Option<GlobalElementId>,
}

pub(super) fn node_graph_controls_overlay_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    props: NodeGraphControlsOverlayElementProps,
) -> AnyElement {
    let panel = controls_panel_size(&props.style);
    let style = props.style;
    let bindings = props.bindings;
    let connection_mode = props.connection_mode;
    let focus_target = props.focus_target;
    let interaction = cx.local_model(|| ControlsInteractionState::new(None, None, None));
    let active = interaction
        .read_ref(cx.app, |state| state.keyboard_active)
        .ok()
        .flatten()
        .or_else(|| controls_buttons().first().copied())
        .expect("controls buttons");

    cx.semantics_with_id(
        controls_semantics_props(active),
        move |cx, controls_root| {
            let key_bindings = bindings.clone();
            let key_interaction = interaction.clone();
            cx.key_on_key_down_for(
                controls_root,
                Arc::new(move |host, action_cx, down| {
                    let mut plan = ControlsKeyboardInteractionPlan::Ignore;
                    let updated = host
                        .models_mut()
                        .update(&key_interaction, |state| {
                            plan = plan_controls_keyboard_interaction(state, down.key);
                        })
                        .is_ok();
                    if !updated {
                        return false;
                    }

                    match plan {
                        ControlsKeyboardInteractionPlan::Ignore => false,
                        ControlsKeyboardInteractionPlan::Select { finish_event, .. } => {
                            host.notify(action_cx);
                            finish_event
                        }
                        ControlsKeyboardInteractionPlan::Activate {
                            button,
                            finish_event,
                        } => {
                            dispatch_controls_button(
                                host,
                                action_cx,
                                &key_bindings,
                                button,
                                focus_target,
                                fret_ui::action::ActivateReason::Keyboard,
                            );
                            host.notify(action_cx);
                            finish_event
                        }
                        ControlsKeyboardInteractionPlan::FocusCanvas { finish_event } => {
                            if let Some(focus_target) = focus_target {
                                host.request_focus(focus_target);
                            }
                            host.notify(action_cx);
                            finish_event
                        }
                    }
                }),
            );

            vec![cx.pointer_region(PointerRegionProps::default(), move |cx| {
                cx.pointer_region_on_pointer_down(Arc::new(move |host, action_cx, down| {
                    let Some(plan) = plan_controls_declarative_panel_pointer_down(
                        down.button,
                        down.hit_is_pressable,
                    ) else {
                        return false;
                    };

                    if plan.request_focus {
                        host.request_focus(controls_root);
                    }
                    if plan.capture_pointer {
                        host.capture_pointer();
                    }
                    if plan.repaint {
                        host.request_redraw(action_cx.window);
                    }
                    plan.stop_propagation
                }));

                vec![cx.container(container_for_size(panel, &style), move |cx| {
                    vec![cx.column(column_for_style(&style), move |cx| {
                        controls_buttons()
                            .iter()
                            .copied()
                            .map(|button| {
                                cx.keyed(controls_button_slot(button), |cx| {
                                    control_button(
                                        cx,
                                        &style,
                                        &bindings,
                                        connection_mode,
                                        button,
                                        focus_target,
                                        controls_root,
                                        interaction.clone(),
                                    )
                                })
                            })
                            .collect::<Vec<_>>()
                    })]
                })]
            })]
        },
    )
}

fn controls_semantics_props(active: ControlsButton) -> SemanticsProps {
    SemanticsProps {
        role: SemanticsRole::Panel,
        label: Some(Arc::from("Controls")),
        test_id: Some(Arc::from("node_graph.controls")),
        value: Some(Arc::from(controls_button_a11y_label(active))),
        focusable: true,
        ..Default::default()
    }
}

fn dispatch_controls_button(
    host: &mut dyn fret_ui::action::UiFocusActionHost,
    action_cx: fret_ui::action::ActionCx,
    bindings: &NodeGraphControlsBindings,
    button: ControlsButton,
    focus_target: Option<GlobalElementId>,
    reason: fret_ui::action::ActivateReason,
) {
    if let Some(focus_target) = focus_target {
        host.request_focus(focus_target);
    }

    if let Some(command_id) = resolve_controls_command_id(bindings, button) {
        host.record_pending_command_dispatch_source(action_cx, &command_id, reason);
        host.dispatch_command(Some(action_cx.window), command_id);
    }
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
    focus_target: Option<GlobalElementId>,
    controls_root: GlobalElementId,
    interaction: fret_runtime::Model<ControlsInteractionState>,
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
        let pointer_interaction = interaction.clone();
        cx.pressable_on_pointer_down(Arc::new(move |host, action_cx, down| {
            let _ = host.models_mut().update(&pointer_interaction, |state| {
                let _ = plan_controls_pointer_down_interaction(state, down.button, Some(button));
            });
            host.request_focus(controls_root);
            host.notify(action_cx);
            PressablePointerDownResult::Continue
        }));

        if let Some(command_id) = command_id.clone() {
            let activate_interaction = interaction.clone();
            cx.pressable_on_activate(Arc::new(move |host, action_cx, reason| {
                let _ = host
                    .models_mut()
                    .update(&activate_interaction, ControlsInteractionState::clear);
                host.record_pending_command_dispatch_source(action_cx, &command_id, reason);
                host.dispatch_command(Some(action_cx.window), command_id.clone());
                host.notify(action_cx);
            }));
        }
        if let Some(focus_target) = focus_target {
            cx.pressable_on_activate_focus(Arc::new(move |host, _action_cx, _reason| {
                host.request_focus(focus_target);
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
    use std::sync::Arc;

    use fret_core::{
        AppWindowId, MaterialDescriptor, MaterialId, MaterialRegistrationError, Modifiers,
        MouseButton, PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle,
        Point, PointerId, PointerType, Px, Rect, SemanticsRole, SvgId, SvgService, TextConstraints,
        TextMetrics, TextService,
    };
    use fret_runtime::{
        ClipboardToken, CommandId, CommandMeta, CommandRegistry, CommandScope, CommandsHost,
        DragHost, DragKindId, DragSession, Effect, EffectSink, FrameId, GlobalsHost,
        ImageUploadToken, Model, ModelHost, ModelId, ModelStore, ModelsHost, ShareSheetToken,
        TickId, TimeHost, TimerToken,
    };
    use fret_ui::GlobalElementId;
    use fret_ui::element::{ContainerProps, ElementKind, Length, StackProps};

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
                focus_target: None,
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

        let ElementKind::Semantics(root_props) = &root.kind else {
            panic!("controls root should be a declarative semantics wrapper");
        };
        assert_eq!(root_props.role, SemanticsRole::Panel);
        assert_eq!(root_props.label.as_deref(), Some("Controls"));
        assert_eq!(root_props.test_id.as_deref(), Some("node_graph.controls"));
        assert_eq!(root_props.value.as_deref(), Some("Toggle connection mode"));
        assert!(root_props.focusable);
        assert_eq!(root.children.len(), 1);

        let pointer_region = &root.children[0];
        let ElementKind::PointerRegion(_) = &pointer_region.kind else {
            panic!("controls semantics root should contain a pointer region");
        };
        assert_eq!(pointer_region.children.len(), 1);

        let panel = &pointer_region.children[0];
        let ElementKind::Container(panel_props) = &panel.kind else {
            panic!("controls pointer region should contain a declarative container");
        };
        assert_eq!(panel_props.layout.size.width, Length::Px(expected.width));
        assert_eq!(panel_props.layout.size.height, Length::Px(expected.height));
        assert_eq!(panel.children.len(), 1);

        let ElementKind::Column(_) = &panel.children[0].kind else {
            panic!("controls root should contain a declarative column");
        };
        assert_eq!(panel.children[0].children.len(), 6);

        let first = &panel.children[0].children[0];
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
        let pointer_region = &root.children[0];
        let panel = &pointer_region.children[0];
        let column = &panel.children[0];

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
                        focus_target: None,
                    },
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut host, &mut services, bounds, 1.0);

        let controls = ui.children(root)[0];
        let pointer_region = ui.children(controls)[0];
        let panel = ui.children(pointer_region)[0];
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

    #[test]
    fn controls_declarative_button_pointer_up_completes_capture_focus_and_command_dispatch() {
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
        bindings.zoom_out =
            NodeGraphControlsCommandBinding::Command(CommandId::from("node_graph.custom.zoom_out"));

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut host,
            &mut services,
            window,
            bounds,
            "controls-button-pointer-up-completion",
            |cx| {
                vec![node_graph_controls_overlay_element(
                    cx,
                    NodeGraphControlsOverlayElementProps {
                        style: NodeGraphStyle::default(),
                        bindings,
                        connection_mode: NodeGraphConnectionMode::Strict,
                        focus_target: None,
                    },
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut host, &mut services, bounds, 1.0);

        let controls = ui.children(root)[0];
        let pointer_region = ui.children(controls)[0];
        let panel = ui.children(pointer_region)[0];
        let column = ui.children(panel)[0];
        let zoom_in = ui.children(column)[1];

        let position = pointer_position_inside_node(&ui, zoom_in);
        dispatch_pointer_down_at(&mut ui, &mut host, &mut services, position);
        assert_eq!(
            ui.captured(),
            Some(zoom_in),
            "controls pressable should capture pointer on button down"
        );
        assert!(
            !host
                .effects
                .iter()
                .any(|effect| matches!(effect, Effect::Command { .. })),
            "controls command should wait for pointer-up completion: {:?}",
            host.effects
        );

        dispatch_pointer_up_at(&mut ui, &mut host, &mut services, position);
        assert_eq!(
            ui.captured(),
            None,
            "controls pressable should release pointer capture on button up"
        );
        assert_eq!(
            ui.focus(),
            Some(zoom_in),
            "controls button pointer-up activation should focus the pressable button"
        );
        assert!(
            host.effects.iter().any(|effect| {
                matches!(
                    effect,
                    Effect::Command { window: effect_window, command }
                        if *effect_window == Some(window) && command == &custom_zoom
                )
            }),
            "controls command should dispatch after pointer-up activation: {:?}",
            host.effects
        );

        host.effects.clear();
        let zoom_out = ui.children(column)[2];
        let down = pointer_position_inside_node(&ui, zoom_out);
        let outside = Point::new(Px(bounds.origin.x.0 + bounds.size.width.0 + 10.0), down.y);
        dispatch_pointer_down_at(&mut ui, &mut host, &mut services, down);
        assert_eq!(ui.captured(), Some(zoom_out));
        dispatch_pointer_up_at(&mut ui, &mut host, &mut services, outside);
        assert_eq!(ui.captured(), None);
        assert!(
            !host
                .effects
                .iter()
                .any(|effect| matches!(effect, Effect::Command { .. })),
            "button release outside should complete capture without command dispatch: {:?}",
            host.effects
        );
    }

    #[test]
    fn controls_declarative_pointer_down_promotes_keyboard_active_semantics_value() {
        let mut host = TestUiHost::default();
        let mut ui = fret_ui::UiTree::<TestUiHost>::new();
        let mut services = FakeUiServices;
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(120.0), Px(220.0)),
        );
        let bindings = NodeGraphControlsBindings::default();
        let root_name = "controls-pointer-down-active-semantics";
        let root = render_controls_root(
            &mut ui,
            &mut host,
            &mut services,
            window,
            bounds,
            root_name,
            NodeGraphStyle::default(),
            bindings.clone(),
            None,
        );

        assert_eq!(
            controls_semantics_value(&mut ui, &mut host, &mut services, bounds),
            Some("Toggle connection mode".to_string())
        );

        let controls = ui.children(root)[0];
        let pointer_region = ui.children(controls)[0];
        let panel = ui.children(pointer_region)[0];
        let column = ui.children(panel)[0];
        let zoom_in = ui.children(column)[1];
        let zoom_in_position = pointer_position_inside_node(&ui, zoom_in);
        dispatch_pointer_down_at(&mut ui, &mut host, &mut services, zoom_in_position);
        assert_eq!(
            ui.focus(),
            Some(controls),
            "controls button pointer-down should acquire root controls focus for keyboard follow-up"
        );

        render_controls_root(
            &mut ui,
            &mut host,
            &mut services,
            window,
            bounds,
            root_name,
            NodeGraphStyle::default(),
            bindings,
            None,
        );
        assert_eq!(
            controls_semantics_value(&mut ui, &mut host, &mut services, bounds),
            Some("Zoom in".to_string())
        );
    }

    #[test]
    fn controls_declarative_root_keyboard_navigation_activation_dispatches_and_restores_focus() {
        let mut host = TestUiHost::default();
        let mut ui = fret_ui::UiTree::<TestUiHost>::new();
        let mut services = FakeUiServices;
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(160.0), Px(240.0)),
        );
        let custom_zoom = CommandId::from("node_graph.custom.zoom_in");
        let mut bindings = NodeGraphControlsBindings::default();
        bindings.zoom_in = NodeGraphControlsCommandBinding::Command(custom_zoom.clone());
        let root_name = "controls-root-keyboard-activation";

        let root = rerender_controls_with_surface(
            &mut ui,
            &mut host,
            &mut services,
            window,
            bounds,
            root_name,
            bindings.clone(),
        );

        let stack = ui.children(root)[0];
        let surface = ui.children(stack)[0];
        let controls = ui.children(stack)[1];
        ui.set_focus(Some(controls));

        dispatch_key_down(
            &mut ui,
            &mut host,
            &mut services,
            fret_core::KeyCode::ArrowDown,
        );
        rerender_controls_with_surface(
            &mut ui,
            &mut host,
            &mut services,
            window,
            bounds,
            root_name,
            bindings.clone(),
        );
        assert_eq!(
            controls_semantics_value(&mut ui, &mut host, &mut services, bounds),
            Some("Zoom in".to_string())
        );

        dispatch_key_down(&mut ui, &mut host, &mut services, fret_core::KeyCode::Enter);
        assert!(
            host.effects.iter().any(|effect| {
                matches!(
                    effect,
                    Effect::Command { window: effect_window, command }
                        if *effect_window == Some(window) && command == &custom_zoom
                )
            }),
            "root keyboard activation should dispatch the selected controls command: {:?}",
            host.effects
        );
        assert_eq!(
            ui.focus(),
            Some(surface),
            "root keyboard activation should restore focus to the node graph surface target"
        );

        rerender_controls_with_surface(
            &mut ui,
            &mut host,
            &mut services,
            window,
            bounds,
            root_name,
            bindings,
        );
        assert_eq!(
            controls_semantics_value(&mut ui, &mut host, &mut services, bounds),
            Some("Toggle connection mode".to_string())
        );
    }

    #[test]
    fn controls_declarative_escape_restores_focus_without_dispatch_and_clears_active_semantics() {
        let mut host = TestUiHost::default();
        let mut ui = fret_ui::UiTree::<TestUiHost>::new();
        let mut services = FakeUiServices;
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(160.0), Px(240.0)),
        );
        let bindings = NodeGraphControlsBindings::default();
        let root_name = "controls-root-keyboard-escape";

        let root = rerender_controls_with_surface(
            &mut ui,
            &mut host,
            &mut services,
            window,
            bounds,
            root_name,
            bindings.clone(),
        );

        let stack = ui.children(root)[0];
        let surface = ui.children(stack)[0];
        let controls = ui.children(stack)[1];
        ui.set_focus(Some(controls));

        dispatch_key_down(
            &mut ui,
            &mut host,
            &mut services,
            fret_core::KeyCode::ArrowDown,
        );
        rerender_controls_with_surface(
            &mut ui,
            &mut host,
            &mut services,
            window,
            bounds,
            root_name,
            bindings.clone(),
        );
        assert_eq!(
            controls_semantics_value(&mut ui, &mut host, &mut services, bounds),
            Some("Zoom in".to_string())
        );

        host.effects.clear();
        dispatch_key_down(
            &mut ui,
            &mut host,
            &mut services,
            fret_core::KeyCode::Escape,
        );
        assert!(
            !host
                .effects
                .iter()
                .any(|effect| matches!(effect, Effect::Command { .. })),
            "Escape should restore focus without dispatching controls commands: {:?}",
            host.effects
        );
        assert_eq!(ui.focus(), Some(surface));

        rerender_controls_with_surface(
            &mut ui,
            &mut host,
            &mut services,
            window,
            bounds,
            root_name,
            bindings,
        );
        assert_eq!(
            controls_semantics_value(&mut ui, &mut host, &mut services, bounds),
            Some("Toggle connection mode".to_string())
        );
    }

    #[test]
    fn controls_declarative_button_activation_restores_focus_to_surface_target() {
        let mut host = TestUiHost::default();
        let mut ui = fret_ui::UiTree::<TestUiHost>::new();
        let mut services = FakeUiServices;
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(160.0), Px(240.0)),
        );
        let custom_zoom = CommandId::from("node_graph.custom.zoom_in");
        let mut bindings = NodeGraphControlsBindings::default();
        bindings.zoom_in = NodeGraphControlsCommandBinding::Command(custom_zoom.clone());

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut host,
            &mut services,
            window,
            bounds,
            "controls-button-activation-restores-surface-focus",
            |cx| {
                let mut stack = StackProps::default();
                stack.layout.size.width = Length::Fill;
                stack.layout.size.height = Length::Fill;
                vec![cx.stack_props(stack, move |cx| {
                    let mut surface_props = ContainerProps::default();
                    surface_props.layout.size.width = Length::Fill;
                    surface_props.layout.size.height = Length::Fill;
                    let surface = cx.container(surface_props, |_cx| Vec::new());
                    let surface_target = surface.id;

                    let controls = node_graph_controls_overlay_element(
                        cx,
                        NodeGraphControlsOverlayElementProps {
                            style: NodeGraphStyle::default(),
                            bindings,
                            connection_mode: NodeGraphConnectionMode::Strict,
                            focus_target: Some(surface_target),
                        },
                    );
                    vec![surface, controls]
                })]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut host, &mut services, bounds, 1.0);

        let stack = ui.children(root)[0];
        let surface = ui.children(stack)[0];
        let controls = ui.children(stack)[1];
        let pointer_region = ui.children(controls)[0];
        let panel = ui.children(pointer_region)[0];
        let column = ui.children(panel)[0];
        let zoom_in = ui.children(column)[1];

        let position = pointer_position_inside_node(&ui, zoom_in);
        dispatch_pointer_down_at(&mut ui, &mut host, &mut services, position);
        assert_eq!(
            ui.captured(),
            Some(zoom_in),
            "controls pressable should still capture pointer before focus restore"
        );

        dispatch_pointer_up_at(&mut ui, &mut host, &mut services, position);
        assert_eq!(ui.captured(), None);
        assert_eq!(
            ui.focus(),
            Some(surface),
            "controls activation should restore focus to the node graph surface target"
        );
        assert!(
            host.effects.iter().any(|effect| {
                matches!(
                    effect,
                    Effect::Command { window: effect_window, command }
                        if *effect_window == Some(window) && command == &custom_zoom
                )
            }),
            "controls command should still dispatch while restoring focus: {:?}",
            host.effects
        );
    }

    #[test]
    fn controls_declarative_keyboard_activation_restores_focus_to_surface_target() {
        let mut host = TestUiHost::default();
        let mut ui = fret_ui::UiTree::<TestUiHost>::new();
        let mut services = FakeUiServices;
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(160.0), Px(240.0)),
        );
        let custom_zoom = CommandId::from("node_graph.custom.zoom_in");
        let mut bindings = NodeGraphControlsBindings::default();
        bindings.zoom_in = NodeGraphControlsCommandBinding::Command(custom_zoom.clone());

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut host,
            &mut services,
            window,
            bounds,
            "controls-keyboard-activation-restores-surface-focus",
            |cx| {
                let mut stack = StackProps::default();
                stack.layout.size.width = Length::Fill;
                stack.layout.size.height = Length::Fill;
                vec![cx.stack_props(stack, move |cx| {
                    let mut surface_props = ContainerProps::default();
                    surface_props.layout.size.width = Length::Fill;
                    surface_props.layout.size.height = Length::Fill;
                    let surface = cx.container(surface_props, |_cx| Vec::new());
                    let surface_target = surface.id;

                    let controls = node_graph_controls_overlay_element(
                        cx,
                        NodeGraphControlsOverlayElementProps {
                            style: NodeGraphStyle::default(),
                            bindings,
                            connection_mode: NodeGraphConnectionMode::Strict,
                            focus_target: Some(surface_target),
                        },
                    );
                    vec![surface, controls]
                })]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut host, &mut services, bounds, 1.0);

        let stack = ui.children(root)[0];
        let surface = ui.children(stack)[0];
        let controls = ui.children(stack)[1];
        let pointer_region = ui.children(controls)[0];
        let panel = ui.children(pointer_region)[0];
        let column = ui.children(panel)[0];
        let zoom_in = ui.children(column)[1];
        ui.set_focus(Some(zoom_in));

        ui.dispatch_event(
            &mut host,
            &mut services,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::Enter,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        ui.dispatch_event(
            &mut host,
            &mut services,
            &fret_core::Event::KeyUp {
                key: fret_core::KeyCode::Enter,
                modifiers: Modifiers::default(),
            },
        );

        assert_eq!(
            ui.focus(),
            Some(surface),
            "controls keyboard activation should restore focus to the node graph surface target"
        );
        assert!(
            host.effects.iter().any(|effect| {
                matches!(
                    effect,
                    Effect::Command { window: effect_window, command }
                        if *effect_window == Some(window) && command == &custom_zoom
                )
            }),
            "controls keyboard activation should still dispatch the bound command: {:?}",
            host.effects
        );
    }

    #[test]
    fn controls_declarative_panel_blank_pointer_down_focuses_overlay_without_command() {
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
        bindings.zoom_in = NodeGraphControlsCommandBinding::Command(custom_zoom);

        let mut style = NodeGraphStyle::default();
        style.paint.controls_padding = 8.0;

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut host,
            &mut services,
            window,
            bounds,
            "controls-panel-blank-pointer-down",
            |cx| {
                vec![node_graph_controls_overlay_element(
                    cx,
                    NodeGraphControlsOverlayElementProps {
                        style,
                        bindings,
                        connection_mode: NodeGraphConnectionMode::Strict,
                        focus_target: None,
                    },
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut host, &mut services, bounds, 1.0);

        let controls = ui.children(root)[0];
        let pointer_region = ui.children(controls)[0];
        let panel = ui.children(pointer_region)[0];
        let panel_bounds = ui
            .debug_node_bounds(panel)
            .expect("panel should be laid out");
        let blank_position = Point::new(
            Px(panel_bounds.origin.x.0 + 2.0),
            Px(panel_bounds.origin.y.0 + 2.0),
        );

        ui.dispatch_event(
            &mut host,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position: blank_position,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        assert_eq!(
            ui.focus(),
            Some(controls),
            "blank panel pointer-down should focus the declarative controls host"
        );
        assert!(
            !host
                .effects
                .iter()
                .any(|effect| matches!(effect, Effect::Command { .. })),
            "blank panel pointer-down must not dispatch a controls command: {:?}",
            host.effects
        );
    }

    #[test]
    fn controls_declarative_pointer_events_fall_through_outside_panel_to_surface() {
        let mut host = TestUiHost::default();
        let mut ui = fret_ui::UiTree::<TestUiHost>::new();
        let mut services = FakeUiServices;
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(160.0), Px(240.0)),
        );
        let underlay_downs = host.models.insert(0_u32);

        let root = render_controls_with_recording_surface(
            &mut ui,
            &mut host,
            &mut services,
            window,
            bounds,
            "controls-pointer-fallthrough",
            NodeGraphStyle::default(),
            NodeGraphControlsBindings::default(),
            underlay_downs.clone(),
        );

        let stack = ui.children(root)[0];
        let surface = ui.children(stack)[0];
        let controls = ui.children(stack)[1];
        let pointer_region = ui.children(controls)[0];
        let panel = ui.children(pointer_region)[0];
        let surface_bounds = ui
            .debug_node_bounds(surface)
            .expect("surface should be laid out");
        let panel_bounds = ui
            .debug_node_bounds(panel)
            .expect("controls panel should be laid out");
        let position = Point::new(
            Px((panel_bounds.origin.x.0 + panel_bounds.size.width.0 + 8.0)
                .min(surface_bounds.origin.x.0 + surface_bounds.size.width.0 - 2.0)),
            Px(surface_bounds.origin.y.0 + surface_bounds.size.height.0 - 2.0),
        );

        dispatch_pointer_down_at(&mut ui, &mut host, &mut services, position);

        assert_eq!(
            underlay_downs
                .read_ref(&host, |count| *count)
                .expect("underlay counter"),
            1,
            "pointer-down outside the controls panel should fall through to the surface"
        );
    }

    #[test]
    fn controls_declarative_blocks_surface_input_within_panel_even_off_button() {
        let mut host = TestUiHost::default();
        let mut ui = fret_ui::UiTree::<TestUiHost>::new();
        let mut services = FakeUiServices;
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(160.0), Px(240.0)),
        );
        let mut style = NodeGraphStyle::default();
        style.paint.controls_padding = 8.0;
        let underlay_downs = host.models.insert(0_u32);

        let root = render_controls_with_recording_surface(
            &mut ui,
            &mut host,
            &mut services,
            window,
            bounds,
            "controls-panel-blocks-underlay",
            style,
            NodeGraphControlsBindings::default(),
            underlay_downs.clone(),
        );

        let stack = ui.children(root)[0];
        let surface = ui.children(stack)[0];
        let controls = ui.children(stack)[1];
        let pointer_region = ui.children(controls)[0];
        let panel = ui.children(pointer_region)[0];
        let panel_bounds = ui
            .debug_node_bounds(panel)
            .expect("controls panel should be laid out");
        let blank_position = Point::new(
            Px(panel_bounds.origin.x.0 + 2.0),
            Px(panel_bounds.origin.y.0 + 2.0),
        );

        ui.set_focus(Some(surface));
        dispatch_pointer_down_at(&mut ui, &mut host, &mut services, blank_position);

        assert_eq!(
            underlay_downs
                .read_ref(&host, |count| *count)
                .expect("underlay counter"),
            0,
            "blank panel pointer-down should not leak through to the surface"
        );
        assert_eq!(
            ui.focus(),
            Some(controls),
            "blank panel pointer-down should focus the controls root for keyboard follow-up"
        );
    }

    #[test]
    fn controls_declarative_focus_traversal_reaches_controls_from_surface() {
        let mut host = TestUiHost::default();
        host.commands.register(
            CommandId::from("focus.next"),
            CommandMeta::new("Focus Next").with_scope(CommandScope::Widget),
        );
        host.commands.register(
            CommandId::from("focus.previous"),
            CommandMeta::new("Focus Previous").with_scope(CommandScope::Widget),
        );

        let mut ui = fret_ui::UiTree::<TestUiHost>::new();
        let mut services = FakeUiServices;
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(160.0), Px(240.0)),
        );
        let underlay_downs = host.models.insert(0_u32);

        let root = render_controls_with_recording_surface(
            &mut ui,
            &mut host,
            &mut services,
            window,
            bounds,
            "controls-focus-traversal",
            NodeGraphStyle::default(),
            NodeGraphControlsBindings::default(),
            underlay_downs,
        );

        let stack = ui.children(root)[0];
        let surface = ui.children(stack)[0];
        let controls = ui.children(stack)[1];
        ui.set_focus(Some(surface));

        assert!(
            ui.dispatch_command(&mut host, &mut services, &CommandId::from("focus.next")),
            "focus.next should be handled by the declarative focus traversal path"
        );
        assert_eq!(
            ui.focus(),
            Some(controls),
            "focus traversal should reach the focusable controls root after the surface"
        );

        host.effects.clear();
        dispatch_key_down(
            &mut ui,
            &mut host,
            &mut services,
            fret_core::KeyCode::Escape,
        );
        assert_eq!(
            ui.focus(),
            Some(surface),
            "Escape from the traversed controls root should return focus to the surface target"
        );
        assert!(
            !host
                .effects
                .iter()
                .any(|effect| matches!(effect, Effect::Command { .. })),
            "Escape after focus traversal should not dispatch controls commands: {:?}",
            host.effects
        );
    }

    fn click_node(
        ui: &mut fret_ui::UiTree<TestUiHost>,
        host: &mut TestUiHost,
        services: &mut FakeUiServices,
        node: fret_core::NodeId,
    ) {
        let position = pointer_position_inside_node(ui, node);
        dispatch_pointer_down_at(ui, host, services, position);
        dispatch_pointer_up_at(ui, host, services, position);
    }

    #[allow(clippy::too_many_arguments)]
    fn render_controls_root(
        ui: &mut fret_ui::UiTree<TestUiHost>,
        host: &mut TestUiHost,
        services: &mut FakeUiServices,
        window: AppWindowId,
        bounds: Rect,
        root_name: &str,
        style: NodeGraphStyle,
        bindings: NodeGraphControlsBindings,
        focus_target: Option<GlobalElementId>,
    ) -> fret_core::NodeId {
        let root = fret_ui::declarative::render_root(
            ui,
            host,
            services,
            window,
            bounds,
            root_name,
            |cx| {
                vec![node_graph_controls_overlay_element(
                    cx,
                    NodeGraphControlsOverlayElementProps {
                        style,
                        bindings,
                        connection_mode: NodeGraphConnectionMode::Strict,
                        focus_target,
                    },
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(host, services, bounds, 1.0);
        root
    }

    fn rerender_controls_with_surface(
        ui: &mut fret_ui::UiTree<TestUiHost>,
        host: &mut TestUiHost,
        services: &mut FakeUiServices,
        window: AppWindowId,
        bounds: Rect,
        root_name: &str,
        bindings: NodeGraphControlsBindings,
    ) -> fret_core::NodeId {
        let root = fret_ui::declarative::render_root(
            ui,
            host,
            services,
            window,
            bounds,
            root_name,
            |cx| {
                let mut stack = StackProps::default();
                stack.layout.size.width = Length::Fill;
                stack.layout.size.height = Length::Fill;
                vec![cx.stack_props(stack, move |cx| {
                    let mut surface_props = ContainerProps::default();
                    surface_props.layout.size.width = Length::Fill;
                    surface_props.layout.size.height = Length::Fill;
                    let surface = cx.container(surface_props, |_cx| Vec::new());
                    let surface_target = surface.id;
                    let controls = node_graph_controls_overlay_element(
                        cx,
                        NodeGraphControlsOverlayElementProps {
                            style: NodeGraphStyle::default(),
                            bindings,
                            connection_mode: NodeGraphConnectionMode::Strict,
                            focus_target: Some(surface_target),
                        },
                    );
                    vec![surface, controls]
                })]
            },
        );
        ui.set_root(root);
        ui.layout_all(host, services, bounds, 1.0);
        root
    }

    #[allow(clippy::too_many_arguments)]
    fn render_controls_with_recording_surface(
        ui: &mut fret_ui::UiTree<TestUiHost>,
        host: &mut TestUiHost,
        services: &mut FakeUiServices,
        window: AppWindowId,
        bounds: Rect,
        root_name: &str,
        style: NodeGraphStyle,
        bindings: NodeGraphControlsBindings,
        underlay_downs: Model<u32>,
    ) -> fret_core::NodeId {
        let root = fret_ui::declarative::render_root(
            ui,
            host,
            services,
            window,
            bounds,
            root_name,
            |cx| {
                let mut stack = StackProps::default();
                stack.layout.size.width = Length::Fill;
                stack.layout.size.height = Length::Fill;
                vec![cx.stack_props(stack, move |cx| {
                    let mut surface_props = fret_ui::element::PointerRegionProps::default();
                    surface_props.layout.size.width = Length::Fill;
                    surface_props.layout.size.height = Length::Fill;

                    let surface_counter = underlay_downs.clone();
                    let surface_pointer = cx.pointer_region(surface_props, move |cx| {
                        cx.pointer_region_on_pointer_down(Arc::new(
                            move |host, _action_cx, down| {
                                if down.button != MouseButton::Left {
                                    return false;
                                }
                                let _ = host.models_mut().update(&surface_counter, |count| {
                                    *count = count.saturating_add(1);
                                });
                                true
                            },
                        ));
                        Vec::new()
                    });

                    let mut surface_semantics = fret_ui::element::SemanticsProps::default();
                    surface_semantics.layout.size.width = Length::Fill;
                    surface_semantics.layout.size.height = Length::Fill;
                    surface_semantics.role = SemanticsRole::Viewport;
                    surface_semantics.label = Some(Arc::from("Surface"));
                    surface_semantics.test_id = Some(Arc::from("node_graph.surface"));
                    surface_semantics.focusable = true;
                    let surface = cx.semantics(surface_semantics, move |_cx| vec![surface_pointer]);
                    let surface_target = surface.id;

                    let controls = node_graph_controls_overlay_element(
                        cx,
                        NodeGraphControlsOverlayElementProps {
                            style,
                            bindings,
                            connection_mode: NodeGraphConnectionMode::Strict,
                            focus_target: Some(surface_target),
                        },
                    );
                    vec![surface, controls]
                })]
            },
        );
        ui.set_root(root);
        ui.layout_all(host, services, bounds, 1.0);
        root
    }

    fn controls_semantics_value(
        ui: &mut fret_ui::UiTree<TestUiHost>,
        host: &mut TestUiHost,
        services: &mut FakeUiServices,
        bounds: Rect,
    ) -> Option<String> {
        ui.request_semantics_snapshot();
        ui.layout_all(host, services, bounds, 1.0);
        ui.semantics_snapshot()
            .expect("semantics snapshot")
            .nodes
            .iter()
            .find(|node| node.test_id.as_deref() == Some("node_graph.controls"))
            .and_then(|node| node.value.clone())
    }

    fn dispatch_key_down(
        ui: &mut fret_ui::UiTree<TestUiHost>,
        host: &mut TestUiHost,
        services: &mut FakeUiServices,
        key: fret_core::KeyCode,
    ) {
        ui.dispatch_event(
            host,
            services,
            &fret_core::Event::KeyDown {
                key,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
    }

    fn pointer_position_inside_node(
        ui: &fret_ui::UiTree<TestUiHost>,
        node: fret_core::NodeId,
    ) -> Point {
        let rect = ui.debug_node_bounds(node).expect("node should be laid out");
        Point::new(Px(rect.origin.x.0 + 1.0), Px(rect.origin.y.0 + 1.0))
    }

    fn dispatch_pointer_down_at(
        ui: &mut fret_ui::UiTree<TestUiHost>,
        host: &mut TestUiHost,
        services: &mut FakeUiServices,
        position: Point,
    ) {
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
    }

    fn dispatch_pointer_up_at(
        ui: &mut fret_ui::UiTree<TestUiHost>,
        host: &mut TestUiHost,
        services: &mut FakeUiServices,
        position: Point,
    ) {
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
