use super::super::super::*;

pub(in crate::ui) fn preview_chart_torture(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    use delinea::data::{Column, DataTable};
    use delinea::{
        AxisKind, AxisPointerSpec, AxisPointerTrigger, AxisPointerType, AxisRange, AxisScale,
        ChartSpec, DatasetSpec, FieldSpec, GridSpec, SeriesEncode, SeriesKind, SeriesSpec,
        TimeAxisScale,
    };
    use fret_chart::ChartCanvas;
    use fret_ui::element::{LayoutStyle, Length, SemanticsProps};
    use fret_ui::retained_bridge::RetainedSubtreeProps;

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: stress canvas charts with pan/zoom (candidate for prepaint-windowed sampling)."),
                cx.text("Use scripted drag+wheel steps to validate correctness and collect perf bundles."),
            ]
        },
    );

    let chart =
        cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
            let dataset_id = delinea::ids::DatasetId::new(1);
            let grid_id = delinea::ids::GridId::new(1);
            let x_axis = delinea::AxisId::new(1);
            let y_axis = delinea::AxisId::new(2);
            let series_id = delinea::ids::SeriesId::new(1);
            let x_field = delinea::FieldId::new(1);
            let y_field = delinea::FieldId::new(2);

            let spec = ChartSpec {
                id: delinea::ids::ChartId::new(1),
                viewport: None,
                datasets: vec![DatasetSpec {
                    id: dataset_id,
                    fields: vec![
                        FieldSpec {
                            id: x_field,
                            column: 0,
                        },
                        FieldSpec {
                            id: y_field,
                            column: 1,
                        },
                    ],
                    ..Default::default()
                }],
                grids: vec![GridSpec { id: grid_id }],
                axes: vec![
                    delinea::AxisSpec {
                        id: x_axis,
                        name: Some("Time".to_string()),
                        kind: AxisKind::X,
                        grid: grid_id,
                        position: None,
                        scale: AxisScale::Time(TimeAxisScale),
                        range: Some(AxisRange::Auto),
                    },
                    delinea::AxisSpec {
                        id: y_axis,
                        name: Some("Value".to_string()),
                        kind: AxisKind::Y,
                        grid: grid_id,
                        position: None,
                        scale: Default::default(),
                        range: Some(AxisRange::Auto),
                    },
                ],
                data_zoom_x: vec![],
                data_zoom_y: vec![],
                tooltip: None,
                axis_pointer: Some(AxisPointerSpec {
                    enabled: true,
                    trigger: AxisPointerTrigger::Axis,
                    pointer_type: AxisPointerType::Line,
                    label: Default::default(),
                    snap: false,
                    trigger_distance_px: 12.0,
                    throttle_px: 0.75,
                }),
                visual_maps: vec![],
                series: vec![SeriesSpec {
                    id: series_id,
                    name: Some("Series".to_string()),
                    kind: SeriesKind::Line,
                    dataset: dataset_id,
                    encode: SeriesEncode {
                        x: x_field,
                        y: y_field,
                        y2: None,
                    },
                    x_axis,
                    y_axis,
                    stack: None,
                    stack_strategy: Default::default(),
                    bar_layout: Default::default(),
                    area_baseline: None,
                    lod: None,
                }],
            };

            let mut layout = LayoutStyle::default();
            layout.size.width = Length::Fill;
            layout.size.height = Length::Px(Px(520.0));

            let props = RetainedSubtreeProps::new::<App>(move |ui| {
                use fret_ui::retained_bridge::UiTreeRetainedExt as _;

                let mut canvas =
                    ChartCanvas::new(spec.clone()).expect("chart spec should be valid");
                canvas.set_input_map(fret_chart::input_map::ChartInputMap::default());

                let base_ms = 1_735_689_600_000.0;
                let interval_ms = 60_000.0;

                let n = 200_000usize;
                let mut x: Vec<f64> = Vec::with_capacity(n);
                let mut y: Vec<f64> = Vec::with_capacity(n);
                for i in 0..n {
                    let t = i as f64 / (n - 1) as f64;
                    let xi = base_ms + interval_ms * i as f64;
                    let theta = t * std::f64::consts::TAU;
                    let yi = (theta * 8.0).sin() * 0.8;
                    x.push(xi);
                    y.push(yi);
                }

                let mut table = DataTable::default();
                table.push_column(Column::F64(x));
                table.push_column(Column::F64(y));
                canvas.engine_mut().datasets_mut().insert(dataset_id, table);

                let node = ui.create_node_retained(canvas);
                ui.set_node_view_cache_flags(node, true, true, false);
                node
            })
            .with_layout(layout);

            let subtree = cx.retained_subtree(props);
            vec![cx.semantics(
                SemanticsProps {
                    role: fret_core::SemanticsRole::Group,
                    test_id: Some(Arc::<str>::from("ui-gallery-chart-torture-root")),
                    ..Default::default()
                },
                |_cx| vec![subtree],
            )]
        });

    vec![header, chart]
}

