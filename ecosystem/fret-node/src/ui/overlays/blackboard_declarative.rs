use std::collections::BTreeMap;
use std::sync::Arc;

use fret_core::{Corners, Edges, Point, Px, Rect, SemanticsRole, Size, TextOverflow, TextWrap};
use fret_ui::action::{
    ActionCx, ActivateReason, PressablePointerDownResult, PressablePointerUpResult, UiActionHost,
};
use fret_ui::element::{
    AnyElement, ColumnProps, ContainerProps, CrossAlign, Length, MainAlign, PointerRegionProps,
    PressableProps, RowProps, SemanticsProps, SpacingEdges, SpacingLength, TextProps,
};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::core::{Symbol, SymbolId};
use crate::ui::{NodeGraphStyle, NodeGraphSurfaceBinding};

use super::blackboard_interaction_policy::{
    BlackboardInteractionState, BlackboardKeyboardInteractionPlan,
    plan_blackboard_keyboard_interaction, plan_blackboard_pointer_down_interaction,
    plan_blackboard_pointer_up_interaction,
};
use super::blackboard_layout::blackboard_panel_size;
use super::blackboard_policy::{
    BlackboardAction, BlackboardActionPlan, blackboard_action_a11y_label,
    blackboard_action_button_label, blackboard_actions_in_order, plan_blackboard_action,
};
use super::group_rename::NodeGraphOverlayState;
use super::rename_policy::open_symbol_rename_session;

type BlackboardActionHandler =
    Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, ActivateReason, BlackboardAction) + 'static>;

#[derive(Clone)]
pub(super) struct NodeGraphBlackboardActionIntegration {
    pub(super) binding: NodeGraphSurfaceBinding,
    pub(super) overlay_state: fret_runtime::Model<NodeGraphOverlayState>,
    pub(super) bounds: Rect,
}

#[derive(Clone)]
pub(super) struct NodeGraphBlackboardOverlayElementProps {
    pub(super) style: NodeGraphStyle,
    pub(super) symbols: BTreeMap<SymbolId, Symbol>,
    pub(super) on_action: Option<BlackboardActionHandler>,
    pub(super) action_integration: Option<NodeGraphBlackboardActionIntegration>,
    pub(super) focus_target: Option<GlobalElementId>,
}

pub(super) fn node_graph_blackboard_overlay_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    props: NodeGraphBlackboardOverlayElementProps,
) -> AnyElement {
    let panel = blackboard_panel_size(&props.style, props.symbols.len());
    let style = props.style;
    let symbols = props.symbols;
    let on_action = props.on_action;
    let action_integration = props.action_integration;
    let focus_target = props.focus_target;
    let interaction = cx.local_model(|| BlackboardInteractionState::new(None, None, None));
    let last_invoked_at_window = cx.local_model(|| Point::new(Px(0.0), Px(0.0)));
    let active = interaction
        .read_ref(cx.app, |state| state.keyboard_active)
        .ok()
        .flatten()
        .or_else(|| blackboard_actions_in_order(&symbols).first().copied())
        .unwrap_or(BlackboardAction::AddSymbol);
    let semantics_symbols = symbols.clone();

    cx.semantics_with_id(
        blackboard_semantics_props(active),
        move |cx, blackboard_root| {
            let key_interaction = interaction.clone();
            let key_symbols = semantics_symbols.clone();
            let key_on_action = on_action.clone();
            let key_action_integration = action_integration.clone();
            let key_invoked_at_window = last_invoked_at_window.clone();
            cx.key_on_key_down_for(
                blackboard_root,
                Arc::new(move |host, action_cx, down| {
                    if down.repeat || down.ime_composing {
                        return false;
                    }

                    let items = blackboard_actions_in_order(&key_symbols);
                    let mut plan = BlackboardKeyboardInteractionPlan::Ignore;
                    let updated = host
                        .models_mut()
                        .update(&key_interaction, |state| {
                            plan = plan_blackboard_keyboard_interaction(state, down.key, &items);
                        })
                        .is_ok();
                    if !updated {
                        return false;
                    }

                    match plan {
                        BlackboardKeyboardInteractionPlan::Ignore => false,
                        BlackboardKeyboardInteractionPlan::Select { finish_event, .. } => {
                            host.notify(action_cx);
                            finish_event
                        }
                        BlackboardKeyboardInteractionPlan::Activate {
                            action,
                            finish_event,
                        } => {
                            dispatch_blackboard_action(
                                host,
                                action_cx,
                                key_on_action.as_ref(),
                                key_action_integration.as_ref(),
                                &key_invoked_at_window,
                                action,
                                ActivateReason::Keyboard,
                            );
                            host.request_focus(blackboard_root);
                            host.notify(action_cx);
                            finish_event
                        }
                        BlackboardKeyboardInteractionPlan::FocusCanvas { finish_event } => {
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
                    if down.hit_is_pressable {
                        return false;
                    }
                    let Some(plan) = plan_blackboard_pointer_down_interaction(
                        &mut BlackboardInteractionState::new(None, None, None),
                        down.button,
                        true,
                        None,
                    ) else {
                        return false;
                    };

                    if plan.request_focus {
                        host.request_focus(blackboard_root);
                    }
                    if plan.repaint {
                        host.request_redraw(action_cx.window);
                    }
                    plan.stop_propagation
                }));

                vec![cx.container(panel_container(panel, &style), move |cx| {
                    vec![cx.column(panel_column(&style), move |cx| {
                        let mut children = vec![blackboard_header(
                            cx,
                            &style,
                            on_action.as_ref(),
                            action_integration.as_ref(),
                            blackboard_root,
                            interaction.clone(),
                            last_invoked_at_window.clone(),
                        )];
                        children.extend(symbols.iter().map(|(symbol_id, symbol)| {
                            cx.keyed(("blackboard.symbol.row", symbol_id.0), |cx| {
                                blackboard_symbol_row(
                                    cx,
                                    &style,
                                    *symbol_id,
                                    symbol,
                                    on_action.as_ref(),
                                    action_integration.as_ref(),
                                    blackboard_root,
                                    interaction.clone(),
                                    last_invoked_at_window.clone(),
                                )
                            })
                        }));
                        children
                    })]
                })]
            })]
        },
    )
}

