use std::sync::Arc;

use fret_core::{Corners, Edges, Px, Rect, SemanticsRole, TextStyle};
use fret_runtime::{CommandId, Model};
use fret_ui::element::{
    AnyElement, ContainerProps, InsetEdge, Length, PositionStyle, SemanticsDecoration,
    SpacingEdges, SpacingLength, TextInputProps,
};
use fret_ui::{ElementContext, GlobalElementId, Invalidation, TextInputStyle, UiHost};

use crate::ui::{NodeGraphStyle, NodeGraphSurfaceBinding};

use super::group_rename::NodeGraphOverlayState;
use super::rename_command::{
    RenameCommandOutcome, apply_rename_text_command, parse_rename_text_command,
    rename_cancel_command, rename_submit_command,
};
use super::rename_lifecycle::{RenameHostLifecyclePlan, plan_rename_host_lifecycle};
use super::rename_policy::{
    RenameOverlaySession, RenameOverlaySessionKey, active_rename_session, clear_rename_sessions,
    rename_session_seed_text,
};

#[derive(Debug, Clone)]
pub(super) struct NodeGraphRenameOverlayElementProps {
    pub(super) style: NodeGraphStyle,
    pub(super) bounds: Rect,
    pub(super) overlay_state: NodeGraphOverlayState,
    pub(super) rename_text: Model<String>,
    pub(super) last_opened_session: Option<RenameOverlaySessionKey>,
    pub(super) focus: Option<fret_core::NodeId>,
    pub(super) text_input_node: Option<fret_core::NodeId>,
}

#[derive(Debug, Clone)]
pub(super) struct NodeGraphRenameOverlayHostProps {
    pub(super) style: NodeGraphStyle,
    pub(super) bounds: Rect,
    pub(super) binding: NodeGraphSurfaceBinding,
    pub(super) overlay_state: Model<NodeGraphOverlayState>,
    pub(super) rename_text: Model<String>,
    pub(super) focus_restore: Option<GlobalElementId>,
}

#[derive(Debug, Clone, Default)]
struct RenameManagedHostState {
    last_opened_session: Option<RenameOverlaySessionKey>,
}

pub(super) fn node_graph_rename_overlay_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    props: NodeGraphRenameOverlayElementProps,
) -> Option<AnyElement> {
    let session = active_rename_session(&props.overlay_state)?;
    let plan = plan_rename_host_lifecycle(
        &props.style,
        props.bounds,
        Some(&session),
        props.text_input_node,
        props.focus,
        None,
        props.last_opened_session,
        |_| String::new(),
    );
    let RenameHostLifecyclePlan::Active { rect, .. } = plan else {
        return None;
    };

    let command_key = RenameOverlaySessionKey::from(session.key());
    let label = rename_session_label(&session);
    let test_id = rename_session_test_id(command_key);
    let input_test_id = rename_session_input_test_id(command_key);

    Some(
        cx.container(
            rename_container(props.bounds, rect, &props.style),
            move |cx| {
                vec![rename_text_input(
                    cx,
                    props.rename_text,
                    &props.style,
                    label,
                    input_test_id,
                    rename_submit_command(command_key),
                    rename_cancel_command(command_key),
                )]
            },
        )
        .attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Panel)
                .label(label)
                .test_id(test_id),
        ),
    )
}

