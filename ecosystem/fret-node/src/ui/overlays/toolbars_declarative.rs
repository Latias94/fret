use std::sync::Arc;

use fret_core::{Point, Px, Rect, SemanticsRole, Size};
use fret_ui::element::{
    AnyElement, ContainerProps, InsetEdge, Length, PositionStyle, SemanticsDecoration,
    SpacingEdges, SpacingLength,
};
use fret_ui::{ElementContext, UiHost};

use crate::ui::screen_space_placement::{rect_adjacent_to_rect, rect_anchored_at_point};

use super::toolbar_policy::{
    NodeGraphToolbarAlign, NodeGraphToolbarPosition, NodeGraphToolbarVisibility,
    toolbar_align_axis, toolbar_position_to_adjacent, toolbar_visible,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct NodeGraphNodeToolbarTarget {
    pub(super) rect: Rect,
    pub(super) selected: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct NodeGraphEdgeToolbarTarget {
    pub(super) center: Point,
    pub(super) selected: bool,
}

#[derive(Debug, Clone)]
pub(super) struct NodeGraphNodeToolbarElementProps {
    pub(super) bounds: Rect,
    pub(super) target: Option<NodeGraphNodeToolbarTarget>,
    pub(super) visibility: NodeGraphToolbarVisibility,
    pub(super) position: NodeGraphToolbarPosition,
    pub(super) align: NodeGraphToolbarAlign,
    pub(super) size: Size,
    pub(super) gap_px: f32,
    pub(super) offset: Point,
    pub(super) label: Arc<str>,
    pub(super) test_id: Arc<str>,
}

#[derive(Debug, Clone)]
pub(super) struct NodeGraphEdgeToolbarElementProps {
    pub(super) bounds: Rect,
    pub(super) target: Option<NodeGraphEdgeToolbarTarget>,
    pub(super) visibility: NodeGraphToolbarVisibility,
    pub(super) align_x: NodeGraphToolbarAlign,
    pub(super) align_y: NodeGraphToolbarAlign,
    pub(super) size: Size,
    pub(super) offset: Point,
    pub(super) label: Arc<str>,
    pub(super) test_id: Arc<str>,
}

pub(super) fn node_graph_node_toolbar_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    props: NodeGraphNodeToolbarElementProps,
    children: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> Option<AnyElement> {
    let rect = plan_node_toolbar_rect(
        props.bounds,
        props.target,
        props.visibility,
        props.size,
        props.position,
        props.align,
        props.gap_px,
        props.offset,
    )?;
    Some(toolbar_element(
        cx,
        props.bounds,
        rect,
        props.label,
        props.test_id,
        children,
    ))
}

pub(super) fn node_graph_edge_toolbar_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    props: NodeGraphEdgeToolbarElementProps,
    children: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> Option<AnyElement> {
    let rect = plan_edge_toolbar_rect(
        props.bounds,
        props.target,
        props.visibility,
        props.size,
        props.align_x,
        props.align_y,
        props.offset,
    )?;
    Some(toolbar_element(
        cx,
        props.bounds,
        rect,
        props.label,
        props.test_id,
        children,
    ))
}

fn toolbar_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    bounds: Rect,
    rect: Rect,
    label: Arc<str>,
    test_id: Arc<str>,
    children: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    cx.container(toolbar_container(bounds, rect), children)
        .attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Toolbar)
                .label(label)
                .test_id(test_id),
        )
}

pub(super) fn plan_node_toolbar_rect(
    bounds: Rect,
    target: Option<NodeGraphNodeToolbarTarget>,
    visibility: NodeGraphToolbarVisibility,
    size: Size,
    position: NodeGraphToolbarPosition,
    align: NodeGraphToolbarAlign,
    gap_px: f32,
    offset: Point,
) -> Option<Rect> {
    let target = target?;
    if !toolbar_visible(visibility, target.selected) || toolbar_size_is_empty(size) {
        return None;
    }

    Some(rect_adjacent_to_rect(
        bounds,
        target.rect,
        size,
        toolbar_position_to_adjacent(position),
        toolbar_align_axis(align),
        gap_px,
        offset,
    ))
}

pub(super) fn plan_edge_toolbar_rect(
    bounds: Rect,
    target: Option<NodeGraphEdgeToolbarTarget>,
    visibility: NodeGraphToolbarVisibility,
    size: Size,
    align_x: NodeGraphToolbarAlign,
    align_y: NodeGraphToolbarAlign,
    offset: Point,
) -> Option<Rect> {
    let target = target?;
    if !toolbar_visible(visibility, target.selected) || toolbar_size_is_empty(size) {
        return None;
    }

    Some(rect_anchored_at_point(
        bounds,
        target.center,
        size,
        toolbar_align_axis(align_x),
        toolbar_align_axis(align_y),
        offset,
    ))
}

