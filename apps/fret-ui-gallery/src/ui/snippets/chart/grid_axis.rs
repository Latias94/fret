pub const SOURCE: &str = include_str!("grid_axis.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

#[allow(dead_code)]
pub fn build_chart_spec() -> delinea::ChartSpec {
    use delinea::{AxisKind, AxisScale, CategoryAxisScale, ChartSpec, GridSpec};

    let grid_id = delinea::GridId::new(1);
    let x_axis = delinea::AxisId::new(1);
    let y_axis = delinea::AxisId::new(2);

    ChartSpec {
        id: delinea::ChartId::new(1),
        grids: vec![GridSpec { id: grid_id }],
        axes: vec![
            delinea::AxisSpec {
                id: x_axis,
                name: Some("Month".to_string()),
                kind: AxisKind::X,
                grid: grid_id,
                position: None,
                scale: AxisScale::Category(CategoryAxisScale {
                    categories: vec![
                        "January".to_string(),
                        "February".to_string(),
                        "March".to_string(),
                        "April".to_string(),
                        "May".to_string(),
                        "June".to_string(),
                    ],
                }),
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
        axis_pointer: Some(delinea::AxisPointerSpec::default()),
        ..Default::default()
    }
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    ui::v_flex(|cx| {
        vec![
            shadcn::raw::typography::muted(
                "In Fret today, chart grid and axis setup lives in the retained `delinea::ChartSpec` instead of separate child widgets.",
            )
            .into_element(cx),
            shadcn::raw::typography::muted(
                "Define `GridSpec` and `AxisSpec` before you construct `fret_chart::ChartCanvas`, then keep tooltip and legend composition on the `ChartContainer` recipe surface.",
            )
            .into_element(cx),
        ]
    })
    .gap(Space::N2)
    .items_start()
    .layout(LayoutRefinement::default().w_full().max_w(Px(640.0)))
    .into_element(cx)
    .test_id("ui-gallery-chart-grid-axis-spec")
}
// endregion: example