pub(in crate::ui) fn preview_canvas_cull_torture(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    use fret_canvas::ui::{
        PanZoomCanvasSurfacePanelProps, PanZoomInputPreset, pan_zoom_canvas_surface_panel,
    };
    use fret_canvas::view::{PanZoom2D, visible_canvas_rect};
    use fret_core::{
        Corners, DrawOrder, Edges, FontId, Px, SemanticsRole, TextOverflow, TextStyle, TextWrap,
    };
    use fret_ui::canvas::CanvasTextConstraints;
    use fret_ui::element::{CanvasCachePolicy, Length};
    use std::cmp::Ordering;

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: stress a pan/zoom canvas scene with viewport-driven culling (candidate for prepaint-windowed cull windows)."),
                cx.text("Use scripted middle-drag + wheel steps to validate correctness and collect perf bundles."),
            ]
        },
    );

    let canvas =
        cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
            let bg_even = theme.color_required("background");
            let bg_odd = theme.color_required("muted");
            let fg = theme.color_required("foreground");
            let grid = theme.color_required("border");

            let text_style = TextStyle {
                font: FontId::monospace(),
                size: Px(11.0),
                ..Default::default()
            };

            let mut props = PanZoomCanvasSurfacePanelProps::default();
            props.preset = PanZoomInputPreset::DesktopCanvasCad;
            props.pointer_region.layout.size.width = Length::Fill;
            props.pointer_region.layout.size.height = Length::Px(Px(520.0));
            props.canvas.cache_policy = CanvasCachePolicy::smooth_default();
            props.default_view = PanZoom2D {
                pan: fret_core::Point::new(Px(0.0), Px(0.0)),
                zoom: 1.0,
            };
            props.min_zoom = 0.05;
            props.max_zoom = 64.0;

            let cell_size = 48.0f32;
            let cell_pad = 3.0f32;
            let max_cells = 40_000i64;

            let canvas = pan_zoom_canvas_surface_panel(cx, props, move |painter, paint_cx| {
                let bounds = painter.bounds();

                let Some(transform) = paint_cx.view.render_transform(bounds) else {
                    return;
                };

                let vis = visible_canvas_rect(bounds, paint_cx.view);
                let min_x = vis.origin.x.0;
                let max_x = vis.origin.x.0 + vis.size.width.0;
                let min_y = vis.origin.y.0;
                let max_y = vis.origin.y.0 + vis.size.height.0;

                let start_x = (min_x / cell_size).floor() as i32 - 2;
                let end_x = (max_x / cell_size).ceil() as i32 + 2;
                let start_y = (min_y / cell_size).floor() as i32 - 2;
                let end_y = (max_y / cell_size).ceil() as i32 + 2;

                let x_count = (end_x - start_x + 1).max(0) as i64;
                let y_count = (end_y - start_y + 1).max(0) as i64;
                let estimated = x_count.saturating_mul(y_count);

                let stride = match estimated.cmp(&max_cells) {
                    Ordering::Less | Ordering::Equal => 1i32,
                    Ordering::Greater => {
                        ((estimated as f64 / max_cells as f64).ceil() as i32).max(1)
                    }
                };

                let clip = bounds;
                painter.with_clip_rect(clip, |painter| {
                    painter.with_transform(transform, |painter| {
                        let scope = painter.key_scope(&"ui-gallery-canvas-cull");

                        let mut y = start_y;
                        while y <= end_y {
                            let mut x = start_x;
                            while x <= end_x {
                                let ox = x as f32 * cell_size + cell_pad;
                                let oy = y as f32 * cell_size + cell_pad;
                                let size = cell_size - cell_pad * 2.0;
                                if size.is_finite() && size > 0.0 {
                                    let rect = fret_core::Rect::new(
                                        fret_core::Point::new(Px(ox), Px(oy)),
                                        fret_core::Size::new(Px(size), Px(size)),
                                    );

                                    let background =
                                        if ((x ^ y) & 1) == 0 { bg_even } else { bg_odd };
                                    painter.scene().push(fret_core::SceneOp::Quad {
                                        order: DrawOrder(0),
                                        rect,
                                        background: fret_core::Paint::Solid(background),
                                        border: Edges::all(Px(1.0)),
                                        border_paint: fret_core::Paint::Solid(grid),

                                        corner_radii: Corners::all(Px(4.0)),
                                    });

                                    if x == 0 && y == 0 {
                                        painter.scene().push(fret_core::SceneOp::Quad {
                                            order: DrawOrder(1),
                                            rect,
                                            background: fret_core::Paint::TRANSPARENT,

                                            border: Edges::all(Px(2.0)),
                                            border_paint: fret_core::Paint::Solid(fg),

                                            corner_radii: Corners::all(Px(4.0)),
                                        });
                                    }

                                    if (x % 20) == 0 && (y % 20) == 0 {
                                        let key: u64 = painter.child_key(scope, &(x, y)).into();
                                        let label = format!("{x},{y}");
                                        let origin = fret_core::Point::new(
                                            Px(rect.origin.x.0 + 6.0),
                                            Px(rect.origin.y.0 + 6.0),
                                        );
                                        let _ = painter.text(
                                            key,
                                            DrawOrder(2),
                                            origin,
                                            label,
                                            text_style.clone(),
                                            fg,
                                            CanvasTextConstraints {
                                                max_width: Some(Px(
                                                    (rect.size.width.0 - 12.0).max(0.0)
                                                )),
                                                wrap: TextWrap::None,
                                                overflow: TextOverflow::Clip,
                                            },
                                            painter.scale_factor(),
                                        );
                                    }
                                }

                                x = x.saturating_add(stride);
                            }
                            y = y.saturating_add(stride);
                        }
                    });
                });
            });

            vec![
                canvas.attach_semantics(
                    SemanticsDecoration::default()
                        .role(SemanticsRole::Group)
                        .test_id("ui-gallery-canvas-cull-root"),
                ),
            ]
        });

    vec![header, canvas]
}