fn toolbar_size_is_empty(size: Size) -> bool {
    size.width.0 <= 0.0 && size.height.0 <= 0.0
}

fn toolbar_container(bounds: Rect, rect: Rect) -> ContainerProps {
    let mut props = ContainerProps::default();
    props.layout.position = PositionStyle::Absolute;
    props.layout.inset.left = InsetEdge::Px(Px(rect.origin.x.0 - bounds.origin.x.0));
    props.layout.inset.top = InsetEdge::Px(Px(rect.origin.y.0 - bounds.origin.y.0));
    props.layout.size.width = Length::Px(rect.size.width);
    props.layout.size.height = Length::Px(rect.size.height);
    props.padding = SpacingEdges::all(SpacingLength::Px(Px(0.0)));
    props.snap_to_device_pixels = true;
    props
}

#[cfg(test)]
mod tests {
    use std::any::{Any, TypeId};
    use std::collections::HashMap;
    use std::sync::Arc;

    use fret_core::{AppWindowId, Point, PointerId, Px, Rect, SemanticsRole, Size};
    use fret_runtime::{
        ClipboardToken, CommandRegistry, CommandsHost, DragHost, DragKindId, DragSession, Effect,
        EffectSink, FrameId, GlobalsHost, ImageUploadToken, ModelHost, ModelId, ModelStore,
        ModelsHost, ShareSheetToken, TickId, TimeHost, TimerToken,
    };
    use fret_ui::element::{ElementKind, InsetEdge, Length, PositionStyle};