pub(super) fn node_graph_rename_overlay_host_element<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    props: NodeGraphRenameOverlayHostProps,
) -> AnyElement {
    let NodeGraphRenameOverlayHostProps {
        style,
        bounds,
        binding,
        overlay_state,
        rename_text,
        focus_restore,
    } = props;
    let text_input_id = cx.keyed_slot_id("rename.text_input");
    let host_state = cx.local_model_keyed("rename.host_state", RenameManagedHostState::default);
    let overlay_snapshot = cx
        .get_model_cloned(&overlay_state, Invalidation::Layout)
        .unwrap_or_default();
    let graph_snapshot = cx
        .get_model_cloned(&binding.graph_model(), Invalidation::Layout)
        .unwrap_or_default();

    let mut surface = fret_ui::element::ManagedSurfaceProps::default();
    surface.layout.position = PositionStyle::Absolute;
    surface.layout.inset.left = InsetEdge::Px(Px(0.0));
    surface.layout.inset.top = InsetEdge::Px(Px(0.0));
    surface.layout.size.width = Length::Fill;
    surface.layout.size.height = Length::Fill;

    let binding_for_command = binding.clone();
    let style_for_layout = style.clone();
    let style_for_child = style.clone();
    let overlay_state_for_layout = overlay_state.clone();
    let overlay_state_for_command = overlay_state.clone();
    let rename_text_for_layout = rename_text.clone();
    let rename_text_for_command = rename_text.clone();
    let rename_text_for_child = rename_text.clone();
    let focus_restore_for_command = focus_restore;
    let focus_restore_for_layout = focus_restore;
    let host_state_for_layout = host_state.clone();

    let element = cx.managed_surface(
        surface,
        move |cx| {
            let child = cx.children().first().copied();
            let session = cx
                .app()
                .models()
                .read(&overlay_state_for_layout, active_rename_session)
                .ok()
                .flatten();
            let focus = cx.focus();
            let restore_focus_marker = focus_restore_for_layout.map(|_| cx.node());
            let bounds = cx.bounds();
            let lifecycle_focus_node = if cx.focus_in_subtree() { focus } else { child };
            let last_opened_session = cx
                .app()
                .models()
                .read(&host_state_for_layout, |state| state.last_opened_session)
                .ok()
                .flatten();
            let plan = {
                let graph_snapshot = graph_snapshot.clone();
                plan_rename_host_lifecycle(
                    &style_for_layout,
                    bounds,
                    session.as_ref(),
                    lifecycle_focus_node,
                    focus,
                    restore_focus_marker,
                    last_opened_session,
                    |session| rename_session_seed_text(&graph_snapshot, session),
                )
            };

            match plan {
                RenameHostLifecyclePlan::CancelActiveSession { focus_restore } => {
                    let _ = cx
                        .app()
                        .models_mut()
                        .update(&overlay_state_for_layout, |state| {
                            clear_rename_sessions(state);
                        });
                    if let Some(child) = child {
                        cx.layout_child(
                            child,
                            Rect::new(bounds.origin, fret_core::Size::new(Px(0.0), Px(0.0))),
                        );
                    }
                    if focus_restore.is_some()
                        && let Some(target) = focus_restore_for_layout
                    {
                        cx.request_focus_element(target);
                    }
                    cx.set_hit_test_rects([]);
                    let _ = cx
                        .app()
                        .models_mut()
                        .update(&host_state_for_layout, |state| {
                            state.last_opened_session = None;
                        });
                    cx.request_redraw();
                }
                RenameHostLifecyclePlan::Active {
                    rect,
                    session_key,
                    seed_text,
                    focus_request,
                    ..
                } => {
                    if let Some(seed_text) = seed_text {
                        let _ = cx
                            .app()
                            .models_mut()
                            .update(&rename_text_for_layout, |text| {
                                *text = seed_text;
                            });
                    }
                    if focus_request.is_some() {
                        cx.request_focus_element(text_input_id);
                    }
                    if let Some(child) = child {
                        cx.layout_child(child, rect);
                    }
                    cx.set_hit_test_rects([rect]);
                    let _ = cx
                        .app()
                        .models_mut()
                        .update(&host_state_for_layout, |state| {
                            state.last_opened_session = Some(session_key);
                        });
                }
                RenameHostLifecyclePlan::Hidden { focus_restore } => {
                    if let Some(child) = child {
                        cx.layout_child(
                            child,
                            Rect::new(bounds.origin, fret_core::Size::new(Px(0.0), Px(0.0))),
                        );
                    }
                    if focus_restore.is_some()
                        && let Some(target) = focus_restore_for_layout
                    {
                        cx.request_focus_element(target);
                    }
                    cx.set_hit_test_rects([]);
                    let _ = cx
                        .app()
                        .models_mut()
                        .update(&host_state_for_layout, |state| {
                            state.last_opened_session = None;
                        });
                }
            }
        },
        move |cx| {
            for child in cx.children().to_vec() {
                if let Some(bounds) = cx.child_bounds(child) {
                    cx.paint_child(child, bounds);
                }
            }
        },
        move |cx| {
            let overlay_state_value = overlay_snapshot.clone();
            let rename_child = node_graph_rename_overlay_element(
                cx,
                NodeGraphRenameOverlayElementProps {
                    style: style_for_child.clone(),
                    bounds,
                    overlay_state: overlay_state_value,
                    rename_text: rename_text_for_child.clone(),
                    last_opened_session: None,
                    focus: None,
                    text_input_node: None,
                },
            );
            rename_child
                .map(|element| vec![replace_text_input_id(element, text_input_id)])
                .unwrap_or_default()
        },
    );

    cx.managed_surface_on_command_for(element.id, move |cx, command| {
        let Some(command) = parse_rename_text_command(command) else {
            return false;
        };
        let graph = cx
            .app()
            .models()
            .read(&binding_for_command.graph_model(), Clone::clone)
            .ok()
            .unwrap_or_default();
        let rename_text = cx
            .app()
            .models()
            .read(&rename_text_for_command, Clone::clone)
            .ok()
            .unwrap_or_default();
        let outcome = cx
            .app()
            .models_mut()
            .update(&overlay_state_for_command, |state| {
                apply_rename_text_command(&graph, state, &rename_text, command)
            })
            .ok()
            .unwrap_or(RenameCommandOutcome::NotHandled);

        match outcome {
            RenameCommandOutcome::NotHandled => false,
            RenameCommandOutcome::Handled => {
                if let Some(target) = focus_restore_for_command {
                    cx.request_focus_element(target);
                }
                cx.notify();
                true
            }
            RenameCommandOutcome::Commit(tx) => {
                let _ = binding_for_command.submit_transaction(cx.app(), &tx);
                if let Some(target) = focus_restore_for_command {
                    cx.request_focus_element(target);
                }
                cx.notify();
                true
            }
        }
    });

    element
}

