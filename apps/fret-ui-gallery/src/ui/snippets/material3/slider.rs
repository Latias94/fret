pub const SOURCE: &str = include_str!("slider.rs");

// region: example
use fret_ui_material3 as material3;
use fret_ui_shadcn::prelude::*;

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>, value: Model<f32>) -> AnyElement {
    let value_now = cx
        .get_model_copied(&value, Invalidation::Layout)
        .unwrap_or(0.0)
        .clamp(0.0, 1.0);
    let value_for_main_row = value.clone();
    let value_for_ticks_row = value.clone();

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N3)
            .items_start(),
        move |cx| {
            vec![
                cx.text("Material 3 Slider: token-driven track + handle + state layer."),
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N4).items_center(),
                    move |cx| {
                        vec![
                            material3::Slider::new(value_for_main_row.clone())
                                .a11y_label("Material 3 Slider")
                                .test_id("ui-gallery-material3-slider")
                                .into_element(cx),
                            cx.text(format!("value={value_now:.3}"))
                                .test_id("ui-gallery-material3-slider-value"),
                            material3::Slider::new(value_for_main_row.clone())
                                .disabled(true)
                                .a11y_label("Disabled Material 3 Slider")
                                .test_id("ui-gallery-material3-slider-disabled")
                                .into_element(cx),
                        ]
                    },
                ),
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N4).items_center(),
                    move |cx| {
                        vec![
                            material3::Slider::new(value_for_ticks_row.clone())
                                .with_tick_marks(true)
                                .tick_marks_count(5)
                                .a11y_label("Material 3 Slider (tick marks)")
                                .test_id("ui-gallery-material3-slider-tick-marks")
                                .into_element(cx),
                            cx.text("tick_marks=5"),
                        ]
                    },
                ),
            ]
        },
    )
    .into()
}

// endregion: example