fn blackboard_semantics_props(active: BlackboardAction) -> SemanticsProps {
    SemanticsProps {
        role: SemanticsRole::Panel,
        label: Some(Arc::from("Blackboard")),
        test_id: Some(Arc::from("node_graph.blackboard")),
        value: Some(Arc::from(blackboard_action_a11y_label(active))),
        focusable: true,
        ..Default::default()
    }
}

fn panel_container(size: Size, style: &NodeGraphStyle) -> ContainerProps {
    let mut props = ContainerProps::default();
    props.layout.size.width = Length::Px(size.width);
    props.layout.size.height = Length::Px(size.height);
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

fn panel_column(style: &NodeGraphStyle) -> ColumnProps {
    let mut props = ColumnProps::default();
    props.layout.size.width = Length::Fill;
    props.layout.size.height = Length::Fill;
    props.gap = SpacingLength::Px(Px(0.0));
    props.align = CrossAlign::Stretch;
    props.layout.size.min_height = Some(Length::Px(Px(style
        .paint
        .context_menu_item_height
        .max(20.0))));
    props
}

fn blackboard_header<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    style: &NodeGraphStyle,
    on_action: Option<&BlackboardActionHandler>,
    action_integration: Option<&NodeGraphBlackboardActionIntegration>,
    blackboard_root: GlobalElementId,
    interaction: fret_runtime::Model<BlackboardInteractionState>,
    last_invoked_at_window: fret_runtime::Model<Point>,
) -> AnyElement {
    cx.keyed("blackboard.header", |cx| {
        cx.row(row_props(style), move |cx| {
            vec![
                text_element(cx, style, "Blackboard", Some("node_graph.blackboard.title")),
                action_button(
                    cx,
                    style,
                    BlackboardAction::AddSymbol,
                    on_action,
                    action_integration,
                    blackboard_root,
                    interaction,
                    last_invoked_at_window,
                ),
            ]
        })
    })
}

fn blackboard_symbol_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    style: &NodeGraphStyle,
    symbol_id: SymbolId,
    symbol: &Symbol,
    on_action: Option<&BlackboardActionHandler>,
    action_integration: Option<&NodeGraphBlackboardActionIntegration>,
    blackboard_root: GlobalElementId,
    interaction: fret_runtime::Model<BlackboardInteractionState>,
    last_invoked_at_window: fret_runtime::Model<Point>,
) -> AnyElement {
    cx.row(row_props(style), move |cx| {
        vec![
            text_element(
                cx,
                style,
                symbol.name.clone(),
                Some(symbol_label_test_id(symbol_id)),
            ),
            action_button(
                cx,
                style,
                BlackboardAction::InsertRef { symbol: symbol_id },
                on_action,
                action_integration,
                blackboard_root,
                interaction.clone(),
                last_invoked_at_window.clone(),
            ),
            action_button(
                cx,
                style,
                BlackboardAction::Rename { symbol: symbol_id },
                on_action,
                action_integration,
                blackboard_root,
                interaction.clone(),
                last_invoked_at_window.clone(),
            ),
            action_button(
                cx,
                style,
                BlackboardAction::Delete { symbol: symbol_id },
                on_action,
                action_integration,
                blackboard_root,
                interaction,
                last_invoked_at_window,
            ),
        ]
    })
}

fn row_props(style: &NodeGraphStyle) -> RowProps {
    let mut props = RowProps::default();
    props.layout.size.width = Length::Fill;
    props.layout.size.height = Length::Px(Px(style.paint.context_menu_item_height.max(20.0)));
    props.gap = SpacingLength::Px(Px(6.0));
    props.justify = MainAlign::SpaceBetween;
    props.align = CrossAlign::Center;
    props
}

fn text_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    style: &NodeGraphStyle,
    text: impl Into<Arc<str>>,
    test_id: Option<impl Into<Arc<str>>>,
) -> AnyElement {
    let mut props = TextProps::new(text);
    props.layout.size.width = Length::Fill;
    props.style = Some(style.geometry.context_menu_text_style.clone());
    props.color = Some(style.paint.context_menu_text);
    props.wrap = TextWrap::None;
    props.overflow = TextOverflow::Clip;

    let element = cx.text_props(props);
    if let Some(test_id) = test_id {
        element.test_id(test_id)
    } else {
        element
    }
}

