use std::sync::Arc;

use fret_core::{Point, Px, Rect, SemanticsRole, Size};
use fret_runtime::{Model, ModelHost};
use fret_ui::element::{
    AnyElement, ContainerProps, InsetEdge, Length, PositionStyle, SemanticsDecoration,
    SpacingEdges, SpacingLength,
};
use fret_ui::layout_constraints::{AvailableSpace, LayoutConstraints, LayoutSize};
use fret_ui::{ElementContext, UiHost};

use crate::core::{EdgeId, NodeId};
use crate::io::NodeGraphViewState;
use crate::ui::NodeGraphInternalsStore;

use super::toolbar_layout_policy::{
    plan_edge_toolbar_child_layout, plan_node_toolbar_child_layout, visible_toolbar_anchor,
};
use super::toolbar_policy::{
    NodeGraphToolbarAlign, NodeGraphToolbarPosition, NodeGraphToolbarSize,
    NodeGraphToolbarVisibility, resolve_edge_toolbar_window_target,
    resolve_node_toolbar_window_target,
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
pub(super) struct NodeGraphNodeToolbarHostElementProps {
    pub(super) bounds: Rect,
    pub(super) target: Option<NodeGraphNodeToolbarTarget>,
    pub(super) visibility: NodeGraphToolbarVisibility,
    pub(super) position: NodeGraphToolbarPosition,
    pub(super) align: NodeGraphToolbarAlign,
    pub(super) size: NodeGraphToolbarSize,
    pub(super) gap_px: f32,
    pub(super) offset: Point,
    pub(super) label: Arc<str>,
    pub(super) test_id: Arc<str>,
    pub(super) focus_fallback: Option<fret_ui::GlobalElementId>,
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

#[derive(Debug, Clone)]
pub(super) struct NodeGraphEdgeToolbarHostElementProps {
    pub(super) bounds: Rect,
    pub(super) target: Option<NodeGraphEdgeToolbarTarget>,
    pub(super) visibility: NodeGraphToolbarVisibility,
    pub(super) align_x: NodeGraphToolbarAlign,
    pub(super) align_y: NodeGraphToolbarAlign,
    pub(super) size: NodeGraphToolbarSize,
    pub(super) offset: Point,
    pub(super) label: Arc<str>,
    pub(super) test_id: Arc<str>,
    pub(super) focus_fallback: Option<fret_ui::GlobalElementId>,
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

pub(super) fn node_graph_node_toolbar_host_element<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    props: NodeGraphNodeToolbarHostElementProps,
    children: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    let layout_props = props.clone();
    toolbar_host_element(
        cx,
        props.bounds,
        props.label,
        props.test_id,
        props.focus_fallback,
        move |cx, child| {
            let size = resolve_declarative_toolbar_child_size(cx, layout_props.size, child);
            plan_node_toolbar_rect(
                layout_props.bounds,
                layout_props.target,
                layout_props.visibility,
                size,
                layout_props.position,
                layout_props.align,
                layout_props.gap_px,
                layout_props.offset,
            )
        },
        children,
    )
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

pub(super) fn node_graph_edge_toolbar_host_element<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    props: NodeGraphEdgeToolbarHostElementProps,
    children: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    let layout_props = props.clone();
    toolbar_host_element(
        cx,
        props.bounds,
        props.label,
        props.test_id,
        props.focus_fallback,
        move |cx, child| {
            let size = resolve_declarative_toolbar_child_size(cx, layout_props.size, child);
            plan_edge_toolbar_rect(
                layout_props.bounds,
                layout_props.target,
                layout_props.visibility,
                size,
                layout_props.align_x,
                layout_props.align_y,
                layout_props.offset,
            )
        },
        children,
    )
}

fn toolbar_host_element<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    bounds: Rect,
    label: Arc<str>,
    test_id: Arc<str>,
    focus_fallback: Option<fret_ui::GlobalElementId>,
    layout_child_rect: impl for<'p, 'q> Fn(
        &mut fret_ui::managed_surface::ManagedSurfaceLayoutCx<'p, 'q, H>,
        fret_core::NodeId,
    ) -> Option<Rect>
    + 'static,
    children: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    let mut surface = fret_ui::element::ManagedSurfaceProps::default();
    surface.layout.position = PositionStyle::Absolute;
    surface.layout.inset.left = InsetEdge::Px(Px(0.0));
    surface.layout.inset.top = InsetEdge::Px(Px(0.0));
    surface.layout.size.width = Length::Px(bounds.size.width);
    surface.layout.size.height = Length::Px(bounds.size.height);

    cx.managed_surface(
        surface,
        move |cx| {
            let Some(child) = cx.children().first().copied() else {
                return;
            };
            let rect = layout_child_rect(cx, child)
                .unwrap_or_else(|| hidden_toolbar_child_rect(cx.bounds()));
            cx.layout_child(child, rect);
            cx.set_hit_test_rects([rect]);
            if rect.size.width.0 <= 0.0
                && rect.size.height.0 <= 0.0
                && cx.focus_in_subtree()
                && let Some(focus_fallback) = focus_fallback
            {
                cx.request_focus_element(focus_fallback);
            }
        },
        move |cx| {
            for child in cx.children().to_vec() {
                if let Some(bounds) = cx.child_bounds(child) {
                    cx.paint_child(child, bounds);
                }
            }
        },
        children,
    )
    .attach_semantics(
        SemanticsDecoration::default()
            .role(SemanticsRole::Toolbar)
            .label(label)
            .test_id(test_id),
    )
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
    let target = visible_toolbar_anchor(
        target.map(|target| (target.rect, target.selected)),
        visibility,
    );
    plan_node_toolbar_child_layout(bounds, target, size, position, align, gap_px, offset).rect()
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
    let target = visible_toolbar_anchor(
        target.map(|target| (target.center, target.selected)),
        visibility,
    );
    plan_edge_toolbar_child_layout(bounds, target, size, align_x, align_y, offset).rect()
}

pub(super) fn resolve_node_toolbar_declarative_target<H: ModelHost>(
    view_state: &Model<NodeGraphViewState>,
    requested_node: Option<NodeId>,
    internals: &NodeGraphInternalsStore,
    host: &H,
) -> Option<NodeGraphNodeToolbarTarget> {
    resolve_node_toolbar_window_target(view_state, requested_node, internals, host)
        .map(|(rect, selected)| NodeGraphNodeToolbarTarget { rect, selected })
}

pub(super) fn resolve_edge_toolbar_declarative_target<H: ModelHost>(
    view_state: &Model<NodeGraphViewState>,
    requested_edge: Option<EdgeId>,
    internals: &NodeGraphInternalsStore,
    host: &H,
) -> Option<NodeGraphEdgeToolbarTarget> {
    resolve_edge_toolbar_window_target(view_state, requested_edge, internals, host)
        .map(|(center, selected)| NodeGraphEdgeToolbarTarget { center, selected })
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

fn resolve_declarative_toolbar_child_size<H: UiHost>(
    cx: &mut fret_ui::managed_surface::ManagedSurfaceLayoutCx<'_, '_, H>,
    size: NodeGraphToolbarSize,
    child: fret_core::NodeId,
) -> Size {
    match size {
        NodeGraphToolbarSize::Fixed(size) => size,
        NodeGraphToolbarSize::Auto => {
            let bounds = cx.bounds();
            cx.measure_child(
                child,
                LayoutConstraints::new(
                    LayoutSize::new(None, None),
                    LayoutSize::new(
                        AvailableSpace::Definite(bounds.size.width),
                        AvailableSpace::Definite(bounds.size.height),
                    ),
                ),
            )
        }
    }
}

fn hidden_toolbar_child_rect(bounds: Rect) -> Rect {
    Rect::new(bounds.origin, Size::new(Px(0.0), Px(0.0)))
}

#[cfg(test)]
fn pointer_region_child<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    size: Size,
    test_id: &'static str,
    on_down: Option<Arc<std::sync::atomic::AtomicUsize>>,
) -> AnyElement {
    let mut props = fret_ui::element::PointerRegionProps::default();
    props.layout.size.width = Length::Px(size.width);
    props.layout.size.height = Length::Px(size.height);
    cx.pointer_region(props, move |cx| {
        if let Some(on_down) = on_down {
            cx.pointer_region_on_pointer_down(Arc::new(move |_host, _cx, down| {
                if down.button == fret_core::MouseButton::Left {
                    on_down.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    return true;
                }
                false
            }));
        }
        Vec::new()
    })
    .test_id(test_id)
}

#[cfg(test)]
mod tests {
    use std::any::{Any, TypeId};
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use fret_core::{
        AppWindowId, Event, Modifiers, MouseButton, Point, PointerEvent, PointerId, PointerType,
        Px, Rect, SemanticsRole, Size,
    };
    use fret_core::{
        MaterialDescriptor, MaterialId, MaterialRegistrationError, PathCommand, PathConstraints,
        PathId, PathMetrics, PathService, PathStyle, SvgId, SvgService, TextConstraints,
        TextMetrics, TextService,
    };
    use fret_runtime::{
        ClipboardToken, CommandRegistry, CommandsHost, DragHost, DragKindId, DragSession, Effect,
        EffectSink, FrameId, GlobalsHost, ImageUploadToken, ModelHost, ModelId, ModelStore,
        ModelsHost, ShareSheetToken, TickId, TimeHost, TimerToken,
    };
    use fret_ui::element::{
        ContainerProps, ElementKind, InsetEdge, Length, PositionStyle, StackProps,
    };

    use crate::core::{EdgeId, NodeId};
    use crate::io::NodeGraphViewState;
    use crate::ui::internals::{NodeGraphInternalsSnapshot, NodeGraphInternalsStore};
    use crate::ui::overlays::toolbar_policy::{
        NodeGraphToolbarAlign, NodeGraphToolbarPosition, NodeGraphToolbarSize,
        NodeGraphToolbarVisibility,
    };
    use crate::ui::overlays::toolbars_declarative::{
        NodeGraphEdgeToolbarElementProps, NodeGraphEdgeToolbarHostElementProps,
        NodeGraphEdgeToolbarTarget, NodeGraphNodeToolbarElementProps,
        NodeGraphNodeToolbarHostElementProps, NodeGraphNodeToolbarTarget,
        node_graph_edge_toolbar_element, node_graph_edge_toolbar_host_element,
        node_graph_node_toolbar_element, node_graph_node_toolbar_host_element,
        plan_edge_toolbar_rect, plan_node_toolbar_rect, pointer_region_child,
        resolve_edge_toolbar_declarative_target, resolve_node_toolbar_declarative_target,
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

    #[test]
    fn node_toolbar_declarative_target_resolution_uses_view_state_and_internals() {
        let mut host = TestUiHost::default();
        let node_a = NodeId::from_u128(1201);
        let node_b = NodeId::from_u128(1202);
        let missing = NodeId::from_u128(1203);
        let rect_a = Rect::new(
            Point::new(Px(40.0), Px(50.0)),
            Size::new(Px(80.0), Px(30.0)),
        );
        let rect_b = Rect::new(
            Point::new(Px(140.0), Px(150.0)),
            Size::new(Px(90.0), Px(40.0)),
        );
        let mut view = NodeGraphViewState::default();
        view.selected_nodes = vec![node_b];
        let view = host.models_mut().insert(view);

        let internals = NodeGraphInternalsStore::new();
        let mut snapshot = NodeGraphInternalsSnapshot::default();
        snapshot.nodes_window.insert(node_a, rect_a);
        snapshot.nodes_window.insert(node_b, rect_b);
        internals.update(snapshot);

        assert_eq!(
            resolve_node_toolbar_declarative_target(&view, None, &internals, &host),
            Some(NodeGraphNodeToolbarTarget {
                rect: rect_b,
                selected: true,
            })
        );
        assert_eq!(
            resolve_node_toolbar_declarative_target(&view, Some(node_a), &internals, &host),
            Some(NodeGraphNodeToolbarTarget {
                rect: rect_a,
                selected: false,
            })
        );
        assert_eq!(
            resolve_node_toolbar_declarative_target(&view, Some(node_b), &internals, &host),
            Some(NodeGraphNodeToolbarTarget {
                rect: rect_b,
                selected: true,
            })
        );
        assert_eq!(
            resolve_node_toolbar_declarative_target(&view, Some(missing), &internals, &host),
            None,
            "a selected or requested node without internals geometry must keep the toolbar hidden"
        );
    }

    #[test]
    fn edge_toolbar_declarative_target_resolution_uses_view_state_and_internals() {
        let mut host = TestUiHost::default();
        let edge_a = EdgeId::from_u128(2201);
        let edge_b = EdgeId::from_u128(2202);
        let missing = EdgeId::from_u128(2203);
        let center_a = Point::new(Px(70.0), Px(80.0));
        let center_b = Point::new(Px(170.0), Px(180.0));
        let mut view = NodeGraphViewState::default();
        view.selected_edges = vec![edge_b];
        let view = host.models_mut().insert(view);

        let internals = NodeGraphInternalsStore::new();
        let mut snapshot = NodeGraphInternalsSnapshot::default();
        snapshot.edge_centers_window.insert(edge_a, center_a);
        snapshot.edge_centers_window.insert(edge_b, center_b);
        internals.update(snapshot);

        assert_eq!(
            resolve_edge_toolbar_declarative_target(&view, None, &internals, &host),
            Some(NodeGraphEdgeToolbarTarget {
                center: center_b,
                selected: true,
            })
        );
        assert_eq!(
            resolve_edge_toolbar_declarative_target(&view, Some(edge_a), &internals, &host),
            Some(NodeGraphEdgeToolbarTarget {
                center: center_a,
                selected: false,
            })
        );
        assert_eq!(
            resolve_edge_toolbar_declarative_target(&view, Some(edge_b), &internals, &host),
            Some(NodeGraphEdgeToolbarTarget {
                center: center_b,
                selected: true,
            })
        );
        assert_eq!(
            resolve_edge_toolbar_declarative_target(&view, Some(missing), &internals, &host),
            None,
            "an edge without internals center must keep the toolbar hidden"
        );
    }

    #[test]
    fn node_toolbar_declarative_host_auto_measures_and_places_child_without_retained_widget() {
        let mut host = TestUiHost::default();
        let mut ui = fret_ui::UiTree::<TestUiHost>::new();
        let mut services = FakeUiServices::default();
        let window = AppWindowId::default();
        let root_bounds = bounds();
        ui.set_window(window);

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut host,
            &mut services,
            window,
            root_bounds,
            "node-toolbar-auto-host",
            |cx| {
                vec![node_graph_node_toolbar_host_element(
                    cx,
                    NodeGraphNodeToolbarHostElementProps {
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
                        size: NodeGraphToolbarSize::Auto,
                        gap_px: 8.0,
                        offset: Point::new(Px(0.0), Px(0.0)),
                        label: Arc::from("Node toolbar"),
                        test_id: Arc::from("node_graph.node_toolbar"),
                        focus_fallback: None,
                    },
                    |cx| {
                        let mut child = ContainerProps::default();
                        child.layout.size.width = Length::Px(Px(42.0));
                        child.layout.size.height = Length::Px(Px(16.0));
                        vec![cx.container(child, |_| Vec::new())]
                    },
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut host, &mut services, root_bounds, 1.0);

        let toolbar = ui.children(root)[0];
        let child = ui.children(toolbar)[0];
        let expected = plan_node_toolbar_rect(
            root_bounds,
            Some(NodeGraphNodeToolbarTarget {
                rect: Rect::new(
                    Point::new(Px(60.0), Px(60.0)),
                    Size::new(Px(40.0), Px(20.0)),
                ),
                selected: true,
            }),
            NodeGraphToolbarVisibility::WhenSelected,
            Size::new(Px(42.0), Px(16.0)),
            NodeGraphToolbarPosition::Top,
            NodeGraphToolbarAlign::Center,
            8.0,
            Point::new(Px(0.0), Px(0.0)),
        )
        .expect("auto-measured node toolbar should be visible");

        assert_eq!(ui.debug_node_bounds(toolbar), Some(root_bounds));
        assert_eq!(ui.debug_node_bounds(child), Some(expected));

        let root_element = render_node_toolbar();
        let semantics = root_element
            .semantics_decoration
            .as_ref()
            .expect("toolbar semantics");
        assert_eq!(semantics.role, Some(SemanticsRole::Toolbar));
    }

    #[test]
    fn edge_toolbar_declarative_host_auto_measures_and_hides_child_without_retained_widget() {
        let mut host = TestUiHost::default();
        let mut ui = fret_ui::UiTree::<TestUiHost>::new();
        let mut services = FakeUiServices::default();
        let window = AppWindowId::default();
        let root_bounds = bounds();
        ui.set_window(window);

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut host,
            &mut services,
            window,
            root_bounds,
            "edge-toolbar-auto-host",
            |cx| {
                vec![node_graph_edge_toolbar_host_element(
                    cx,
                    NodeGraphEdgeToolbarHostElementProps {
                        bounds: root_bounds,
                        target: Some(NodeGraphEdgeToolbarTarget {
                            center: Point::new(Px(90.0), Px(70.0)),
                            selected: true,
                        }),
                        visibility: NodeGraphToolbarVisibility::Always,
                        align_x: NodeGraphToolbarAlign::Center,
                        align_y: NodeGraphToolbarAlign::Center,
                        size: NodeGraphToolbarSize::Auto,
                        offset: Point::new(Px(0.0), Px(0.0)),
                        label: Arc::from("Edge toolbar"),
                        test_id: Arc::from("node_graph.edge_toolbar"),
                        focus_fallback: None,
                    },
                    |cx| {
                        let mut child = ContainerProps::default();
                        child.layout.size.width = Length::Px(Px(44.0));
                        child.layout.size.height = Length::Px(Px(18.0));
                        vec![cx.container(child, |_| Vec::new())]
                    },
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut host, &mut services, root_bounds, 1.0);

        let toolbar = ui.children(root)[0];
        let child = ui.children(toolbar)[0];
        let expected = plan_edge_toolbar_rect(
            root_bounds,
            Some(NodeGraphEdgeToolbarTarget {
                center: Point::new(Px(90.0), Px(70.0)),
                selected: true,
            }),
            NodeGraphToolbarVisibility::Always,
            Size::new(Px(44.0), Px(18.0)),
            NodeGraphToolbarAlign::Center,
            NodeGraphToolbarAlign::Center,
            Point::new(Px(0.0), Px(0.0)),
        )
        .expect("auto-measured edge toolbar should be visible");
        assert_eq!(ui.debug_node_bounds(child), Some(expected));

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut host,
            &mut services,
            window,
            root_bounds,
            "edge-toolbar-auto-host",
            |cx| {
                vec![node_graph_edge_toolbar_host_element(
                    cx,
                    NodeGraphEdgeToolbarHostElementProps {
                        bounds: root_bounds,
                        target: None,
                        visibility: NodeGraphToolbarVisibility::Always,
                        align_x: NodeGraphToolbarAlign::Center,
                        align_y: NodeGraphToolbarAlign::Center,
                        size: NodeGraphToolbarSize::Auto,
                        offset: Point::new(Px(0.0), Px(0.0)),
                        label: Arc::from("Edge toolbar"),
                        test_id: Arc::from("node_graph.edge_toolbar"),
                        focus_fallback: None,
                    },
                    |cx| {
                        let mut child = ContainerProps::default();
                        child.layout.size.width = Length::Px(Px(44.0));
                        child.layout.size.height = Length::Px(Px(18.0));
                        vec![cx.container(child, |_| Vec::new())]
                    },
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut host, &mut services, root_bounds, 1.0);

        let toolbar = ui.children(root)[0];
        let child = ui.children(toolbar)[0];
        assert_eq!(
            ui.debug_node_bounds(child),
            Some(Rect::new(root_bounds.origin, Size::new(Px(0.0), Px(0.0))))
        );
    }

    #[test]
    fn node_toolbar_declarative_host_falls_through_outside_child_and_intercepts_inside() {
        let mut host = TestUiHost::default();
        let mut ui = fret_ui::UiTree::<TestUiHost>::new();
        let mut services = FakeUiServices::default();
        let window = AppWindowId::default();
        let root_bounds = bounds();
        ui.set_window(window);
        let underlay_downs = Arc::new(AtomicUsize::new(0));
        let toolbar_downs = Arc::new(AtomicUsize::new(0));
        let underlay_downs_for_render = Arc::clone(&underlay_downs);
        let toolbar_downs_for_render = Arc::clone(&toolbar_downs);

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut host,
            &mut services,
            window,
            root_bounds,
            "node-toolbar-hit-test-mask",
            move |cx| {
                let mut stack = StackProps::default();
                stack.layout.size.width = Length::Px(root_bounds.size.width);
                stack.layout.size.height = Length::Px(root_bounds.size.height);
                vec![cx.stack_props(stack, move |cx| {
                    let underlay = pointer_region_child(
                        cx,
                        root_bounds.size,
                        "node_graph.underlay",
                        Some(Arc::clone(&underlay_downs_for_render)),
                    );
                    let toolbar = node_graph_node_toolbar_host_element(
                        cx,
                        NodeGraphNodeToolbarHostElementProps {
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
                            size: NodeGraphToolbarSize::Fixed(Size::new(Px(30.0), Px(10.0))),
                            gap_px: 8.0,
                            offset: Point::new(Px(0.0), Px(0.0)),
                            label: Arc::from("Node toolbar"),
                            test_id: Arc::from("node_graph.node_toolbar"),
                            focus_fallback: None,
                        },
                        |cx| {
                            vec![pointer_region_child(
                                cx,
                                Size::new(Px(30.0), Px(10.0)),
                                "node_graph.node_toolbar.child",
                                Some(Arc::clone(&toolbar_downs_for_render)),
                            )]
                        },
                    );
                    vec![underlay, toolbar]
                })]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut host, &mut services, root_bounds, 1.0);

        let stack = ui.children(root)[0];
        let underlay = ui.children(stack)[0];
        let toolbar = ui.children(stack)[1];
        let toolbar_child = ui.children(toolbar)[0];
        let expected = plan_node_toolbar_rect(
            root_bounds,
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
        assert_eq!(ui.debug_node_bounds(toolbar_child), Some(expected));
        assert_eq!(
            ui.debug_hit_test(Point::new(Px(11.0), Px(21.0))).hit,
            Some(underlay),
            "node toolbar host must not block canvas input outside the child rect"
        );
        assert_eq!(
            ui.debug_hit_test(Point::new(
                Px(expected.origin.x.0 + 1.0),
                Px(expected.origin.y.0 + 1.0),
            ))
            .hit,
            Some(toolbar_child),
            "node toolbar child remains the input target inside its child rect"
        );
        dispatch_pointer_down_at(
            &mut ui,
            &mut host,
            &mut services,
            Point::new(Px(11.0), Px(21.0)),
        );
        assert_eq!(underlay_downs.load(Ordering::Relaxed), 1);
        assert_eq!(toolbar_downs.load(Ordering::Relaxed), 0);
        dispatch_pointer_down_at(
            &mut ui,
            &mut host,
            &mut services,
            Point::new(Px(expected.origin.x.0 + 1.0), Px(expected.origin.y.0 + 1.0)),
        );
        assert_eq!(underlay_downs.load(Ordering::Relaxed), 1);
        assert_eq!(toolbar_downs.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn edge_toolbar_declarative_host_falls_through_outside_child_and_intercepts_inside() {
        let mut host = TestUiHost::default();
        let mut ui = fret_ui::UiTree::<TestUiHost>::new();
        let mut services = FakeUiServices::default();
        let window = AppWindowId::default();
        let root_bounds = bounds();
        ui.set_window(window);
        let underlay_downs = Arc::new(AtomicUsize::new(0));
        let toolbar_downs = Arc::new(AtomicUsize::new(0));
        let underlay_downs_for_render = Arc::clone(&underlay_downs);
        let toolbar_downs_for_render = Arc::clone(&toolbar_downs);

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut host,
            &mut services,
            window,
            root_bounds,
            "edge-toolbar-hit-test-mask",
            move |cx| {
                let mut stack = StackProps::default();
                stack.layout.size.width = Length::Px(root_bounds.size.width);
                stack.layout.size.height = Length::Px(root_bounds.size.height);
                vec![cx.stack_props(stack, move |cx| {
                    let underlay = pointer_region_child(
                        cx,
                        root_bounds.size,
                        "node_graph.underlay",
                        Some(Arc::clone(&underlay_downs_for_render)),
                    );
                    let toolbar = node_graph_edge_toolbar_host_element(
                        cx,
                        NodeGraphEdgeToolbarHostElementProps {
                            bounds: root_bounds,
                            target: Some(NodeGraphEdgeToolbarTarget {
                                center: Point::new(Px(90.0), Px(70.0)),
                                selected: true,
                            }),
                            visibility: NodeGraphToolbarVisibility::Always,
                            align_x: NodeGraphToolbarAlign::Center,
                            align_y: NodeGraphToolbarAlign::Center,
                            size: NodeGraphToolbarSize::Fixed(Size::new(Px(20.0), Px(10.0))),
                            offset: Point::new(Px(0.0), Px(0.0)),
                            label: Arc::from("Edge toolbar"),
                            test_id: Arc::from("node_graph.edge_toolbar"),
                            focus_fallback: None,
                        },
                        |cx| {
                            vec![pointer_region_child(
                                cx,
                                Size::new(Px(20.0), Px(10.0)),
                                "node_graph.edge_toolbar.child",
                                Some(Arc::clone(&toolbar_downs_for_render)),
                            )]
                        },
                    );
                    vec![underlay, toolbar]
                })]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut host, &mut services, root_bounds, 1.0);

        let stack = ui.children(root)[0];
        let underlay = ui.children(stack)[0];
        let toolbar = ui.children(stack)[1];
        let toolbar_child = ui.children(toolbar)[0];
        let expected = plan_edge_toolbar_rect(
            root_bounds,
            Some(NodeGraphEdgeToolbarTarget {
                center: Point::new(Px(90.0), Px(70.0)),
                selected: true,
            }),
            NodeGraphToolbarVisibility::Always,
            Size::new(Px(20.0), Px(10.0)),
            NodeGraphToolbarAlign::Center,
            NodeGraphToolbarAlign::Center,
            Point::new(Px(0.0), Px(0.0)),
        )
        .expect("visible edge toolbar");
        assert_eq!(ui.debug_node_bounds(toolbar_child), Some(expected));
        assert_eq!(
            ui.debug_hit_test(Point::new(Px(11.0), Px(21.0))).hit,
            Some(underlay),
            "edge toolbar host must not block canvas input outside the child rect"
        );
        assert_eq!(
            ui.debug_hit_test(Point::new(
                Px(expected.origin.x.0 + 1.0),
                Px(expected.origin.y.0 + 1.0),
            ))
            .hit,
            Some(toolbar_child),
            "edge toolbar child remains the input target inside its child rect"
        );
        dispatch_pointer_down_at(
            &mut ui,
            &mut host,
            &mut services,
            Point::new(Px(11.0), Px(21.0)),
        );
        assert_eq!(underlay_downs.load(Ordering::Relaxed), 1);
        assert_eq!(toolbar_downs.load(Ordering::Relaxed), 0);
        dispatch_pointer_down_at(
            &mut ui,
            &mut host,
            &mut services,
            Point::new(Px(expected.origin.x.0 + 1.0), Px(expected.origin.y.0 + 1.0)),
        );
        assert_eq!(underlay_downs.load(Ordering::Relaxed), 1);
        assert_eq!(toolbar_downs.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn toolbar_declarative_host_hides_child_and_releases_focus_to_canvas() {
        fn render_focus_release_root(
            ui: &mut fret_ui::UiTree<TestUiHost>,
            host: &mut TestUiHost,
            services: &mut FakeUiServices,
            window: AppWindowId,
            root_bounds: Rect,
            selected: bool,
        ) -> fret_core::NodeId {
            fret_ui::declarative::render_root(
                ui,
                host,
                services,
                window,
                root_bounds,
                "node-toolbar-focus-release",
                move |cx| {
                    let mut stack = StackProps::default();
                    stack.layout.size.width = Length::Px(root_bounds.size.width);
                    stack.layout.size.height = Length::Px(root_bounds.size.height);
                    vec![cx.stack_props(stack, move |cx| {
                        let underlay =
                            pointer_region_child(cx, root_bounds.size, "node_graph.underlay", None);
                        let focus_fallback = Some(underlay.id);
                        let toolbar = node_graph_node_toolbar_host_element(
                            cx,
                            NodeGraphNodeToolbarHostElementProps {
                                bounds: root_bounds,
                                target: Some(NodeGraphNodeToolbarTarget {
                                    rect: Rect::new(
                                        Point::new(Px(60.0), Px(60.0)),
                                        Size::new(Px(40.0), Px(20.0)),
                                    ),
                                    selected,
                                }),
                                visibility: NodeGraphToolbarVisibility::WhenSelected,
                                position: NodeGraphToolbarPosition::Top,
                                align: NodeGraphToolbarAlign::Center,
                                size: NodeGraphToolbarSize::Fixed(Size::new(Px(30.0), Px(10.0))),
                                gap_px: 8.0,
                                offset: Point::new(Px(0.0), Px(0.0)),
                                label: Arc::from("Node toolbar"),
                                test_id: Arc::from("node_graph.node_toolbar"),
                                focus_fallback,
                            },
                            |cx| {
                                vec![pointer_region_child(
                                    cx,
                                    Size::new(Px(30.0), Px(10.0)),
                                    "node_graph.node_toolbar.child",
                                    None,
                                )]
                            },
                        );
                        vec![underlay, toolbar]
                    })]
                },
            )
        }

        let mut host = TestUiHost::default();
        let mut ui = fret_ui::UiTree::<TestUiHost>::new();
        let mut services = FakeUiServices::default();
        let window = AppWindowId::default();
        let root_bounds = bounds();
        ui.set_window(window);

        let root =
            render_focus_release_root(&mut ui, &mut host, &mut services, window, root_bounds, true);
        ui.set_root(root);
        ui.layout_all(&mut host, &mut services, root_bounds, 1.0);

        let toolbar = ui.children(ui.children(root)[0])[1];
        let toolbar_child = ui.children(toolbar)[0];
        ui.set_focus(Some(toolbar_child));

        let root = render_focus_release_root(
            &mut ui,
            &mut host,
            &mut services,
            window,
            root_bounds,
            false,
        );
        ui.set_root(root);
        ui.layout_all(&mut host, &mut services, root_bounds, 1.0);

        let stack = ui.children(root)[0];
        let underlay = ui.children(stack)[0];
        let toolbar = ui.children(stack)[1];
        let toolbar_child = ui.children(toolbar)[0];

        assert_eq!(
            ui.debug_node_bounds(toolbar_child),
            Some(Rect::new(root_bounds.origin, Size::new(Px(0.0), Px(0.0))))
        );
        assert_eq!(
            ui.focus(),
            Some(underlay),
            "hidden declarative toolbar child should relinquish focus to the canvas-equivalent underlay"
        );
        assert_eq!(
            ui.debug_hit_test(Point::new(Px(11.0), Px(21.0))).hit,
            Some(underlay),
            "hidden toolbar must not leave a full-bounds hit-test blocker behind"
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
            &Event::Pointer(PointerEvent::Down {
                pointer_id: PointerId(0),
                position,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_type: PointerType::Mouse,
            }),
        );
    }
}
