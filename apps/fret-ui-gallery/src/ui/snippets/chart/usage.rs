pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_kit::declarative::{CachedSubtreeExt as _, CachedSubtreeProps};
use fret_ui_kit::ui;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

fn sample_chart_canvas(cx: &mut UiCx<'_>, test_id: impl Into<Arc<str>>) -> AnyElement {
    use delinea::data::{Column, DataTable};
    use delinea::{
        AxisKind, AxisScale, CategoryAxisScale, ChartSpec, DatasetSpec, FieldSpec, GridSpec,
        SeriesEncode, SeriesKind, SeriesSpec,
    };
    use fret_chart::ChartCanvas;
    use fret_ui::element::{LayoutStyle, Length};
    use fret_ui::retained_bridge::RetainedSubtreeProps;

    let test_id: Arc<str> = test_id.into();

    cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
        use fret_ui::retained_bridge::UiTreeRetainedExt as _;

        let dataset_id = delinea::ids::DatasetId::new(1);
        let grid_id = delinea::ids::GridId::new(1);
        let x_axis = delinea::AxisId::new(1);
        let y_axis = delinea::AxisId::new(2);
        let x_field = delinea::FieldId::new(1);
        let desktop_field = delinea::FieldId::new(2);
        let mobile_field = delinea::FieldId::new(3);

        let categories = vec![
            "January".to_string(),
            "February".to_string(),
            "March".to_string(),
            "April".to_string(),
            "May".to_string(),
            "June".to_string(),
        ];
        let x = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
        let desktop = vec![186.0, 305.0, 237.0, 73.0, 209.0, 214.0];
        let mobile = vec![80.0, 200.0, 120.0, 190.0, 130.0, 140.0];

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
                        id: desktop_field,
                        column: 1,
                    },
                    FieldSpec {
                        id: mobile_field,
                        column: 2,
                    },
                ],
                ..Default::default()
            }],
            grids: vec![GridSpec { id: grid_id }],
            axes: vec![
                delinea::AxisSpec {
                    id: x_axis,
                    name: Some("Month".to_string()),
                    kind: AxisKind::X,
                    grid: grid_id,
                    position: None,
                    scale: AxisScale::Category(CategoryAxisScale { categories }),
                    range: Default::default(),
                },
                delinea::AxisSpec {
                    id: y_axis,
                    name: Some("Visitors".to_string()),
                    kind: AxisKind::Y,
                    grid: grid_id,
                    position: None,
                    scale: Default::default(),
                    range: Default::default(),
                },
            ],
            data_zoom_x: vec![],
            data_zoom_y: vec![],
            tooltip: None,
            axis_pointer: None,
            visual_maps: vec![],
            series: vec![
                SeriesSpec {
                    id: delinea::ids::SeriesId::new(1),
                    name: Some("Desktop".to_string()),
                    kind: SeriesKind::Bar,
                    dataset: dataset_id,
                    encode: SeriesEncode {
                        x: x_field,
                        y: desktop_field,
                        y2: None,
                    },
                    x_axis,
                    y_axis,
                    stack: None,
                    stack_strategy: Default::default(),
                    bar_layout: Default::default(),
                    area_baseline: None,
                    lod: None,
                },
                SeriesSpec {
                    id: delinea::ids::SeriesId::new(2),
                    name: Some("Mobile".to_string()),
                    kind: SeriesKind::Bar,
                    dataset: dataset_id,
                    encode: SeriesEncode {
                        x: x_field,
                        y: mobile_field,
                        y2: None,
                    },
                    x_axis,
                    y_axis,
                    stack: None,
                    stack_strategy: Default::default(),
                    bar_layout: Default::default(),
                    area_baseline: None,
                    lod: None,
                },
            ],
        };

        let mut layout = LayoutStyle::default();
        layout.size.width = Length::Fill;
        layout.size.height = Length::Px(Px(208.0));

        let canvas_test_id = test_id.to_string();
        let props = RetainedSubtreeProps::new::<fret::app::App>(move |ui| {
            let mut canvas = ChartCanvas::new(spec.clone()).expect("chart spec should be valid");
            canvas.set_accessibility_layer(true);
            canvas.set_input_map(fret_chart::input_map::ChartInputMap::default());
            canvas = canvas.test_id(canvas_test_id.clone());

            let mut table = DataTable::default();
            table.push_column(Column::F64(x.clone()));
            table.push_column(Column::F64(desktop.clone()));
            table.push_column(Column::F64(mobile.clone()));
            canvas.engine_mut().datasets_mut().insert(dataset_id, table);

            let node = ui.create_node_retained(canvas);
            ui.set_node_view_cache_flags(node, true, true, false);
            node
        })
        .with_layout(layout);

        vec![cx.retained_subtree(props)]
    })
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let config: shadcn::ChartConfig = [
        (
            Arc::<str>::from("desktop"),
            shadcn::ChartConfigItem::new()
                .label("Desktop")
                .color(ColorRef::Color(cx.theme().color_token("chart-1"))),
        ),
        (
            Arc::<str>::from("mobile"),
            shadcn::ChartConfigItem::new()
                .label("Mobile")
                .color(ColorRef::Color(cx.theme().color_token("chart-2"))),
        ),
    ]
    .into_iter()
    .collect();

    shadcn::chart_container(config, |cx| {
        let canvas = sample_chart_canvas(cx, "ui-gallery-chart-first-chart-canvas");
        let legend = shadcn::ChartLegend::new(shadcn::ChartLegendContent::new()).into_element(cx);
        let tooltip = shadcn::ChartTooltip::new(
            shadcn::ChartTooltipContent::new().label("January").items([
                shadcn::ChartTooltipItem::new("Desktop", "186")
                    .color(ColorRef::Color(cx.theme().color_token("chart-1"))),
                shadcn::ChartTooltipItem::new("Mobile", "80")
                    .color(ColorRef::Color(cx.theme().color_token("chart-2"))),
            ]),
        )
        .into_element(cx);

        ui::v_flex(|cx| {
            vec![
                canvas,
                legend,
                tooltip,
                shadcn::raw::typography::muted(
                    "Use `chart_container(config, |cx| ...)` for Fret's composable child surface. Legend defaults can come from ChartConfig; tooltip payload wiring is still app-owned today.",
                )
                .into_element(cx),
            ]
        })
        .gap(Space::N4)
        .items_start()
        .layout(LayoutRefinement::default().w_full().h_full())
        .into_element(cx)
    })
    .id("traffic")
    .test_id("ui-gallery-chart-first-chart")
    .refine_layout(
        LayoutRefinement::default()
            .w_full()
            .max_w(Px(560.0))
            .h_px(Px(360.0))
            .aspect_ratio(560.0 / 360.0),
    )
    .into_element(cx)
}
// endregion: example