fn action_button<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    style: &NodeGraphStyle,
    action: BlackboardAction,
    on_action: Option<&BlackboardActionHandler>,
    action_integration: Option<&NodeGraphBlackboardActionIntegration>,
    blackboard_root: GlobalElementId,
    interaction: fret_runtime::Model<BlackboardInteractionState>,
    last_invoked_at_window: fret_runtime::Model<Point>,
) -> AnyElement {
    cx.keyed(blackboard_action_slot(action), |cx| {
        let mut props = PressableProps::default();
        let side = style.paint.context_menu_item_height.max(20.0);
        props.layout.size.width = Length::Px(Px(side));
        props.layout.size.height = Length::Px(Px(side));
        props.a11y.role = Some(SemanticsRole::Button);
        props.a11y.label = Some(Arc::from(blackboard_action_a11y_label(action)));
        props.a11y.test_id = Some(blackboard_action_test_id(action));

        cx.pressable(props, move |cx, _state| {
            let pointer_interaction = interaction.clone();
            cx.pressable_on_pointer_down(Arc::new(move |host, action_cx, down| {
                let _ = host.models_mut().update(&pointer_interaction, |state| {
                    let _ = plan_blackboard_pointer_down_interaction(
                        state,
                        down.button,
                        true,
                        Some(action),
                    );
                });
                host.request_focus(blackboard_root);
                host.notify(action_cx);
                PressablePointerDownResult::Continue
            }));

            let pointer_up_interaction = interaction.clone();
            let pointer_up_invoked_at_window = last_invoked_at_window.clone();
            cx.pressable_on_pointer_up(Arc::new(move |host, action_cx, up| {
                let _ = host.models_mut().update(&pointer_up_interaction, |state| {
                    let _ = plan_blackboard_pointer_up_interaction(
                        state,
                        up.button,
                        true,
                        Some(action),
                    );
                });
                let invoked_at = up.position_window.unwrap_or(up.position);
                let _ = host
                    .models_mut()
                    .update(&pointer_up_invoked_at_window, |point| {
                        *point = invoked_at;
                    });
                host.notify(action_cx);
                PressablePointerUpResult::Continue
            }));

            let action_integration = action_integration.cloned();
            let activate_invoked_at_window = last_invoked_at_window.clone();
            if let Some(on_action) = on_action.cloned() {
                let action_integration = action_integration.clone();
                let activate_invoked_at_window = activate_invoked_at_window.clone();
                cx.pressable_on_activate(Arc::new(move |host, action_cx, reason| {
                    dispatch_blackboard_action(
                        host,
                        action_cx,
                        Some(&on_action),
                        action_integration.as_ref(),
                        &activate_invoked_at_window,
                        action,
                        reason,
                    );
                    host.notify(action_cx);
                }));
            } else if let Some(action_integration) = action_integration {
                cx.pressable_on_activate(Arc::new(move |host, action_cx, reason| {
                    dispatch_blackboard_action(
                        host,
                        action_cx,
                        None,
                        Some(&action_integration),
                        &activate_invoked_at_window,
                        action,
                        reason,
                    );
                    host.notify(action_cx);
                }));
            }
            cx.pressable_on_activate_focus(Arc::new(move |host, _action_cx, _reason| {
                host.request_focus(blackboard_root);
            }));

            vec![text_element(
                cx,
                style,
                blackboard_action_button_label(action),
                None::<Arc<str>>,
            )]
        })
    })
}

fn dispatch_blackboard_action(
    host: &mut dyn UiActionHost,
    action_cx: ActionCx,
    on_action: Option<&BlackboardActionHandler>,
    action_integration: Option<&NodeGraphBlackboardActionIntegration>,
    last_invoked_at_window: &fret_runtime::Model<Point>,
    action: BlackboardAction,
    reason: ActivateReason,
) {
    if let Some(integration) = action_integration {
        dispatch_blackboard_integrated_action(host, integration, last_invoked_at_window, action);
    }
    if let Some(on_action) = on_action {
        on_action(host, action_cx, reason, action);
    }
}

fn dispatch_blackboard_integrated_action(
    host: &mut dyn UiActionHost,
    integration: &NodeGraphBlackboardActionIntegration,
    last_invoked_at_window: &fret_runtime::Model<Point>,
    action: BlackboardAction,
) -> bool {
    let store = integration.binding.store_model();
    let snapshot = host
        .models_mut()
        .read(&store, |store| {
            (store.graph().clone(), store.view_state().clone())
        })
        .ok();
    let Some((graph, view_state)) = snapshot else {
        return false;
    };
    let invoked_at_window = host
        .models_mut()
        .read(last_invoked_at_window, |point| *point)
        .ok()
        .unwrap_or_else(|| Point::new(Px(0.0), Px(0.0)));

    let Some(plan) = plan_blackboard_action(
        &graph,
        &view_state,
        integration.bounds,
        action,
        invoked_at_window,
    ) else {
        return false;
    };

    match plan {
        BlackboardActionPlan::Transaction(tx) => integration
            .binding
            .dispatch_transaction_action_host(host, &tx)
            .is_ok(),
        BlackboardActionPlan::OpenSymbolRename(rename) => host
            .models_mut()
            .update(&integration.overlay_state, |state| {
                open_symbol_rename_session(state, rename);
            })
            .is_ok(),
    }
}

fn blackboard_action_slot(action: BlackboardAction) -> (&'static str, Option<uuid::Uuid>) {
    match action {
        BlackboardAction::AddSymbol => ("blackboard.add_symbol", None),
        BlackboardAction::InsertRef { symbol } => ("blackboard.insert_ref", Some(symbol.0)),
        BlackboardAction::Rename { symbol } => ("blackboard.rename", Some(symbol.0)),
        BlackboardAction::Delete { symbol } => ("blackboard.delete", Some(symbol.0)),
    }
}

fn blackboard_action_test_id(action: BlackboardAction) -> Arc<str> {
    match action {
        BlackboardAction::AddSymbol => Arc::from("node_graph.blackboard.add_symbol"),
        BlackboardAction::InsertRef { symbol } => Arc::from(format!(
            "node_graph.blackboard.symbol.{}.insert_ref",
            symbol.0
        )),
        BlackboardAction::Rename { symbol } => {
            Arc::from(format!("node_graph.blackboard.symbol.{}.rename", symbol.0))
        }
        BlackboardAction::Delete { symbol } => {
            Arc::from(format!("node_graph.blackboard.symbol.{}.delete", symbol.0))
        }
    }
}