fn replace_text_input_id(mut element: AnyElement, text_input_id: GlobalElementId) -> AnyElement {
    if matches!(element.kind, fret_ui::element::ElementKind::TextInput(_)) {
        element.id = text_input_id;
        return element;
    }
    element.children = element
        .children
        .into_iter()
        .map(|child| replace_text_input_id(child, text_input_id))
        .collect();
    element
}

fn rename_container(bounds: Rect, rect: Rect, style: &NodeGraphStyle) -> ContainerProps {
    let mut props = ContainerProps::default();
    props.layout.position = PositionStyle::Absolute;
    props.layout.inset.left = InsetEdge::Px(Px(rect.origin.x.0 - bounds.origin.x.0));
    props.layout.inset.top = InsetEdge::Px(Px(rect.origin.y.0 - bounds.origin.y.0));
    props.layout.size.width = Length::Px(rect.size.width);
    props.layout.size.height = Length::Px(rect.size.height);
    props.padding = SpacingEdges::all(SpacingLength::Px(Px(style
        .paint
        .context_menu_padding
        .max(0.0))));
    props.background = Some(style.paint.context_menu_background);
    props.border = Edges::all(Px(1.0));
    props.border_color = Some(style.paint.context_menu_border);
    props.corner_radii = Corners::all(Px(style.paint.context_menu_corner_radius.max(0.0)));
    props.snap_to_device_pixels = true;
    props
}

fn rename_text_input<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<String>,
    style: &NodeGraphStyle,
    label: &'static str,
    test_id: Arc<str>,
    submit_command: CommandId,
    cancel_command: CommandId,
) -> AnyElement {
    let mut props = TextInputProps::new(model);
    props.layout.size.width = Length::Fill;
    props.layout.size.height = Length::Fill;
    props.a11y_role = Some(SemanticsRole::TextField);
    props.a11y_label = Some(Arc::from(label));
    props.test_id = Some(test_id);
    props.placeholder = Some(Arc::from("Rename"));
    props.chrome = rename_text_input_chrome(style);
    props.text_style = rename_text_style(style);
    props.submit_command = Some(submit_command);
    props.cancel_command = Some(cancel_command);

    cx.text_input(props)
}

fn rename_text_input_chrome(style: &NodeGraphStyle) -> TextInputStyle {
    let mut chrome = TextInputStyle::default();
    chrome.padding = Edges::all(Px(0.0));
    chrome.background = style.paint.context_menu_background;
    chrome.border = Edges::all(Px(0.0));
    chrome.border_color = style.paint.context_menu_border;
    chrome.border_color_focused = style.paint.context_menu_border;
    chrome.focus_ring = None;
    chrome.corner_radii = Corners::all(Px(0.0));
    chrome.text_color = style.paint.context_menu_text;
    chrome.placeholder_color = style.paint.context_menu_text_disabled;
    chrome.caret_color = style.paint.context_menu_text;
    chrome
}

fn rename_text_style(style: &NodeGraphStyle) -> TextStyle {
    style.geometry.context_menu_text_style.clone()
}

fn rename_session_label(session: &RenameOverlaySession) -> &'static str {
    match session {
        RenameOverlaySession::Group(_) => "Rename group",
        RenameOverlaySession::Symbol(_) => "Rename symbol",
    }
}

fn rename_session_test_id(session: RenameOverlaySessionKey) -> Arc<str> {
    match session {
        RenameOverlaySessionKey::Group(group) => {
            Arc::from(format!("node_graph.rename.group.{}", group.0))
        }
        RenameOverlaySessionKey::Symbol(symbol) => {
            Arc::from(format!("node_graph.rename.symbol.{}", symbol.0))
        }
    }
}