pub(in crate::ui) fn preview_node_graph_cull_torture(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    use fret_core::{Px, SemanticsRole};
    use fret_node::io::NodeGraphViewState;
    use fret_node::ui::NodeGraphCanvas;
    use fret_node::{
        Edge, EdgeId, EdgeKind, Graph, GraphId, Node, NodeId, NodeKindKey, Port, PortCapacity,
        PortDirection, PortId, PortKey, PortKind, TypeDesc,
    };
    use fret_ui::element::{LayoutStyle, Length, SemanticsProps};
    use fret_ui::retained_bridge::RetainedSubtreeProps;

    fn uuid_from_tag(tag: u64, ix: u64) -> uuid::Uuid {
        uuid::Uuid::from_u128(((tag as u128) << 64) | (ix as u128))
    }

    fn build_stress_graph(graph_id: GraphId, target_nodes: usize) -> Graph {
        let mut graph = Graph::new(graph_id);

        let add_nodes = target_nodes.saturating_sub(1) / 2;
        let float_nodes = add_nodes.saturating_add(1);

        let cols: usize = 64;
        let x_step = 360.0f32;
        let y_step = 220.0f32;

        let float_x_offset = -260.0f32;
        let float_y_offset = 40.0f32;

        let node_tag = u64::from_le_bytes(*b"NODEGRAF");
        let port_tag = u64::from_le_bytes(*b"PORTGRAF");
        let edge_tag = u64::from_le_bytes(*b"EDGEGRAF");

        let node_id = |ix: u64| NodeId(uuid_from_tag(node_tag, ix));
        let port_id = |ix: u64| PortId(uuid_from_tag(port_tag, ix));
        let edge_id = |ix: u64| EdgeId(uuid_from_tag(edge_tag, ix));

        let mut next_node_ix: u64 = 1;
        let mut next_port_ix: u64 = 1;
        let mut next_edge_ix: u64 = 1;

        let mut float_out_ports: Vec<PortId> = Vec::with_capacity(float_nodes);
        for i in 0..float_nodes {
            let node_id = {
                let id = node_id(next_node_ix);
                next_node_ix = next_node_ix.saturating_add(1);
                id
            };
            let port_out = {
                let id = port_id(next_port_ix);
                next_port_ix = next_port_ix.saturating_add(1);
                id
            };

            let col = i % cols;
            let row = i / cols;
            let x = col as f32 * x_step + float_x_offset;
            let y = row as f32 * y_step + float_y_offset;
            let value = (i as f64) * 0.001;

            graph.nodes.insert(
                node_id,
                Node {
                    kind: NodeKindKey::new("demo.float"),
                    kind_version: 1,
                    pos: fret_node::CanvasPoint { x, y },
                    selectable: None,
                    draggable: None,
                    connectable: None,
                    deletable: None,
                    parent: None,
                    extent: None,
                    expand_parent: None,
                    size: None,
                    hidden: false,
                    collapsed: false,
                    ports: vec![port_out],
                    data: serde_json::json!({ "value": value }),
                },
            );
            graph.ports.insert(
                port_out,
                Port {
                    node: node_id,
                    key: PortKey::new("out"),
                    dir: PortDirection::Out,
                    kind: PortKind::Data,
                    capacity: PortCapacity::Multi,
                    connectable: None,
                    connectable_start: None,
                    connectable_end: None,
                    ty: Some(TypeDesc::Float),
                    data: serde_json::Value::Null,
                },
            );

            float_out_ports.push(port_out);
        }

        let mut prev_out: Option<PortId> = None;
        for i in 0..add_nodes {
            let node_id = {
                let id = node_id(next_node_ix);
                next_node_ix = next_node_ix.saturating_add(1);
                id
            };
            let port_a = {
                let id = port_id(next_port_ix);
                next_port_ix = next_port_ix.saturating_add(1);
                id
            };
            let port_b = {
                let id = port_id(next_port_ix);
                next_port_ix = next_port_ix.saturating_add(1);
                id
            };
            let port_out = {
                let id = port_id(next_port_ix);
                next_port_ix = next_port_ix.saturating_add(1);
                id
            };

            let col = i % cols;
            let row = i / cols;
            let x = col as f32 * x_step;
            let y = row as f32 * y_step;

            graph.nodes.insert(
                node_id,
                Node {
                    kind: NodeKindKey::new("demo.add"),
                    kind_version: 1,
                    pos: fret_node::CanvasPoint { x, y },
                    selectable: None,
                    draggable: None,
                    connectable: None,
                    deletable: None,
                    parent: None,
                    extent: None,
                    expand_parent: None,
                    size: None,
                    hidden: false,
                    collapsed: false,
                    ports: vec![port_a, port_b, port_out],
                    data: serde_json::Value::Null,
                },
            );
            graph.ports.insert(
                port_a,
                Port {
                    node: node_id,
                    key: PortKey::new("a"),
                    dir: PortDirection::In,
                    kind: PortKind::Data,
                    capacity: PortCapacity::Single,
                    connectable: None,
                    connectable_start: None,
                    connectable_end: None,
                    ty: Some(TypeDesc::Float),
                    data: serde_json::Value::Null,
                },
            );
            graph.ports.insert(
                port_b,
                Port {
                    node: node_id,
                    key: PortKey::new("b"),
                    dir: PortDirection::In,
                    kind: PortKind::Data,
                    capacity: PortCapacity::Single,
                    connectable: None,
                    connectable_start: None,
                    connectable_end: None,
                    ty: Some(TypeDesc::Float),
                    data: serde_json::Value::Null,
                },
            );
            graph.ports.insert(
                port_out,
                Port {
                    node: node_id,
                    key: PortKey::new("out"),
                    dir: PortDirection::Out,
                    kind: PortKind::Data,
                    capacity: PortCapacity::Multi,
                    connectable: None,
                    connectable_start: None,
                    connectable_end: None,
                    ty: Some(TypeDesc::Float),
                    data: serde_json::Value::Null,
                },
            );

            let port_a_source = prev_out.unwrap_or(float_out_ports[0]);
            let port_b_source =
                float_out_ports[(i + 1).min(float_out_ports.len().saturating_sub(1))];

            let edge_a = edge_id(next_edge_ix);
            next_edge_ix = next_edge_ix.saturating_add(1);
            graph.edges.insert(
                edge_a,
                Edge {
                    kind: EdgeKind::Data,
                    from: port_a_source,
                    to: port_a,
                    selectable: None,
                    deletable: None,
                    reconnectable: None,
                },
            );

            let edge_b = edge_id(next_edge_ix);
            next_edge_ix = next_edge_ix.saturating_add(1);
            graph.edges.insert(
                edge_b,
                Edge {
                    kind: EdgeKind::Data,
                    from: port_b_source,
                    to: port_b,
                    selectable: None,
                    deletable: None,
                    reconnectable: None,
                },
            );

            prev_out = Some(port_out);
        }

        graph
    }

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: stress a large node-graph canvas with viewport-driven culling (candidate for prepaint-windowed cull windows)."),
                cx.text("Use scripted middle-drag + wheel steps to validate correctness and collect perf bundles."),
            ]
        },
    );

    #[derive(Default)]
    struct HarnessState {
        graph: Option<Model<Graph>>,
        view: Option<Model<NodeGraphViewState>>,
    }

    let existing = cx.with_state(HarnessState::default, |st| {
        match (st.graph.clone(), st.view.clone()) {
            (Some(graph), Some(view)) => Some((graph, view)),
            _ => None,
        }
    });

    let (graph, view) = if let Some((graph, view)) = existing {
        (graph, view)
    } else {
        let graph_id = GraphId::from_u128(1);
        let graph = build_stress_graph(graph_id, 8_000);
        let graph = cx.app.models_mut().insert(graph);
        let view = cx.app.models_mut().insert(NodeGraphViewState::default());

        cx.with_state(HarnessState::default, |st| {
            st.graph = Some(graph.clone());
            st.view = Some(view.clone());
        });

        (graph, view)
    };

    let surface =
        cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
            let graph = graph.clone();
            let view = view.clone();

            let mut layout = LayoutStyle::default();
            layout.size.width = Length::Fill;
            layout.size.height = Length::Px(Px(520.0));

            let props = RetainedSubtreeProps::new::<App>(move |ui| {
                use fret_ui::retained_bridge::UiTreeRetainedExt as _;
                let canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
                ui.create_node_retained(canvas)
            })
            .with_layout(layout);

            let subtree = cx.retained_subtree(props);
            vec![cx.semantics(
                SemanticsProps {
                    role: SemanticsRole::Group,
                    test_id: Some(Arc::<str>::from("ui-gallery-node-graph-cull-root")),
                    ..Default::default()
                },
                |_cx| vec![subtree],
            )]
        });

    vec![header, surface]
}