fn symbol_label_test_id(symbol: SymbolId) -> Arc<str> {
    Arc::from(format!("node_graph.blackboard.symbol.{}.label", symbol.0))
}

#[cfg(test)]
mod tests {
    use std::any::{Any, TypeId};
    use std::collections::{BTreeMap, HashMap};
    use std::sync::Arc;

    use fret_core::{
        AppWindowId, MaterialDescriptor, MaterialId, MaterialRegistrationError, Modifiers,
        MouseButton, PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle,
        Point, PointerId, PointerType, Px, Rect, SemanticsRole, SvgId, SvgService, TextConstraints,
        TextMetrics, TextService,
    };
    use fret_runtime::{
        ClipboardToken, CommandRegistry, CommandsHost, DragHost, DragKindId, DragSession, Effect,
        EffectSink, FrameId, GlobalsHost, ImageUploadToken, ModelHost, ModelId, ModelStore,
        ModelsHost, ShareSheetToken, TickId, TimeHost, TimerToken,
    };
    use fret_ui::GlobalElementId;
    use fret_ui::action::ActivateReason;
    use fret_ui::element::{ContainerProps, ElementKind, Length, StackProps};

    use super::BlackboardActionHandler;
    use crate::core::{
        CanvasPoint, CanvasSize, Graph, GraphId, Node, NodeId, NodeKindKey, SYMBOL_REF_NODE_KIND,
        Symbol, SymbolId,
    };
    use crate::io::{NodeGraphEditorConfig, NodeGraphViewState};
    use crate::ui::NodeGraphSurfaceBinding;
    use crate::ui::overlays::blackboard_declarative::{
        NodeGraphBlackboardActionIntegration, NodeGraphBlackboardOverlayElementProps,
        node_graph_blackboard_overlay_element,
    };
    use crate::ui::overlays::blackboard_layout::blackboard_panel_size;
    use crate::ui::overlays::blackboard_policy::BlackboardAction;
    use crate::ui::overlays::group_rename::NodeGraphOverlayState;
    use crate::ui::overlays::rename_policy::RenameOverlaySession;
    use crate::ui::overlays::rename_policy::active_rename_session;
    use crate::ui::style::NodeGraphStyle;

    #[derive(Default)]
    struct TestUiHost {
        globals: HashMap<TypeId, Box<dyn Any>>,
        models: ModelStore,
        commands: CommandRegistry,
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

        fn push_effect(&mut self, _effect: Effect) {}
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

    fn symbol(name: &str) -> Symbol {
        Symbol {
            name: name.to_string(),
            ty: None,
            default_value: None,
            meta: serde_json::Value::Null,
        }
    }

    fn render_blackboard(
        style: NodeGraphStyle,
        symbols: BTreeMap<SymbolId, Symbol>,
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
        node_graph_blackboard_overlay_element(
            &mut cx,
            NodeGraphBlackboardOverlayElementProps {
                style,
                symbols,
                on_action: None,
                action_integration: None,
                focus_target: None,
            },
        )
    }

    #[test]
    fn blackboard_declarative_composition_builds_panel_and_rows_without_retained_widget() {
        let mut style = NodeGraphStyle::default();
        style.paint.context_menu_width = 180.0;
        style.paint.context_menu_item_height = 24.0;
        style.paint.context_menu_padding = 6.0;
        let symbol_a = SymbolId::from_u128(2);
        let symbol_b = SymbolId::from_u128(1);
        let symbols = BTreeMap::from([(symbol_a, symbol("Beta")), (symbol_b, symbol("Alpha"))]);
        let expected = blackboard_panel_size(&style, symbols.len());

        let root = render_blackboard(style, symbols);

        let ElementKind::Semantics(root_props) = &root.kind else {
            panic!("blackboard root should be a declarative semantics wrapper");
        };
        assert_eq!(root_props.role, SemanticsRole::Panel);
        assert_eq!(root_props.label.as_deref(), Some("Blackboard"));
        assert_eq!(root_props.test_id.as_deref(), Some("node_graph.blackboard"));
        assert_eq!(root_props.value.as_deref(), Some("Add symbol"));
        assert!(root_props.focusable);
        assert_eq!(root.children.len(), 1);

        let pointer_region = &root.children[0];
        let ElementKind::PointerRegion(_) = &pointer_region.kind else {
            panic!("blackboard root should contain a pointer region");
        };

        let panel_el = &pointer_region.children[0];
        let ElementKind::Container(panel) = &panel_el.kind else {
            panic!("blackboard pointer region should contain the panel container");
        };
        assert_eq!(panel.layout.size.width, Length::Px(expected.width));
        assert_eq!(panel.layout.size.height, Length::Px(expected.height));

        let ElementKind::Column(_) = &panel_el.children[0].kind else {
            panic!("blackboard panel should contain a declarative column");
        };
        assert_eq!(panel_el.children[0].children.len(), 3);

        let header = &panel_el.children[0].children[0];
        assert_eq!(header.children.len(), 2);
        let ElementKind::Text(title) = &header.children[0].kind else {
            panic!("blackboard header should start with a title");
        };
        assert_eq!(title.text.as_ref(), "Blackboard");
        let ElementKind::Pressable(add) = &header.children[1].kind else {
            panic!("blackboard add action should be a pressable");
        };
        assert_eq!(add.a11y.label.as_deref(), Some("Add symbol"));
        assert_eq!(
            add.a11y.test_id.as_deref(),
            Some("node_graph.blackboard.add_symbol")
        );

        let first_row = &panel_el.children[0].children[1];
        let ElementKind::Text(first_label) = &first_row.children[0].kind else {
            panic!("blackboard row should start with a symbol label");
        };
        assert_eq!(first_label.text.as_ref(), "Alpha");
        assert_eq!(first_row.children.len(), 4);
    }

