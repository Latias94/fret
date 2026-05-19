use fret_canvas::view::{PanZoom2D, screen_rect_to_canvas_rect, visible_canvas_rect};
use fret_core::{
    Color, Corners, DrawOrder, Edges, Paint, Point, Px, Rect, SceneOp, SemanticsRole, Size,
};
use fret_ui::element::{
    AnyElement, CanvasProps, ContainerProps, Length, SemanticsDecoration, SpacingEdges,
    SpacingLength,
};
use fret_ui::{ElementContext, UiHost};

use crate::core::CanvasPoint;
use crate::ui::NodeGraphStyle;

use super::minimap_projection::{minimap_world_bounds, project_world_rect_to_minimap};

#[derive(Debug, Clone)]
pub(super) struct NodeGraphMiniMapSnapshot {
    pub(super) canvas_bounds: Rect,
    pub(super) pan: CanvasPoint,
    pub(super) zoom: f32,
    pub(super) nodes_window: Vec<Rect>,
}

#[derive(Debug, Clone)]
pub(super) struct NodeGraphMiniMapOverlayElementProps {
    pub(super) style: NodeGraphStyle,
    pub(super) snapshot: NodeGraphMiniMapSnapshot,
}

pub(super) fn node_graph_minimap_overlay_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    props: NodeGraphMiniMapOverlayElementProps,
) -> AnyElement {
    let size = minimap_panel_size(&props.style);
    let style = props.style;
    let snapshot = props.snapshot;
    let semantics_value = format!("zoom {:.3}", snapshot.zoom);
    let paint_style = style.clone();
    let paint_snapshot = snapshot.clone();

    cx.container(minimap_container(size), move |cx| {
        vec![cx.canvas(CanvasProps::default(), move |p| {
            for op in build_minimap_scene_ops(p.bounds(), &paint_style, &paint_snapshot) {
                p.scene().push(op);
            }
        })]
    })
    .attach_semantics(
        SemanticsDecoration::default()
            .role(SemanticsRole::Panel)
            .label("MiniMap")
            .value(semantics_value)
            .test_id("node_graph.minimap"),
    )
}

fn minimap_container(size: Size) -> ContainerProps {
    let mut props = ContainerProps::default();
    props.layout.size.width = Length::Px(size.width);
    props.layout.size.height = Length::Px(size.height);
    props.padding = SpacingEdges::all(SpacingLength::Px(Px(0.0)));
    props.snap_to_device_pixels = true;
    props
}

fn minimap_panel_size(style: &NodeGraphStyle) -> Size {
    Size::new(
        Px(style.paint.minimap_width.max(0.0)),
        Px(style.paint.minimap_height.max(0.0)),
    )
}

fn build_minimap_scene_ops(
    minimap: Rect,
    style: &NodeGraphStyle,
    snapshot: &NodeGraphMiniMapSnapshot,
) -> Vec<SceneOp> {
    let canvas_bounds = snapshot.canvas_bounds;
    let view = PanZoom2D {
        pan: Point::new(Px(snapshot.pan.x), Px(snapshot.pan.y)),
        zoom: snapshot.zoom,
    };
    let viewport = visible_canvas_rect(canvas_bounds, view);
    let node_canvas_rects: Vec<_> = snapshot
        .nodes_window
        .iter()
        .copied()
        .map(|rect| screen_rect_to_canvas_rect(canvas_bounds, view, rect))
        .collect();
    let world = minimap_world_bounds(
        node_canvas_rects.iter().copied(),
        viewport,
        style.paint.minimap_world_padding.max(0.0),
    );

    let mut ops = Vec::with_capacity(2 + node_canvas_rects.len());
    ops.push(SceneOp::Quad {
        order: DrawOrder(20_000),
        rect: minimap,
        background: Paint::Solid(style.paint.context_menu_background).into(),
        border: Edges::all(Px(1.0)),
        border_paint: Paint::Solid(style.paint.context_menu_border).into(),
        corner_radii: Corners::all(Px(style.paint.context_menu_corner_radius)),
    });

    for rect in node_canvas_rects {
        ops.push(SceneOp::Quad {
            order: DrawOrder(20_001),
            rect: project_world_rect_to_minimap(minimap, world, rect),
            background: Paint::Solid(style.paint.node_background).into(),
            border: Edges::all(Px(0.5)),
            border_paint: Paint::Solid(style.paint.node_border).into(),
            corner_radii: Corners::all(Px(2.0)),
        });
    }

    ops.push(SceneOp::Quad {
        order: DrawOrder(20_002),
        rect: project_world_rect_to_minimap(minimap, world, viewport),
        background: Paint::Solid(Color {
            a: 0.12,
            ..style.paint.node_border_selected
        })
        .into(),
        border: Edges::all(Px(1.0)),
        border_paint: Paint::Solid(style.paint.node_border_selected).into(),
        corner_radii: Corners::all(Px(2.0)),
    });

    ops
}