pub(in crate::ui) fn preview_chrome_torture(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
    popover_open: Model<bool>,
    dialog_open: Model<bool>,
    alert_dialog_open: Model<bool>,
    sheet_open: Model<bool>,
    portal_geometry_popover_open: Model<bool>,
    dropdown_open: Model<bool>,
    context_menu_open: Model<bool>,
    context_menu_edge_open: Model<bool>,
    last_action: Model<Arc<str>>,
    text_input: Model<String>,
    text_area: Model<String>,
    checkbox: Model<bool>,
    switch: Model<bool>,
) -> Vec<AnyElement> {
    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: exercise hover/focus/pressed chrome under view-cache + shell."),
                cx.text(
                    "This page intentionally mixes many focusable widgets and overlay triggers.",
                ),
            ]
        },
    );

    let body = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N4),
        |cx| {
            let mut out = Vec::new();

            out.extend(preview_overlay(
                cx,
                popover_open,
                dialog_open,
                alert_dialog_open,
                sheet_open,
                portal_geometry_popover_open,
                dropdown_open,
                context_menu_open,
                context_menu_edge_open,
                last_action,
            ));

            let controls = stack::vstack(
                cx,
                stack::VStackProps::default()
                    .layout(LayoutRefinement::default().w_full())
                    .gap(Space::N3),
                |cx| {
                    let mut out: Vec<AnyElement> = Vec::new();

                    let row = stack::hstack(
                        cx,
                        stack::HStackProps::default().gap(Space::N2).items_center(),
                        |cx| {
                            vec![
                                shadcn::Button::new("One")
                                    .test_id("ui-gallery-chrome-btn-1")
                                    .into_element(cx),
                                shadcn::Button::new("Two")
                                    .variant(shadcn::ButtonVariant::Secondary)
                                    .test_id("ui-gallery-chrome-btn-2")
                                    .into_element(cx),
                                shadcn::Button::new("Three")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .test_id("ui-gallery-chrome-btn-3")
                                    .into_element(cx),
                                shadcn::Button::new("Disabled")
                                    .disabled(true)
                                    .test_id("ui-gallery-chrome-btn-disabled")
                                    .into_element(cx),
                            ]
                        },
                    );
                    out.push(row);

                    let fields = stack::hstack(
                        cx,
                        stack::HStackProps::default().gap(Space::N2).items_start(),
                        |cx| {
                            vec![
                                stack::vstack(
                                    cx,
                                    stack::VStackProps::default().gap(Space::N1),
                                    |cx| {
                                        let input = shadcn::Input::new(text_input.clone())
                                            .a11y_label("Chrome torture input")
                                            .placeholder("Type")
                                            .into_element(cx);
                                        let input = input.attach_semantics(
                                            SemanticsDecoration::default()
                                                .role(fret_core::SemanticsRole::TextField)
                                                .test_id("ui-gallery-chrome-text-input"),
                                        );
                                        vec![cx.text("Text input"), input]
                                    },
                                ),
                                stack::vstack(
                                    cx,
                                    stack::VStackProps::default().gap(Space::N1),
                                    |cx| {
                                        let textarea = shadcn::Textarea::new(text_area.clone())
                                            .a11y_label("Chrome torture textarea")
                                            .into_element(cx);
                                        let textarea = textarea.attach_semantics(
                                            SemanticsDecoration::default()
                                                .role(fret_core::SemanticsRole::TextField)
                                                .test_id("ui-gallery-chrome-text-area"),
                                        );
                                        vec![cx.text("Text area"), textarea]
                                    },
                                ),
                            ]
                        },
                    );
                    out.push(fields);

                    let toggles = stack::hstack(
                        cx,
                        stack::HStackProps::default().gap(Space::N3).items_center(),
                        |cx| {
                            vec![
                                shadcn::Checkbox::new(checkbox.clone())
                                    .a11y_label("Chrome torture checkbox")
                                    .test_id("ui-gallery-chrome-checkbox")
                                    .into_element(cx),
                                shadcn::Switch::new(switch.clone())
                                    .a11y_label("Chrome torture switch")
                                    .test_id("ui-gallery-chrome-switch")
                                    .into_element(cx),
                            ]
                        },
                    );
                    out.push(toggles);

                    out
                },
            );
            out.push(controls);

            out
        },
    );

    let content = body.attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Group)
            .test_id("ui-gallery-chrome-torture-root"),
    );

    vec![header, content]
}