    #[test]
    fn blackboard_declarative_composition_stamps_symbol_action_a11y_and_test_ids() {
        let symbol_id = SymbolId::from_u128(42);
        let root = render_blackboard(
            NodeGraphStyle::default(),
            BTreeMap::from([(symbol_id, symbol("Value"))]),
        );
        let pointer_region = &root.children[0];
        let panel = &pointer_region.children[0];
        let column = &panel.children[0];
        let row = &column.children[1];

        let ElementKind::Pressable(insert) = &row.children[1].kind else {
            panic!("insert ref should be a pressable");
        };
        assert_eq!(
            insert.a11y.label.as_deref(),
            Some("Insert symbol reference")
        );
        assert_eq!(
            insert.a11y.test_id.as_deref(),
            Some("node_graph.blackboard.symbol.00000000-0000-0000-0000-00000000002a.insert_ref")
        );

        let ElementKind::Pressable(rename) = &row.children[2].kind else {
            panic!("rename should be a pressable");
        };
        assert_eq!(rename.a11y.label.as_deref(), Some("Rename symbol"));
        assert_eq!(
            rename.a11y.test_id.as_deref(),
            Some("node_graph.blackboard.symbol.00000000-0000-0000-0000-00000000002a.rename")
        );

        let ElementKind::Pressable(delete) = &row.children[3].kind else {
            panic!("delete should be a pressable");
        };
        assert_eq!(delete.a11y.role, Some(SemanticsRole::Button));
        assert_eq!(delete.a11y.label.as_deref(), Some("Delete symbol"));
        assert_eq!(
            delete.a11y.test_id.as_deref(),
            Some("node_graph.blackboard.symbol.00000000-0000-0000-0000-00000000002a.delete")
        );
    }

    #[test]
    fn blackboard_declarative_activation_routes_action_hook_without_retained_widget() {
        let mut host = TestUiHost::default();
        let mut ui = fret_ui::UiTree::<TestUiHost>::new();
        let mut services = FakeUiServices;
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(220.0), Px(160.0)),
        );
        let symbol_id = SymbolId::from_u128(7);
        let symbols = BTreeMap::from([(symbol_id, symbol("Value"))]);
        let activations = host
            .models_mut()
            .insert(Vec::<(BlackboardAction, ActivateReason)>::new());

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut host,
            &mut services,
            window,
            bounds,
            "blackboard-activation",
            |cx| {
                let activations = activations.clone();
                vec![node_graph_blackboard_overlay_element(
                    cx,
                    NodeGraphBlackboardOverlayElementProps {
                        style: NodeGraphStyle::default(),
                        symbols,
                        on_action: Some(Arc::new(move |host, _cx, reason, action| {
                            let _ = host.models_mut().update(&activations, |items| {
                                items.push((action, reason));
                            });
                        })),
                        action_integration: None,
                        focus_target: None,
                    },
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut host, &mut services, bounds, 1.0);

        let blackboard = ui.children(root)[0];
        let pointer_region = ui.children(blackboard)[0];
        let panel = ui.children(pointer_region)[0];
        let column = ui.children(panel)[0];
        let header = ui.children(column)[0];
        let add = ui.children(header)[1];
        click_node(&mut ui, &mut host, &mut services, add);

        let row = ui.children(column)[1];
        let rename = ui.children(row)[2];
        click_node(&mut ui, &mut host, &mut services, rename);

        assert_eq!(
            host.models().get_cloned(&activations),
            Some(vec![
                (BlackboardAction::AddSymbol, ActivateReason::Pointer),
                (
                    BlackboardAction::Rename { symbol: symbol_id },
                    ActivateReason::Pointer
                ),
            ])
        );
    }

    #[test]
    fn blackboard_declarative_button_pointer_up_completes_capture_focus_and_activation() {
        let mut host = TestUiHost::default();
        let mut ui = fret_ui::UiTree::<TestUiHost>::new();
        let mut services = FakeUiServices;
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(220.0), Px(160.0)),
        );
        let activations = host
            .models_mut()
            .insert(Vec::<(BlackboardAction, ActivateReason)>::new());
        let root = render_blackboard_root(
            &mut ui,
            &mut host,
            &mut services,
            window,
            bounds,
            "blackboard-pointer-up-completion",
            NodeGraphStyle::default(),
            BTreeMap::new(),
            activations.clone(),
            None,
        );

        let add = blackboard_add_button_node(&ui, root);
        let position = pointer_position_inside_node(&ui, add);
        dispatch_pointer_down_at(&mut ui, &mut host, &mut services, position);
        assert_eq!(
            ui.captured(),
            Some(add),
            "blackboard pressable should capture pointer on button down"
        );
        assert_eq!(host.models().get_cloned(&activations), Some(Vec::new()));