#[cfg(test)]
mod tests {
    use std::any::{Any, TypeId};
    use std::collections::HashMap;

    use fret_core::{
        AppWindowId, DrawOrder, Point, PointerId, Px, Rect, SceneOp, SemanticsRole, Size,
    };
    use fret_runtime::{
        ClipboardToken, CommandRegistry, CommandsHost, DragHost, DragKindId, DragSession, Effect,
        EffectSink, FrameId, GlobalsHost, ImageUploadToken, ModelHost, ModelId, ModelStore,
        ModelsHost, ShareSheetToken, TickId, TimeHost, TimerToken,
    };
    use fret_ui::element::{ElementKind, Length};

    use crate::core::CanvasPoint;
    use crate::ui::NodeGraphStyle;
    use crate::ui::overlays::minimap_declarative::{
        NodeGraphMiniMapOverlayElementProps, NodeGraphMiniMapSnapshot, build_minimap_scene_ops,
        minimap_panel_size, node_graph_minimap_overlay_element,
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

    fn snapshot() -> NodeGraphMiniMapSnapshot {
        NodeGraphMiniMapSnapshot {
            canvas_bounds: Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(800.0), Px(600.0)),
            ),
            pan: CanvasPoint { x: 0.0, y: 0.0 },
            zoom: 2.0,
            nodes_window: vec![Rect::new(
                Point::new(Px(20.0), Px(30.0)),
                Size::new(Px(80.0), Px(40.0)),
            )],
        }
    }

    fn render_minimap(
        style: NodeGraphStyle,
        snapshot: NodeGraphMiniMapSnapshot,
    ) -> fret_ui::element::AnyElement {
        let mut host = TestUiHost::default();
        let mut runtime = fret_ui::ElementRuntime::new();
        let window = AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(300.0)),
        );
        let mut cx = fret_ui::ElementContext::new_for_root_name(
            &mut host,
            &mut runtime,
            window,
            bounds,
            "root",
        );
        node_graph_minimap_overlay_element(
            &mut cx,
            NodeGraphMiniMapOverlayElementProps { style, snapshot },
        )
    }

    #[test]
    fn minimap_declarative_composition_builds_canvas_panel_without_retained_widget() {
        let mut style = NodeGraphStyle::default();
        style.paint.minimap_width = 180.0;
        style.paint.minimap_height = 110.0;
        let expected = minimap_panel_size(&style);

        let root = render_minimap(style, snapshot());

        let ElementKind::Container(panel) = &root.kind else {
            panic!("minimap root should be a declarative container");
        };
        assert_eq!(panel.layout.size.width, Length::Px(expected.width));
        assert_eq!(panel.layout.size.height, Length::Px(expected.height));
        let semantics = root.semantics_decoration.as_ref().expect("root semantics");
        assert_eq!(semantics.role, Some(SemanticsRole::Panel));
        assert_eq!(semantics.label.as_deref(), Some("MiniMap"));
        assert_eq!(semantics.value.as_deref(), Some("zoom 2.000"));
        assert_eq!(semantics.test_id.as_deref(), Some("node_graph.minimap"));

        assert_eq!(root.children.len(), 1);
        let ElementKind::Canvas(canvas) = &root.children[0].kind else {
            panic!("minimap root should contain a declarative canvas");
        };
        assert_eq!(canvas.layout.size.width, Length::Fill);
        assert_eq!(canvas.layout.size.height, Length::Fill);
    }

    #[test]
    fn minimap_declarative_paint_plan_emits_panel_nodes_and_viewport_markers() {
        let style = NodeGraphStyle::default();
        let minimap = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(160.0), Px(100.0)),
        );

        let ops = build_minimap_scene_ops(minimap, &style, &snapshot());

        assert_eq!(ops.len(), 3);
        let SceneOp::Quad {
            order: panel_order,
            rect: panel,
            ..
        } = ops[0]
        else {
            panic!("first minimap op should paint the panel");
        };
        assert_eq!(panel_order, DrawOrder(20_000));
        assert_eq!(panel, minimap);

        let SceneOp::Quad {
            order: node_order,
            rect: node_marker,
            ..
        } = ops[1]
        else {
            panic!("second minimap op should paint a node marker");
        };
        assert_eq!(node_order, DrawOrder(20_001));
        assert!(node_marker.size.width.0 >= 1.0);
        assert!(node_marker.size.height.0 >= 1.0);

        let SceneOp::Quad {
            order: viewport_order,
            rect: viewport_marker,
            ..
        } = ops[2]
        else {
            panic!("last minimap op should paint the viewport marker");
        };
        assert_eq!(viewport_order, DrawOrder(20_002));
        assert!(viewport_marker.size.width.0 > node_marker.size.width.0);
        assert!(viewport_marker.size.height.0 > node_marker.size.height.0);
    }
}