pub(in crate::ui) fn preview_windowed_rows_surface_torture(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    use fret_core::{
        Corners, DrawOrder, Edges, FontId, SemanticsRole, TextOverflow, TextStyle, TextWrap,
    };
    use fret_ui::canvas::CanvasTextConstraints;
    use fret_ui_kit::declarative::windowed_rows_surface::{
        WindowedRowsSurfaceProps, windowed_rows_surface,
    };

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: baseline scroll windowing via a stable element tree (Scroll + Canvas)."),
                cx.text("This is the 'single-node surface' escape hatch: paint only visible rows, avoid per-row subtrees."),
            ]
        },
    );

    let len = 200_000usize;
    let row_h = Px(22.0);
    let overscan = 16usize;

    let scroll_handle = cx.with_state(fret_ui::scroll::ScrollHandle::default, |h| h.clone());

    let surface =
        cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
            let bg_even = theme.color_required("background");
            let bg_odd = theme.color_required("muted");
            let fg = theme.color_required("foreground");

            let text_style = TextStyle {
                font: FontId::monospace(),
                size: Px(12.0),
                ..Default::default()
            };

            let mut props = WindowedRowsSurfaceProps::default();
            props.scroll.layout.size.width = fret_ui::element::Length::Fill;
            props.scroll.layout.size.height = fret_ui::element::Length::Px(Px(420.0));
            props.scroll.layout.overflow = fret_ui::element::Overflow::Clip;
            props.len = len;
            props.row_height = row_h;
            props.overscan = overscan;
            props.scroll_handle = scroll_handle.clone();
            props.canvas.cache_policy = fret_ui::element::CanvasCachePolicy::smooth_default();

            let surface = windowed_rows_surface(cx, props, move |painter, index, rect| {
                let background = if (index % 2) == 0 { bg_even } else { bg_odd };
                painter.scene().push(fret_core::SceneOp::Quad {
                    order: DrawOrder(0),
                    rect,
                    background: fret_core::Paint::Solid(background),
                    border: Edges::all(Px(0.0)),
                    border_paint: fret_core::Paint::TRANSPARENT,

                    corner_radii: Corners::all(Px(0.0)),
                });

                let label = format!("Row {index}");
                let origin =
                    fret_core::Point::new(Px(rect.origin.x.0 + 8.0), Px(rect.origin.y.0 + 4.0));
                let scope = painter.key_scope(&"ui-gallery-windowed-rows");
                let key: u64 = painter.child_key(scope, &index).into();
                let _ = painter.text(
                    key,
                    DrawOrder(1),
                    origin,
                    label,
                    text_style.clone(),
                    fg,
                    CanvasTextConstraints {
                        max_width: Some(Px(rect.size.width.0.max(0.0) - 16.0)),
                        wrap: TextWrap::None,
                        overflow: TextOverflow::Clip,
                    },
                    painter.scale_factor(),
                );
            });

            vec![
                surface.attach_semantics(
                    SemanticsDecoration::default()
                        .role(SemanticsRole::Group)
                        .test_id("ui-gallery-windowed-rows-root"),
                ),
            ]
        });

    vec![header, surface]
}