        dispatch_pointer_up_at(&mut ui, &mut host, &mut services, position);
        assert_eq!(ui.captured(), None);
        assert_eq!(
            ui.focus(),
            Some(ui.children(root)[0]),
            "blackboard activation should restore focus to the focusable panel root"
        );
        assert_eq!(
            host.models().get_cloned(&activations),
            Some(vec![(BlackboardAction::AddSymbol, ActivateReason::Pointer)])
        );
    }

    #[test]
    fn blackboard_declarative_root_keyboard_navigation_activation_and_escape_match_retained_host() {
        let mut host = TestUiHost::default();
        let mut ui = fret_ui::UiTree::<TestUiHost>::new();
        let mut services = FakeUiServices;
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(240.0), Px(180.0)),
        );
        let symbol_id = SymbolId::from_u128(9);
        let symbols = BTreeMap::from([(symbol_id, symbol("Value"))]);
        let activations = host
            .models_mut()
            .insert(Vec::<(BlackboardAction, ActivateReason)>::new());
        let root_name = "blackboard-root-keyboard";
        let root = rerender_blackboard_with_surface(
            &mut ui,
            &mut host,
            &mut services,
            window,
            bounds,
            root_name,
            symbols.clone(),
            activations.clone(),
        );

        let stack = ui.children(root)[0];
        let surface = ui.children(stack)[0];
        let blackboard = ui.children(stack)[1];
        ui.set_focus(Some(blackboard));

        dispatch_key_down(
            &mut ui,
            &mut host,
            &mut services,
            fret_core::KeyCode::ArrowDown,
        );
        rerender_blackboard_with_surface(
            &mut ui,
            &mut host,
            &mut services,
            window,
            bounds,
            root_name,
            symbols.clone(),
            activations.clone(),
        );
        assert_eq!(
            blackboard_semantics_value(&mut ui, &mut host, &mut services, bounds),
            Some("Insert symbol reference".to_string())
        );

        dispatch_key_down(&mut ui, &mut host, &mut services, fret_core::KeyCode::Enter);
        assert_eq!(
            host.models().get_cloned(&activations),
            Some(vec![(
                BlackboardAction::InsertRef { symbol: symbol_id },
                ActivateReason::Keyboard
            )])
        );
        assert_eq!(
            ui.focus(),
            Some(blackboard),
            "keyboard activation should keep focus on the blackboard root for follow-up keys"
        );

        dispatch_key_down(
            &mut ui,
            &mut host,
            &mut services,
            fret_core::KeyCode::Escape,
        );
        assert_eq!(
            ui.focus(),
            Some(surface),
            "Escape from blackboard root should return focus to the node graph surface target"
        );
        assert_eq!(
            host.models().get_cloned(&activations),
            Some(vec![(
                BlackboardAction::InsertRef { symbol: symbol_id },
                ActivateReason::Keyboard
            )]),
            "Escape should not dispatch blackboard actions"
        );
    }

    #[test]
    fn blackboard_declarative_pointer_events_fall_through_outside_panel_and_block_inside_panel() {
        let mut host = TestUiHost::default();
        let mut ui = fret_ui::UiTree::<TestUiHost>::new();
        let mut services = FakeUiServices;
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(320.0), Px(220.0)),
        );
        let underlay_downs = host.models.insert(0_u32);
        let activations = host
            .models_mut()
            .insert(Vec::<(BlackboardAction, ActivateReason)>::new());

        let root = render_blackboard_with_recording_surface(
            &mut ui,
            &mut host,
            &mut services,
            window,
            bounds,
            "blackboard-pointer-surface-integration",
            NodeGraphStyle::default(),
            BTreeMap::new(),
            activations,
            underlay_downs.clone(),
        );
        let stack = ui.children(root)[0];
        let surface = ui.children(stack)[0];
        let blackboard = ui.children(stack)[1];
        let pointer_region = ui.children(blackboard)[0];
        let panel = ui.children(pointer_region)[0];
        let surface_bounds = ui
            .debug_node_bounds(surface)
            .expect("surface should be laid out");
        let panel_bounds = ui
            .debug_node_bounds(panel)
            .expect("blackboard panel should be laid out");

        let outside_panel = Point::new(
            Px((panel_bounds.origin.x.0 + panel_bounds.size.width.0 + 8.0)
                .min(surface_bounds.origin.x.0 + surface_bounds.size.width.0 - 2.0)),
            Px(surface_bounds.origin.y.0 + surface_bounds.size.height.0 - 2.0),
        );
        dispatch_pointer_down_at(&mut ui, &mut host, &mut services, outside_panel);
        assert_eq!(
            underlay_downs
                .read_ref(&host, |count| *count)
                .expect("underlay counter"),
            1,
            "pointer-down outside the blackboard panel should fall through to the surface"
        );

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
            1,
            "blank panel pointer-down should block the underlying surface"
        );
        assert_eq!(
            ui.focus(),
            Some(blackboard),
            "blank panel pointer-down should focus the blackboard root"
        );
    }

    #[test]
    fn blackboard_declarative_add_symbol_commits_through_surface_binding() {
        let mut host = TestUiHost::default();
        let mut ui = fret_ui::UiTree::<TestUiHost>::new();
        let mut services = FakeUiServices;
        let window = AppWindowId::default();
        ui.set_window(window);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(320.0), Px(220.0)),
        );
        let (binding, overlays) = blackboard_binding_and_overlays(
            &mut host,
            Graph::new(GraphId::new()),
            NodeGraphViewState::default(),
        );

        let root = render_integrated_blackboard(
            &mut ui,
            &mut host,
            &mut services,
            window,
            bounds,
            "blackboard-integrated-add",
            binding.clone(),
            overlays,
            None,
            None,
        );
        let add = blackboard_add_button_node(&ui, root);
        click_node(&mut ui, &mut host, &mut services, add);

        let graph = binding
            .store_model()
            .read_ref(&host, |store| store.graph().clone())
            .expect("store graph");
        assert_eq!(graph.symbols.len(), 1);
        let symbol = graph.symbols.values().next().expect("added symbol");
        assert_eq!(symbol.name, "Symbol");
    }

    #[test]
    fn blackboard_declarative_insert_ref_uses_binding_view_state_and_surface_bounds() {
        let mut host = TestUiHost::default();
        let mut ui = fret_ui::UiTree::<TestUiHost>::new();
        let mut services = FakeUiServices;
        let window = AppWindowId::default();
        ui.set_window(window);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(320.0), Px(220.0)),
        );
        let symbol_id = SymbolId::from_u128(0x11);
        let mut graph = Graph::new(GraphId::new());
        graph.symbols.insert(symbol_id, symbol("Value"));
        let view_state = NodeGraphViewState {
            pan: CanvasPoint { x: 10.0, y: -4.0 },
            zoom: 2.0,
            ..Default::default()
        };
        let (binding, overlays) = blackboard_binding_and_overlays(&mut host, graph, view_state);

        let root = render_integrated_blackboard(
            &mut ui,
            &mut host,
            &mut services,
            window,
            bounds,
            "blackboard-integrated-insert-ref",
            binding.clone(),
            overlays,
            Some(symbol_id),
            None,
        );
        let insert = blackboard_symbol_action_button_node(&ui, root, 1);
        click_node(&mut ui, &mut host, &mut services, insert);

        let graph = binding
            .store_model()
            .read_ref(&host, |store| store.graph().clone())
            .expect("store graph");
        let node = graph.nodes.values().next().expect("inserted symbol ref");
        assert_eq!(node.kind, NodeKindKey::new(SYMBOL_REF_NODE_KIND));
        assert_eq!(
            node.data.get("symbol_id"),
            Some(&serde_json::json!(symbol_id))
        );
        assert!((node.pos.x - 70.0).abs() <= 1.0e-3);
        assert!((node.pos.y - 59.0).abs() <= 1.0e-3);
    }

    #[test]
    fn blackboard_declarative_delete_symbol_removes_refs_before_symbol_through_binding() {
        let mut host = TestUiHost::default();
        let mut ui = fret_ui::UiTree::<TestUiHost>::new();
        let mut services = FakeUiServices;
        let window = AppWindowId::default();
        ui.set_window(window);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(320.0), Px(220.0)),
        );
        let symbol_id = SymbolId::from_u128(0x22);
        let ref_node_id = NodeId::from_u128(0x33);
        let mut graph = Graph::new(GraphId::new());
        graph.symbols.insert(symbol_id, symbol("Value"));
        graph.nodes.insert(
            ref_node_id,
            Node {
                kind: NodeKindKey::new(SYMBOL_REF_NODE_KIND),
                kind_version: 1,
                pos: CanvasPoint { x: 1.0, y: 2.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                extent: None,
                expand_parent: None,
                size: Some(CanvasSize {
                    width: 140.0,
                    height: 40.0,
                }),
                hidden: false,
                collapsed: false,
                ports: Vec::new(),
                data: crate::core::symbol_ref_node_data(symbol_id),
            },
        );
        let (binding, overlays) =
            blackboard_binding_and_overlays(&mut host, graph, NodeGraphViewState::default());

        let root = render_integrated_blackboard(
            &mut ui,
            &mut host,
            &mut services,
            window,
            bounds,
            "blackboard-integrated-delete",
            binding.clone(),
            overlays,
            Some(symbol_id),
            None,
        );
        let delete = blackboard_symbol_action_button_node(&ui, root, 3);
        click_node(&mut ui, &mut host, &mut services, delete);

        let graph = binding
            .store_model()
            .read_ref(&host, |store| {
                assert_eq!(store.history().undo_len(), 1);
                store.graph().clone()
            })
            .expect("store state");
        assert!(!graph.symbols.contains_key(&symbol_id));
        assert!(!graph.nodes.contains_key(&ref_node_id));
    }

    #[test]
    fn blackboard_declarative_rename_opens_overlay_state_without_transaction() {
        let mut host = TestUiHost::default();
        let mut ui = fret_ui::UiTree::<TestUiHost>::new();
        let mut services = FakeUiServices;
        let window = AppWindowId::default();
        ui.set_window(window);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(320.0), Px(220.0)),
        );
        let symbol_id = SymbolId::from_u128(0x44);
        let mut graph = Graph::new(GraphId::new());
        graph.symbols.insert(symbol_id, symbol("Value"));
        let (binding, overlays) =
            blackboard_binding_and_overlays(&mut host, graph, NodeGraphViewState::default());

        let root = render_integrated_blackboard(
            &mut ui,
            &mut host,
            &mut services,
            window,
            bounds,
            "blackboard-integrated-rename",
            binding.clone(),
            overlays.clone(),
            Some(symbol_id),
            None,
        );
        let rename = blackboard_symbol_action_button_node(&ui, root, 2);
        let rename_position = pointer_position_inside_node(&ui, rename);
        click_node(&mut ui, &mut host, &mut services, rename);

        let opened = overlays
            .read_ref(&host, active_rename_session)
            .expect("overlay state")
            .expect("symbol rename session");
        let RenameOverlaySession::Symbol(rename) = opened else {
            panic!("expected symbol rename session");
        };
        assert_eq!(rename.symbol, symbol_id);
        assert_eq!(rename.invoked_at_window, rename_position);
        let undo_len = binding
            .store_model()
            .read_ref(&host, |store| store.history().undo_len())
            .expect("undo len");
        assert_eq!(undo_len, 0, "rename handoff should not commit a graph tx");
    }

    fn blackboard_binding_and_overlays(
        host: &mut TestUiHost,
        graph: Graph,
        view_state: NodeGraphViewState,
    ) -> (
        NodeGraphSurfaceBinding,
        fret_runtime::Model<NodeGraphOverlayState>,
    ) {
        let binding = NodeGraphSurfaceBinding::new(
            host.models_mut(),
            graph,
            view_state,
            NodeGraphEditorConfig::default(),
        );
        let overlays = host.models_mut().insert(NodeGraphOverlayState::default());
        (binding, overlays)
    }

    #[allow(clippy::too_many_arguments)]
    fn render_integrated_blackboard(
        ui: &mut fret_ui::UiTree<TestUiHost>,
        host: &mut TestUiHost,
        services: &mut FakeUiServices,
        window: AppWindowId,
        bounds: Rect,
        root_name: &str,
        binding: NodeGraphSurfaceBinding,
        overlays: fret_runtime::Model<NodeGraphOverlayState>,
        symbol_id: Option<SymbolId>,
        on_action: Option<BlackboardActionHandler>,
    ) -> fret_core::NodeId {
        let symbols = binding
            .store_model()
            .read_ref(host, |store| store.graph().symbols.clone())
            .expect("store symbols");
        let root = fret_ui::declarative::render_root(
            ui,
            host,
            services,
            window,
            bounds,
            root_name,
            |cx| {
                vec![node_graph_blackboard_overlay_element(
                    cx,
                    NodeGraphBlackboardOverlayElementProps {
                        style: NodeGraphStyle::default(),
                        symbols,
                        on_action,
                        action_integration: Some(NodeGraphBlackboardActionIntegration {
                            binding,
                            overlay_state: overlays,
                            bounds,
                        }),
                        focus_target: None,
                    },
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(host, services, bounds, 1.0);

        let _ = symbol_id;

        root
    }

    fn blackboard_symbol_action_button_node(
        ui: &fret_ui::UiTree<TestUiHost>,
        root: fret_core::NodeId,
        action_child_index: usize,
    ) -> fret_core::NodeId {
        let blackboard = ui.children(root)[0];
        let pointer_region = ui.children(blackboard)[0];
        let panel = ui.children(pointer_region)[0];
        let column = ui.children(panel)[0];
        let row = ui.children(column)[1];
        ui.children(row)[action_child_index]
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

    #[allow(clippy::too_many_arguments)]
    fn render_blackboard_root(
        ui: &mut fret_ui::UiTree<TestUiHost>,
        host: &mut TestUiHost,
        services: &mut FakeUiServices,
        window: AppWindowId,
        bounds: Rect,
        root_name: &str,
        style: NodeGraphStyle,
        symbols: BTreeMap<SymbolId, Symbol>,
        activations: fret_runtime::Model<Vec<(BlackboardAction, ActivateReason)>>,
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
                let activations = activations.clone();
                vec![node_graph_blackboard_overlay_element(
                    cx,
                    NodeGraphBlackboardOverlayElementProps {
                        style,
                        symbols,
                        on_action: Some(Arc::new(move |host, _cx, reason, action| {
                            let _ = host.models_mut().update(&activations, |items| {
                                items.push((action, reason));
                            });
                        })),
                        action_integration: None,
                        focus_target,
                    },
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(host, services, bounds, 1.0);
        root
    }

    #[allow(clippy::too_many_arguments)]
    fn rerender_blackboard_with_surface(
        ui: &mut fret_ui::UiTree<TestUiHost>,
        host: &mut TestUiHost,
        services: &mut FakeUiServices,
        window: AppWindowId,
        bounds: Rect,
        root_name: &str,
        symbols: BTreeMap<SymbolId, Symbol>,
        activations: fret_runtime::Model<Vec<(BlackboardAction, ActivateReason)>>,
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
                    let activations = activations.clone();
                    let blackboard = node_graph_blackboard_overlay_element(
                        cx,
                        NodeGraphBlackboardOverlayElementProps {
                            style: NodeGraphStyle::default(),
                            symbols,
                            on_action: Some(Arc::new(move |host, _cx, reason, action| {
                                let _ = host.models_mut().update(&activations, |items| {
                                    items.push((action, reason));
                                });
                            })),
                            action_integration: None,
                            focus_target: Some(surface_target),
                        },
                    );
                    vec![surface, blackboard]
                })]
            },
        );
        ui.set_root(root);
        ui.layout_all(host, services, bounds, 1.0);
        root
    }

    #[allow(clippy::too_many_arguments)]
    fn render_blackboard_with_recording_surface(
        ui: &mut fret_ui::UiTree<TestUiHost>,
        host: &mut TestUiHost,
        services: &mut FakeUiServices,
        window: AppWindowId,
        bounds: Rect,
        root_name: &str,
        style: NodeGraphStyle,
        symbols: BTreeMap<SymbolId, Symbol>,
        activations: fret_runtime::Model<Vec<(BlackboardAction, ActivateReason)>>,
        underlay_downs: fret_runtime::Model<u32>,
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
                    let activations = activations.clone();

                    let blackboard = node_graph_blackboard_overlay_element(
                        cx,
                        NodeGraphBlackboardOverlayElementProps {
                            style,
                            symbols,
                            on_action: Some(Arc::new(move |host, _cx, reason, action| {
                                let _ = host.models_mut().update(&activations, |items| {
                                    items.push((action, reason));
                                });
                            })),
                            action_integration: None,
                            focus_target: Some(surface_target),
                        },
                    );
                    vec![surface, blackboard]
                })]
            },
        );
        ui.set_root(root);
        ui.layout_all(host, services, bounds, 1.0);
        root
    }

    fn blackboard_add_button_node(
        ui: &fret_ui::UiTree<TestUiHost>,
        root: fret_core::NodeId,
    ) -> fret_core::NodeId {
        let blackboard = ui.children(root)[0];
        let pointer_region = ui.children(blackboard)[0];
        let panel = ui.children(pointer_region)[0];
        let column = ui.children(panel)[0];
        let header = ui.children(column)[0];
        ui.children(header)[1]
    }

    fn blackboard_semantics_value(
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
            .find(|node| node.test_id.as_deref() == Some("node_graph.blackboard"))
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