fn rename_session_input_test_id(session: RenameOverlaySessionKey) -> Arc<str> {
    match session {
        RenameOverlaySessionKey::Group(group) => {
            Arc::from(format!("node_graph.rename.group.{}.input", group.0))
        }
        RenameOverlaySessionKey::Symbol(symbol) => {
            Arc::from(format!("node_graph.rename.symbol.{}.input", symbol.0))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::any::{Any, TypeId};
    use std::collections::HashMap;
    use std::sync::Arc;

    use fret_core::{
        AppWindowId, KeyCode, MaterialDescriptor, MaterialId, MaterialRegistrationError, Modifiers,
        MouseButton, PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle,
        Point, PointerEvent, PointerId, PointerType, Px, Rect, SemanticsRole, Size, SvgId,
        SvgService, TextConstraints, TextMetrics, TextService,
    };
    use fret_runtime::{
        ClipboardToken, CommandId, CommandRegistry, CommandsHost, DragHost, DragKindId,
        DragSession, Effect, EffectSink, FrameId, GlobalsHost, ImageUploadToken, ModelHost,
        ModelId, ModelStore, ModelsHost, ShareSheetToken, TickId, TimeHost, TimerToken,
    };
    use fret_ui::element::{
        ElementKind, InsetEdge, Length, PointerRegionProps, PositionStyle, SemanticsProps,
        StackProps,
    };

    use crate::core::{
        CanvasPoint, CanvasRect, CanvasSize, Graph, GraphId, Group, GroupId, Symbol, SymbolId,
    };
    use crate::io::{NodeGraphEditorConfig, NodeGraphViewState};
    use crate::ui::overlays::group_rename::{
        GroupRenameOverlay, NodeGraphOverlayState, SymbolRenameOverlay,
    };
    use crate::ui::overlays::rename_command::{
        RenameTextCommand, parse_rename_text_command, rename_cancel_command, rename_submit_command,
    };
    use crate::ui::overlays::rename_declarative::{
        NodeGraphRenameOverlayElementProps, NodeGraphRenameOverlayHostProps,
        node_graph_rename_overlay_element, node_graph_rename_overlay_host_element,
    };
    use crate::ui::overlays::rename_host_layout::{RenameHostLayoutPlan, plan_rename_host_layout};
    use crate::ui::overlays::rename_policy::{RenameOverlaySessionKey, active_rename_session};
    use crate::ui::{NodeGraphStyle, NodeGraphSurfaceBinding};

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
        fn request_redraw(&mut self, window: AppWindowId) {
            self.effects.push(Effect::Redraw(window));
        }

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
                    size: Size::new(Px(10.0), Px(10.0)),
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

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        )
    }

    fn render_rename(
        host: &mut TestUiHost,
        overlay_state: NodeGraphOverlayState,
        text_model: fret_runtime::Model<String>,
    ) -> Option<fret_ui::element::AnyElement> {
        let mut runtime = fret_ui::ElementRuntime::new();
        let window = AppWindowId::default();
        let mut cx = fret_ui::ElementContext::new_for_root_name(
            host,
            &mut runtime,
            window,
            bounds(),
            "root",
        );
        node_graph_rename_overlay_element(
            &mut cx,
            NodeGraphRenameOverlayElementProps {
                style: NodeGraphStyle::default(),
                bounds: bounds(),
                overlay_state,
                rename_text: text_model,
                last_opened_session: None,
                focus: None,
                text_input_node: None,
            },
        )
    }

    fn graph_with_group_and_symbol() -> (Graph, GroupId, SymbolId) {
        let group = GroupId::from_u128(0x11111111111111111111111111111111);
        let symbol = SymbolId::from_u128(0x22222222222222222222222222222222);
        let mut graph = Graph::new(GraphId::from_u128(0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa));
        graph.groups.insert(
            group,
            Group {
                title: "Group A".to_string(),
                rect: CanvasRect {
                    origin: CanvasPoint { x: 0.0, y: 0.0 },
                    size: CanvasSize {
                        width: 100.0,
                        height: 40.0,
                    },
                },
                color: None,
            },
        );
        graph.symbols.insert(
            symbol,
            Symbol {
                name: "Symbol A".to_string(),
                ty: None,
                default_value: None,
                meta: serde_json::Value::Null,
            },
        );
        (graph, group, symbol)
    }

    #[allow(clippy::type_complexity)]
    fn binding_and_group_overlay(
        host: &mut TestUiHost,
    ) -> (
        NodeGraphSurfaceBinding,
        fret_runtime::Model<NodeGraphOverlayState>,
        fret_runtime::Model<String>,
        GroupId,
        SymbolId,
    ) {
        let (graph, group, symbol) = graph_with_group_and_symbol();
        let binding = NodeGraphSurfaceBinding::new(
            host.models_mut(),
            graph,
            NodeGraphViewState::default(),
            NodeGraphEditorConfig::default(),
        );
        let overlays = host.models_mut().insert(NodeGraphOverlayState {
            group_rename: Some(GroupRenameOverlay {
                group,
                invoked_at_window: Point::new(Px(100.0), Px(120.0)),
            }),
            symbol_rename: None,
        });
        let rename_text = host.models_mut().insert(String::new());
        (binding, overlays, rename_text, group, symbol)
    }

    #[allow(clippy::too_many_arguments)]
    fn render_rename_host_with_surface(
        ui: &mut fret_ui::UiTree<TestUiHost>,
        host: &mut TestUiHost,
        services: &mut FakeUiServices,
        window: AppWindowId,
        root_name: &str,
        binding: NodeGraphSurfaceBinding,
        overlay_state: fret_runtime::Model<NodeGraphOverlayState>,
        rename_text: fret_runtime::Model<String>,
        underlay_downs: fret_runtime::Model<u32>,
    ) -> fret_core::NodeId {
        let root = fret_ui::declarative::render_root(
            ui,
            host,
            services,
            window,
            bounds(),
            root_name,
            |cx| {
                let mut stack = StackProps::default();
                stack.layout.size.width = Length::Fill;
                stack.layout.size.height = Length::Fill;
                vec![cx.stack_props(stack, move |cx| {
                    let mut pointer = PointerRegionProps::default();
                    pointer.layout.size.width = Length::Fill;
                    pointer.layout.size.height = Length::Fill;
                    let underlay_downs = underlay_downs.clone();
                    let surface_pointer = cx.pointer_region(pointer, move |cx| {
                        cx.pointer_region_on_pointer_down(Arc::new(
                            move |host, _action_cx, down| {
                                if down.button != MouseButton::Left {
                                    return false;
                                }
                                let _ = host.models_mut().update(&underlay_downs, |count| {
                                    *count = count.saturating_add(1);
                                });
                                true
                            },
                        ));
                        Vec::new()
                    });

                    let mut surface_props = SemanticsProps::default();
                    surface_props.layout.size.width = Length::Fill;
                    surface_props.layout.size.height = Length::Fill;
                    surface_props.role = SemanticsRole::Viewport;
                    surface_props.label = Some(Arc::from("Surface"));
                    surface_props.test_id = Some(Arc::from("node_graph.surface"));
                    surface_props.focusable = true;
                    let surface = cx.semantics(surface_props, move |_cx| vec![surface_pointer]);
                    let focus_restore = surface.id;

                    let rename = node_graph_rename_overlay_host_element(
                        cx,
                        NodeGraphRenameOverlayHostProps {
                            style: NodeGraphStyle::default(),
                            bounds: bounds(),
                            binding,
                            overlay_state,
                            rename_text,
                            focus_restore: Some(focus_restore),
                        },
                    );
                    vec![surface, rename]
                })]
            },
        );
        ui.set_root(root);
        ui.layout_all(host, services, bounds(), 1.0);
        root
    }

    fn rename_host_nodes(
        ui: &fret_ui::UiTree<TestUiHost>,
        root: fret_core::NodeId,
    ) -> (
        fret_core::NodeId,
        fret_core::NodeId,
        fret_core::NodeId,
        fret_core::NodeId,
    ) {
        let stack = ui.children(root)[0];
        let surface = ui.children(stack)[0];
        let rename_host = ui.children(stack)[1];
        let rename_panel = ui.children(rename_host)[0];
        let rename_input = ui.children(rename_panel)[0];
        (surface, rename_host, rename_panel, rename_input)
    }

    fn dispatch_key_down(
        ui: &mut fret_ui::UiTree<TestUiHost>,
        host: &mut TestUiHost,
        services: &mut FakeUiServices,
        key: KeyCode,
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

    fn dispatch_pointer_down_at(
        ui: &mut fret_ui::UiTree<TestUiHost>,
        host: &mut TestUiHost,
        services: &mut FakeUiServices,
        position: Point,
    ) {
        ui.dispatch_event(
            host,
            services,
            &fret_core::Event::Pointer(PointerEvent::Down {
                position,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );
    }

    fn take_last_command(host: &mut TestUiHost) -> Option<CommandId> {
        host.effects.iter().rev().find_map(|effect| match effect {
            Effect::Command { command, .. } => Some(command.clone()),
            _ => None,
        })
    }

    fn dispatch_rename_text_command(
        ui: &mut fret_ui::UiTree<TestUiHost>,
        host: &mut TestUiHost,
        services: &mut FakeUiServices,
        command: CommandId,
    ) {
        assert!(
            ui.dispatch_command(host, services, &command),
            "rename command should be handled by the managed host"
        );
    }

    fn rerender_rename_host(
        ui: &mut fret_ui::UiTree<TestUiHost>,
        host: &mut TestUiHost,
        services: &mut FakeUiServices,
        window: AppWindowId,
        root_name: &str,
        binding: NodeGraphSurfaceBinding,
        overlay_state: fret_runtime::Model<NodeGraphOverlayState>,
        rename_text: fret_runtime::Model<String>,
        underlay_downs: fret_runtime::Model<u32>,
    ) -> fret_core::NodeId {
        render_rename_host_with_surface(
            ui,
            host,
            services,
            window,
            root_name,
            binding,
            overlay_state,
            rename_text,
            underlay_downs,
        )
    }

    #[test]
    fn rename_managed_host_seeds_focuses_and_masks_hit_testing_without_retained_host() {
        let mut host = TestUiHost::default();
        let mut ui = fret_ui::UiTree::<TestUiHost>::new();
        let mut services = FakeUiServices;
        let window = AppWindowId::default();
        ui.set_window(window);
        let (binding, overlays, rename_text, group, _) = binding_and_group_overlay(&mut host);
        let underlay_downs = host.models_mut().insert(0_u32);

        let root = render_rename_host_with_surface(
            &mut ui,
            &mut host,
            &mut services,
            window,
            "rename-managed-host-seed-focus-hit-test",
            binding,
            overlays.clone(),
            rename_text.clone(),
            underlay_downs.clone(),
        );
        let (surface, _rename_host, rename_panel, rename_input) = rename_host_nodes(&ui, root);

        assert_eq!(
            rename_text
                .read_ref(&host, Clone::clone)
                .expect("rename text"),
            "Group A",
            "managed host should seed the caller-owned rename text model from the graph"
        );
        assert_eq!(
            ui.focus(),
            Some(rename_input),
            "new rename sessions should focus the declarative text input"
        );
        assert!(
            overlays
                .read_ref(&host, |state| state
                    .group_rename
                    .as_ref()
                    .map(|rename| rename.group))
                .expect("overlay state")
                == Some(group)
        );

        let panel_bounds = ui
            .debug_node_bounds(rename_panel)
            .expect("rename panel should be laid out");
        let input_bounds = ui
            .debug_node_bounds(rename_input)
            .expect("rename input should be laid out");
        assert_eq!(
            ui.debug_hit_test(Point::new(
                Px(input_bounds.origin.x.0 + 1.0),
                Px(input_bounds.origin.y.0 + 1.0)
            ))
            .hit,
            Some(rename_input),
            "inside the rename input bounds should hit the declarative input child"
        );
        assert_eq!(
            ui.debug_hit_test(Point::new(
                Px(panel_bounds.origin.x.0 + 1.0),
                Px(panel_bounds.origin.y.0 + 1.0)
            ))
            .hit,
            Some(rename_panel),
            "panel padding should still block the underlying surface"
        );

        let surface_bounds = ui
            .debug_node_bounds(surface)
            .expect("surface should be laid out");
        let outside_panel = Point::new(
            Px(surface_bounds.origin.x.0 + 4.0),
            Px(surface_bounds.origin.y.0 + 4.0),
        );
        assert_eq!(
            ui.debug_hit_test(outside_panel).hit,
            Some(ui.children(surface)[0]),
            "outside the host-selected rename rect should fall through to the surface"
        );
        dispatch_pointer_down_at(&mut ui, &mut host, &mut services, outside_panel);
        assert_eq!(
            underlay_downs
                .read_ref(&host, |count| *count)
                .expect("underlay counter"),
            1,
            "pointer down outside the rename rect should reach the underlying surface"
        );
    }

    #[test]
    fn rename_managed_host_submit_commits_through_surface_binding_and_restores_focus() {
        let mut host = TestUiHost::default();
        let mut ui = fret_ui::UiTree::<TestUiHost>::new();
        let mut services = FakeUiServices;
        let window = AppWindowId::default();
        ui.set_window(window);
        let (binding, overlays, rename_text, group, _) = binding_and_group_overlay(&mut host);
        let underlay_downs = host.models_mut().insert(0_u32);

        let root = render_rename_host_with_surface(
            &mut ui,
            &mut host,
            &mut services,
            window,
            "rename-managed-host-submit",
            binding.clone(),
            overlays.clone(),
            rename_text.clone(),
            underlay_downs,
        );
        let (surface, _rename_host, _rename_panel, rename_input) = rename_host_nodes(&ui, root);
        assert_eq!(ui.focus(), Some(rename_input));

        rename_text
            .update(&mut host, |text, _host| {
                *text = "Group B".to_string();
            })
            .expect("update rename text");
        dispatch_rename_text_command(
            &mut ui,
            &mut host,
            &mut services,
            rename_submit_command(RenameOverlaySessionKey::Group(group)),
        );

        assert!(
            overlays
                .read_ref(&host, |state| state.group_rename.is_none()
                    && state.symbol_rename.is_none())
                .expect("overlay state"),
            "submit should close the active rename session"
        );
        let graph = binding
            .graph_model()
            .read_ref(&host, Clone::clone)
            .expect("graph model");
        assert_eq!(
            graph.groups.get(&group).map(|group| group.title.as_str()),
            Some("Group B")
        );
        let undo_len = binding
            .store_model()
            .read_ref(&host, |store| store.history().undo_len())
            .expect("store undo len");
        assert_eq!(undo_len, 1, "rename submit should enter graph history");
        assert_eq!(
            ui.focus(),
            Some(surface),
            "submit should restore focus to the node graph surface target"
        );
    }

    #[test]
    fn rename_managed_host_escape_closes_without_transaction_and_restores_focus() {
        let mut host = TestUiHost::default();
        let mut ui = fret_ui::UiTree::<TestUiHost>::new();
        let mut services = FakeUiServices;
        let window = AppWindowId::default();
        ui.set_window(window);
        let (binding, overlays, rename_text, group, _) = binding_and_group_overlay(&mut host);
        let underlay_downs = host.models_mut().insert(0_u32);

        let root = render_rename_host_with_surface(
            &mut ui,
            &mut host,
            &mut services,
            window,
            "rename-managed-host-escape",
            binding.clone(),
            overlays.clone(),
            rename_text.clone(),
            underlay_downs,
        );
        let (surface, _rename_host, _rename_panel, rename_input) = rename_host_nodes(&ui, root);
        assert_eq!(ui.focus(), Some(rename_input));

        dispatch_key_down(&mut ui, &mut host, &mut services, KeyCode::Escape);
        let cancel_command = take_last_command(&mut host).expect("cancel command effect");
        assert_eq!(
            cancel_command,
            rename_cancel_command(RenameOverlaySessionKey::Group(group)),
            "Escape in the declarative text input should dispatch the session cancel command"
        );
        dispatch_rename_text_command(&mut ui, &mut host, &mut services, cancel_command);
        assert!(
            overlays
                .read_ref(&host, |state| state.group_rename.is_none()
                    && state.symbol_rename.is_none())
                .expect("overlay state"),
            "Escape should close the active rename session"
        );
        let graph = binding
            .graph_model()
            .read_ref(&host, Clone::clone)
            .expect("graph model");
        assert_eq!(
            graph.groups.get(&group).map(|group| group.title.as_str()),
            Some("Group A"),
            "cancel should not mutate the graph"
        );
        let undo_len = binding
            .store_model()
            .read_ref(&host, |store| store.history().undo_len())
            .expect("store undo len");
        assert_eq!(undo_len, 0, "cancel should not enter graph history");
        assert_eq!(
            ui.focus(),
            Some(surface),
            "cancel should restore focus to the node graph surface target"
        );
    }

    #[test]
    fn rename_managed_host_focus_loss_closes_without_transaction_or_focus_steal() {
        let mut host = TestUiHost::default();
        let mut ui = fret_ui::UiTree::<TestUiHost>::new();
        let mut services = FakeUiServices;
        let window = AppWindowId::default();
        ui.set_window(window);
        let (binding, overlays, rename_text, group, _) = binding_and_group_overlay(&mut host);
        let underlay_downs = host.models_mut().insert(0_u32);
        let root_name = "rename-managed-host-focus-loss";

        let root = render_rename_host_with_surface(
            &mut ui,
            &mut host,
            &mut services,
            window,
            root_name,
            binding.clone(),
            overlays.clone(),
            rename_text.clone(),
            underlay_downs.clone(),
        );
        let (surface, _rename_host, _rename_panel, rename_input) = rename_host_nodes(&ui, root);
        assert_eq!(ui.focus(), Some(rename_input));

        ui.set_focus(Some(surface));
        rerender_rename_host(
            &mut ui,
            &mut host,
            &mut services,
            window,
            root_name,
            binding.clone(),
            overlays.clone(),
            rename_text,
            underlay_downs,
        );

        assert!(
            overlays
                .read_ref(&host, |state| state.group_rename.is_none()
                    && state.symbol_rename.is_none())
                .expect("overlay state"),
            "focus loss after the open frame should close the active rename session"
        );
        assert_eq!(
            ui.focus(),
            Some(surface),
            "focus loss should keep the new focus owner instead of restoring over it"
        );
        let graph = binding
            .graph_model()
            .read_ref(&host, Clone::clone)
            .expect("graph model");
        assert_eq!(
            graph.groups.get(&group).map(|group| group.title.as_str()),
            Some("Group A")
        );
        let undo_len = binding
            .store_model()
            .read_ref(&host, |store| store.history().undo_len())
            .expect("store undo len");
        assert_eq!(undo_len, 0, "focus-loss close should not submit a tx");
    }

    #[test]
    fn rename_text_command_protocol_roundtrips_group_and_symbol_sessions() {
        let group = GroupId::from_u128(0x11111111111111111111111111111111);
        let symbol = SymbolId::from_u128(0x22222222222222222222222222222222);

        assert_eq!(
            parse_rename_text_command(&rename_submit_command(RenameOverlaySessionKey::Group(
                group
            ))),
            Some(RenameTextCommand::Submit {
                session: RenameOverlaySessionKey::Group(group)
            })
        );
        assert_eq!(
            parse_rename_text_command(&rename_cancel_command(RenameOverlaySessionKey::Symbol(
                symbol
            ))),
            Some(RenameTextCommand::Cancel {
                session: RenameOverlaySessionKey::Symbol(symbol)
            })
        );
    }

    #[test]
    fn rename_declarative_returns_none_without_active_session() {
        let mut host = TestUiHost::default();
        let text = host.models_mut().insert(String::new());

        let root = render_rename(&mut host, NodeGraphOverlayState::default(), text);

        assert!(root.is_none());
    }

    #[test]
    fn rename_declarative_builds_group_text_input_and_preserves_bound_text_model() {
        let mut host = TestUiHost::default();
        let group = GroupId::from_u128(0x11111111111111111111111111111111);
        let text = host.models_mut().insert(String::from("Group A"));
        let overlay_state = NodeGraphOverlayState {
            group_rename: Some(GroupRenameOverlay {
                group,
                invoked_at_window: Point::new(Px(780.0), Px(590.0)),
            }),
            symbol_rename: None,
        };
        let session = active_rename_session(&overlay_state).expect("group rename session");
        let expected_plan = plan_rename_host_layout(
            &NodeGraphStyle::default(),
            bounds(),
            Some(&session),
            None,
            None,
            None,
        );
        let RenameHostLayoutPlan::Active { rect, .. } = expected_plan else {
            panic!("expected active rename layout plan");
        };

        let root = render_rename(&mut host, overlay_state, text.clone())
            .expect("active group rename element");

        let expected_left = InsetEdge::Px(Px(rect.origin.x.0 - bounds().origin.x.0));
        let expected_top = InsetEdge::Px(Px(rect.origin.y.0 - bounds().origin.y.0));

        let ElementKind::Container(container) = &root.kind else {
            panic!("rename root should be a declarative container");
        };
        assert_eq!(container.layout.position, PositionStyle::Absolute);
        assert_eq!(container.layout.inset.left, expected_left);
        assert_eq!(container.layout.inset.top, expected_top);

        let semantics = root.semantics_decoration.as_ref().expect("root semantics");
        assert_eq!(semantics.role, Some(SemanticsRole::Panel));
        assert_eq!(semantics.label.as_deref(), Some("Rename group"));
        assert_eq!(
            semantics.test_id.as_deref(),
            Some("node_graph.rename.group.11111111-1111-1111-1111-111111111111")
        );

        assert_eq!(
            host.models().get_cloned(&text).as_deref(),
            Some("Group A"),
            "declarative rename should preserve the caller-owned bound text model"
        );

        assert_eq!(root.children.len(), 1);
        let ElementKind::TextInput(input) = &root.children[0].kind else {
            panic!("rename root should contain a declarative text input");
        };
        assert_eq!(input.layout.size.width, Length::Fill);
        assert_eq!(input.layout.size.height, Length::Fill);
        assert_eq!(input.a11y_role, Some(SemanticsRole::TextField));
        assert_eq!(input.a11y_label.as_deref(), Some("Rename group"));
        assert_eq!(
            input.test_id.as_deref(),
            Some("node_graph.rename.group.11111111-1111-1111-1111-111111111111.input")
        );
        assert_eq!(
            input.submit_command,
            Some(rename_submit_command(RenameOverlaySessionKey::Group(group)))
        );
        assert_eq!(
            input.cancel_command,
            Some(rename_cancel_command(RenameOverlaySessionKey::Group(group)))
        );
    }

    #[test]
    fn rename_declarative_builds_symbol_text_input() {
        let mut host = TestUiHost::default();
        let symbol = SymbolId::from_u128(0x22222222222222222222222222222222);
        let text = host.models_mut().insert(String::from("Symbol A"));

        let root = render_rename(
            &mut host,
            NodeGraphOverlayState {
                group_rename: None,
                symbol_rename: Some(SymbolRenameOverlay {
                    symbol,
                    invoked_at_window: Point::new(Px(40.0), Px(50.0)),
                }),
            },
            text.clone(),
        )
        .expect("active symbol rename element");

        let semantics = root.semantics_decoration.as_ref().expect("root semantics");
        assert_eq!(semantics.label.as_deref(), Some("Rename symbol"));
        assert_eq!(
            semantics.test_id.as_deref(),
            Some("node_graph.rename.symbol.22222222-2222-2222-2222-222222222222")
        );
        assert_eq!(host.models().get_cloned(&text).as_deref(), Some("Symbol A"));

        let ElementKind::TextInput(input) = &root.children[0].kind else {
            panic!("rename root should contain a declarative text input");
        };
        assert_eq!(
            input.submit_command,
            Some(rename_submit_command(RenameOverlaySessionKey::Symbol(
                symbol
            )))
        );
        assert_eq!(
            input.cancel_command,
            Some(rename_cancel_command(RenameOverlaySessionKey::Symbol(
                symbol
            )))
        );
    }
}