pub(in crate::ui) fn preview_windowed_rows_surface_interactive_torture(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    use std::cell::RefCell;
    use std::rc::Rc;

    use fret_core::{Corners, CursorIcon, DrawOrder, Edges, FontId, SemanticsRole, TextStyle};
    use fret_ui::Invalidation;
    use fret_ui::action::{ActionCx, PointerDownCx, PointerMoveCx};
    use fret_ui::canvas::CanvasTextConstraints;
    use fret_ui::element::{Length, PointerRegionProps, SemanticsProps};
    use fret_ui_kit::declarative::windowed_rows_surface::{
        WindowedRowsSurfacePointerHandlers, WindowedRowsSurfaceProps,
        windowed_rows_surface_with_pointer_region,
    };

    #[derive(Default)]
    struct RowChromeState {
        hovered: Option<usize>,
        selected: Option<usize>,
    }

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: demonstrate paint-only hover/selection chrome on a prepaint-windowed row surface (ADR 0175 + ADR 0166)."),
                cx.text("Pattern: stable tree (Scroll + PointerRegion + Canvas), row hit-testing in pointer hooks, paint-only visuals in Canvas."),
            ]
        },
    );

    let len = 200_000usize;
    let row_h = Px(22.0);
    let overscan = 16usize;

    let scroll_handle = cx.with_state(fret_ui::scroll::ScrollHandle::default, |h| h.clone());

    let surface =
        cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
            let bg_even = theme.color_required("background");
            let bg_odd = theme.color_required("muted");
            let bg_hover = theme.color_required("accent");
            let fg = theme.color_required("foreground");

            let text_style = TextStyle {
                font: FontId::monospace(),
                size: Px(12.0),
                ..Default::default()
            };

            let root = cx.semantics_with_id(
                SemanticsProps {
                    role: SemanticsRole::Group,
                    test_id: Some(Arc::<str>::from("ui-gallery-windowed-rows-interactive-root")),
                    ..Default::default()
                },
                move |cx, root_id| {
                    let state = cx.with_state_for(
                        root_id,
                        || Rc::new(RefCell::new(RowChromeState::default())),
                        |s| s.clone(),
                    );

                    let on_move_state = state.clone();
                    let on_pointer_move: fret_ui_kit::declarative::windowed_rows_surface::OnWindowedRowsPointerMove =
                        Arc::new(move |host, action_cx: ActionCx, idx, _mv: PointerMoveCx| {
                            host.set_cursor_icon(CursorIcon::Pointer);
                            let mut st = on_move_state.borrow_mut();
                            if st.hovered == idx {
                                return true;
                            }
                            st.hovered = idx;
                            host.invalidate(Invalidation::Paint);
                            host.request_redraw(action_cx.window);
                            true
                        });

                    let on_down_state = state.clone();
                    let on_pointer_down: fret_ui_kit::declarative::windowed_rows_surface::OnWindowedRowsPointerDown =
                        Arc::new(move |host, action_cx: ActionCx, idx, down: PointerDownCx| {
                            if down.button != fret_core::MouseButton::Left {
                                return false;
                            }
                            let mut st = on_down_state.borrow_mut();
                            st.selected = Some(idx);
                            st.hovered = Some(idx);
                            host.invalidate(Invalidation::Paint);
                            host.request_redraw(action_cx.window);
                            true
                        });

                    let handlers = WindowedRowsSurfacePointerHandlers {
                        on_pointer_down: Some(on_pointer_down),
                        on_pointer_move: Some(on_pointer_move),
                        ..Default::default()
                    };

                    let mut props = WindowedRowsSurfaceProps::default();
                    props.scroll.layout.size.width = Length::Fill;
                    props.scroll.layout.size.height = Length::Px(Px(420.0));
                    props.scroll.layout.overflow = fret_ui::element::Overflow::Clip;
                    props.len = len;
                    props.row_height = row_h;
                    props.overscan = overscan;
                    props.scroll_handle = scroll_handle.clone();
                    props.canvas.cache_policy = fret_ui::element::CanvasCachePolicy::smooth_default();

                    let mut pointer = PointerRegionProps::default();
                    pointer.layout.size.width = Length::Fill;
                    pointer.layout.size.height = Length::Fill;

                    let paint_state = state.clone();
                    let content_semantics = SemanticsProps {
                        role: SemanticsRole::Group,
                        test_id: Some(Arc::<str>::from(
                            "ui-gallery-windowed-rows-interactive-canvas",
                        )),
                        ..Default::default()
                    };

                    vec![windowed_rows_surface_with_pointer_region(
                        cx,
                        props,
                        pointer,
                        handlers,
                        Some(content_semantics),
                        move |painter, index, rect| {
                            let st = paint_state.borrow();
                            let hovered = st.hovered == Some(index);
                            let selected = st.selected == Some(index);

                            let background = if hovered || selected {
                                bg_hover
                            } else if (index % 2) == 0 {
                                bg_even
                            } else {
                                bg_odd
                            };

                            painter.scene().push(fret_core::SceneOp::Quad {
                                order: DrawOrder(0),
                                rect,
                                background: fret_core::Paint::Solid(background),
                                border: if selected {
                                    Edges::all(Px(1.0))
                                } else {
                                    Edges::all(Px(0.0))
                                },
                                border_paint: fret_core::Paint::Solid(if selected {
                                    fg
                                } else {
                                    fret_core::Color::TRANSPARENT
                                }),
                                corner_radii: Corners::all(Px(0.0)),
                            });

                            let label = format!("Row {index}");
                            let origin = fret_core::Point::new(
                                Px(rect.origin.x.0 + 8.0),
                                Px(rect.origin.y.0 + 4.0),
                            );
                            let scope = painter.key_scope(&"ui-gallery-windowed-rows-interactive");
                            let key: u64 = painter.child_key(scope, &index).into();
                            let _ = painter.text(
                                key,
                                DrawOrder(1),
                                origin,
                                label,
                                text_style.clone(),
                                fg,
                                CanvasTextConstraints {
                                    max_width: Some(Px(rect.size.width.0.max(0.0) - 16.0)),
                                    wrap: fret_core::TextWrap::None,
                                    overflow: fret_core::TextOverflow::Clip,
                                },
                                painter.scale_factor(),
                            );
                        },
                    )]
                },
            );

            vec![root]
        });

    vec![header, surface]
}
