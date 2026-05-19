use std::collections::BTreeMap;
use std::sync::Arc;

use fret_core::{Corners, Edges, Px, SemanticsRole, Size, TextOverflow, TextWrap};
use fret_ui::action::{ActionCx, ActivateReason, UiActionHost};
use fret_ui::element::{
    AnyElement, ColumnProps, ContainerProps, CrossAlign, Length, MainAlign, PressableProps,
    RowProps, SemanticsDecoration, SpacingEdges, SpacingLength, TextProps,
};
use fret_ui::{ElementContext, UiHost};

use crate::core::{Symbol, SymbolId};
use crate::ui::NodeGraphStyle;

use super::blackboard_layout::blackboard_panel_size;
use super::blackboard_policy::{
    BlackboardAction, blackboard_action_a11y_label, blackboard_action_button_label,
};

type BlackboardActionHandler =
    Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, ActivateReason, BlackboardAction) + 'static>;

#[derive(Clone)]
pub(super) struct NodeGraphBlackboardOverlayElementProps {
    pub(super) style: NodeGraphStyle,
    pub(super) symbols: BTreeMap<SymbolId, Symbol>,
    pub(super) on_action: Option<BlackboardActionHandler>,
}

pub(super) fn node_graph_blackboard_overlay_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    props: NodeGraphBlackboardOverlayElementProps,
) -> AnyElement {
    let panel = blackboard_panel_size(&props.style, props.symbols.len());
    let style = props.style;
    let symbols = props.symbols;
    let on_action = props.on_action;

    cx.container(panel_container(panel, &style), move |cx| {
        vec![cx.column(panel_column(&style), move |cx| {
            let mut children = vec![blackboard_header(cx, &style, on_action.as_ref())];
            children.extend(symbols.iter().map(|(symbol_id, symbol)| {
                cx.keyed(("blackboard.symbol.row", symbol_id.0), |cx| {
                    blackboard_symbol_row(cx, &style, *symbol_id, symbol, on_action.as_ref())
                })
            }));
            children
        })]
    })
    .attach_semantics(
        SemanticsDecoration::default()
            .role(SemanticsRole::Panel)
            .label("Blackboard")
            .test_id("node_graph.blackboard"),
    )
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
) -> AnyElement {
    cx.keyed("blackboard.header", |cx| {
        cx.row(row_props(style), move |cx| {
            vec![
                text_element(cx, style, "Blackboard", Some("node_graph.blackboard.title")),
                action_button(cx, style, BlackboardAction::AddSymbol, on_action),
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
            ),
            action_button(
                cx,
                style,
                BlackboardAction::Rename { symbol: symbol_id },
                on_action,
            ),
            action_button(
                cx,
                style,
                BlackboardAction::Delete { symbol: symbol_id },
                on_action,
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
            if let Some(on_action) = on_action.cloned() {
                cx.pressable_on_activate(Arc::new(move |host, action_cx, reason| {
                    on_action(host, action_cx, reason, action);
                }));
            }

            vec![text_element(
                cx,
                style,
                blackboard_action_button_label(action),
                None::<Arc<str>>,
            )]
        })
    })
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
    use fret_ui::action::ActivateReason;
    use fret_ui::element::{ElementKind, Length};

    use crate::core::{Symbol, SymbolId};
    use crate::ui::overlays::blackboard_declarative::{
        NodeGraphBlackboardOverlayElementProps, node_graph_blackboard_overlay_element,
    };
    use crate::ui::overlays::blackboard_layout::blackboard_panel_size;
    use crate::ui::overlays::blackboard_policy::BlackboardAction;
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

        let ElementKind::Container(panel) = &root.kind else {
            panic!("blackboard root should be a declarative container");
        };
        assert_eq!(panel.layout.size.width, Length::Px(expected.width));
        assert_eq!(panel.layout.size.height, Length::Px(expected.height));
        let semantics = root.semantics_decoration.as_ref().expect("root semantics");
        assert_eq!(semantics.role, Some(SemanticsRole::Panel));
        assert_eq!(semantics.label.as_deref(), Some("Blackboard"));
        assert_eq!(semantics.test_id.as_deref(), Some("node_graph.blackboard"));

        let ElementKind::Column(_) = &root.children[0].kind else {
            panic!("blackboard root should contain a declarative column");
        };
        assert_eq!(root.children[0].children.len(), 3);

        let header = &root.children[0].children[0];
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

        let first_row = &root.children[0].children[1];
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
        let row = &root.children[0].children[1];

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
                    },
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut host, &mut services, bounds, 1.0);

        let panel = ui.children(root)[0];
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
