use std::sync::Arc;

use fret_core::{Corners, Edges, Px, Rect, SemanticsRole, TextStyle};
use fret_runtime::{CommandId, Model};
use fret_ui::element::{
    AnyElement, ContainerProps, InsetEdge, Length, PositionStyle, SemanticsDecoration,
    SpacingEdges, SpacingLength, TextInputProps,
};
use fret_ui::{ElementContext, TextInputStyle, UiHost};

use crate::ui::NodeGraphStyle;

use super::group_rename::NodeGraphOverlayState;
use super::rename_command::{rename_cancel_command, rename_submit_command};
use super::rename_host_layout::{RenameHostLayoutPlan, plan_rename_host_layout};
use super::rename_policy::{RenameOverlaySession, RenameOverlaySessionKey, active_rename_session};

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

pub(super) fn node_graph_rename_overlay_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    props: NodeGraphRenameOverlayElementProps,
) -> Option<AnyElement> {
    let session = active_rename_session(&props.overlay_state)?;
    let plan = plan_rename_host_layout(
        &props.style,
        props.bounds,
        Some(&session),
        props.text_input_node,
        props.focus,
        props.last_opened_session.map(Into::into),
    );
    let RenameHostLayoutPlan::Active { rect, .. } = plan else {
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

    use fret_core::{AppWindowId, Point, PointerId, Px, Rect, SemanticsRole, Size};
    use fret_runtime::{
        ClipboardToken, CommandRegistry, CommandsHost, DragHost, DragKindId, DragSession, Effect,
        EffectSink, FrameId, GlobalsHost, ImageUploadToken, ModelHost, ModelId, ModelStore,
        ModelsHost, ShareSheetToken, TickId, TimeHost, TimerToken,
    };
    use fret_ui::element::{ElementKind, InsetEdge, Length, PositionStyle};

    use crate::core::{GroupId, SymbolId};
    use crate::ui::NodeGraphStyle;
    use crate::ui::overlays::group_rename::{
        GroupRenameOverlay, NodeGraphOverlayState, SymbolRenameOverlay,
    };
    use crate::ui::overlays::rename_command::{
        RenameTextCommand, parse_rename_text_command, rename_cancel_command, rename_submit_command,
    };
    use crate::ui::overlays::rename_declarative::{
        NodeGraphRenameOverlayElementProps, node_graph_rename_overlay_element,
    };
    use crate::ui::overlays::rename_host_layout::{RenameHostLayoutPlan, plan_rename_host_layout};
    use crate::ui::overlays::rename_policy::{RenameOverlaySessionKey, active_rename_session};

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