    use crate::ui::overlays::toolbar_policy::{
        NodeGraphToolbarAlign, NodeGraphToolbarPosition, NodeGraphToolbarVisibility,
    };
    use crate::ui::overlays::toolbars_declarative::{
        NodeGraphEdgeToolbarElementProps, NodeGraphEdgeToolbarTarget,
        NodeGraphNodeToolbarElementProps, NodeGraphNodeToolbarTarget,
        node_graph_edge_toolbar_element, node_graph_node_toolbar_element, plan_edge_toolbar_rect,
        plan_node_toolbar_rect,
    };

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
            Point::new(Px(10.0), Px(20.0)),
            Size::new(Px(200.0), Px(100.0)),
        )
    }

    fn render_node_toolbar() -> fret_ui::element::AnyElement {
        let mut host = TestUiHost::default();
        let mut runtime = fret_ui::ElementRuntime::new();
        let window = AppWindowId::default();
        let root_bounds = bounds();
        let mut cx = fret_ui::ElementContext::new_for_root_name(
            &mut host,
            &mut runtime,
            window,
            root_bounds,
            "root",
        );
        node_graph_node_toolbar_element(
            &mut cx,
            NodeGraphNodeToolbarElementProps {
                bounds: root_bounds,
                target: Some(NodeGraphNodeToolbarTarget {
                    rect: Rect::new(
                        Point::new(Px(60.0), Px(60.0)),
                        Size::new(Px(40.0), Px(20.0)),
                    ),
                    selected: true,
                }),
                visibility: NodeGraphToolbarVisibility::WhenSelected,
                position: NodeGraphToolbarPosition::Top,
                align: NodeGraphToolbarAlign::Center,
                size: Size::new(Px(30.0), Px(10.0)),
                gap_px: 8.0,
                offset: Point::new(Px(0.0), Px(0.0)),
                label: Arc::from("Node toolbar"),
                test_id: Arc::from("node_graph.node_toolbar"),
            },
            |cx| vec![cx.text("edit")],
        )
        .expect("selected target should produce a toolbar")
    }

    #[test]
    fn node_toolbar_plan_preserves_retained_positioning_and_visibility_rules() {
        let rect = plan_node_toolbar_rect(
            bounds(),
            Some(NodeGraphNodeToolbarTarget {
                rect: Rect::new(
                    Point::new(Px(60.0), Px(60.0)),
                    Size::new(Px(40.0), Px(20.0)),
                ),
                selected: true,
            }),
            NodeGraphToolbarVisibility::WhenSelected,
            Size::new(Px(30.0), Px(10.0)),
            NodeGraphToolbarPosition::Top,
            NodeGraphToolbarAlign::Center,
            8.0,
            Point::new(Px(0.0), Px(0.0)),
        )
        .expect("visible node toolbar");

        assert_eq!(rect.origin.x.0, 65.0);
        assert_eq!(rect.origin.y.0, 42.0);
        assert_eq!(rect.size, Size::new(Px(30.0), Px(10.0)));

        assert!(
            plan_node_toolbar_rect(
                bounds(),
                Some(NodeGraphNodeToolbarTarget {
                    rect,
                    selected: false,
                }),
                NodeGraphToolbarVisibility::WhenSelected,
                rect.size,
                NodeGraphToolbarPosition::Top,
                NodeGraphToolbarAlign::Center,
                8.0,
                Point::new(Px(0.0), Px(0.0)),
            )
            .is_none()
        );
    }

    #[test]
    fn edge_toolbar_plan_preserves_center_anchor_and_always_visibility() {
        let rect = plan_edge_toolbar_rect(
            bounds(),
            Some(NodeGraphEdgeToolbarTarget {
                center: Point::new(Px(90.0), Px(70.0)),
                selected: false,
            }),
            NodeGraphToolbarVisibility::Always,
            Size::new(Px(20.0), Px(10.0)),
            NodeGraphToolbarAlign::Center,
            NodeGraphToolbarAlign::Center,
            Point::new(Px(0.0), Px(0.0)),
        )
        .expect("always-visible edge toolbar");

        assert_eq!(rect.origin.x.0, 80.0);
        assert_eq!(rect.origin.y.0, 65.0);
        assert_eq!(rect.size, Size::new(Px(20.0), Px(10.0)));
    }

    #[test]
    fn node_toolbar_declarative_composition_builds_absolute_toolbar_without_retained_widget() {
        let root = render_node_toolbar();

        let ElementKind::Container(container) = &root.kind else {
            panic!("node toolbar root should be a declarative container");
        };
        assert_eq!(container.layout.position, PositionStyle::Absolute);
        assert_eq!(container.layout.inset.left, InsetEdge::Px(Px(55.0)));
        assert_eq!(container.layout.inset.top, InsetEdge::Px(Px(22.0)));
        assert_eq!(container.layout.size.width, Length::Px(Px(30.0)));
        assert_eq!(container.layout.size.height, Length::Px(Px(10.0)));

        let semantics = root.semantics_decoration.as_ref().expect("root semantics");
        assert_eq!(semantics.role, Some(SemanticsRole::Toolbar));
        assert_eq!(semantics.label.as_deref(), Some("Node toolbar"));
        assert_eq!(
            semantics.test_id.as_deref(),
            Some("node_graph.node_toolbar")
        );

        assert_eq!(root.children.len(), 1);
        let ElementKind::Text(text) = &root.children[0].kind else {
            panic!("node toolbar should preserve declarative content");
        };
        assert_eq!(text.text.as_ref(), "edit");
    }

    #[test]
    fn edge_toolbar_declarative_composition_builds_absolute_toolbar_without_retained_widget() {
        let mut host = TestUiHost::default();
        let mut runtime = fret_ui::ElementRuntime::new();
        let window = AppWindowId::default();
        let root_bounds = bounds();
        let mut cx = fret_ui::ElementContext::new_for_root_name(
            &mut host,
            &mut runtime,
            window,
            root_bounds,
            "root",
        );
        let root = node_graph_edge_toolbar_element(
            &mut cx,
            NodeGraphEdgeToolbarElementProps {
                bounds: root_bounds,
                target: Some(NodeGraphEdgeToolbarTarget {
                    center: Point::new(Px(90.0), Px(70.0)),
                    selected: false,
                }),
                visibility: NodeGraphToolbarVisibility::Always,
                align_x: NodeGraphToolbarAlign::Center,
                align_y: NodeGraphToolbarAlign::Center,
                size: Size::new(Px(20.0), Px(10.0)),
                offset: Point::new(Px(0.0), Px(0.0)),
                label: Arc::from("Edge toolbar"),
                test_id: Arc::from("node_graph.edge_toolbar"),
            },
            |cx| vec![cx.text("edge")],
        )
        .expect("always-visible target should produce a toolbar");

        let ElementKind::Container(container) = &root.kind else {
            panic!("edge toolbar root should be a declarative container");
        };
        assert_eq!(container.layout.inset.left, InsetEdge::Px(Px(70.0)));
        assert_eq!(container.layout.inset.top, InsetEdge::Px(Px(45.0)));

        let semantics = root.semantics_decoration.as_ref().expect("root semantics");
        assert_eq!(semantics.role, Some(SemanticsRole::Toolbar));
        assert_eq!(semantics.label.as_deref(), Some("Edge toolbar"));
        assert_eq!(
            semantics.test_id.as_deref(),
            Some("node_graph.edge_toolbar")
        );
    }
}
