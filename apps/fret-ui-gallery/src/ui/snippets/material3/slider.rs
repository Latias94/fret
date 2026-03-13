pub const SOURCE: &str = include_str!("slider.rs");

// region: example
use fret_ui_material3 as material3;
use fret_ui_shadcn::prelude::*;

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let slider = material3::Slider::uncontrolled(cx, 0.3);
    let value = slider.value_model();
    let value_now = cx
        .get_model_copied(&value, Invalidation::Layout)
        .unwrap_or(0.0)
        .clamp(0.0, 1.0);
    let value_for_ticks_row = value.clone();

    ui::v_flex(move |cx| {
        vec![
            cx.text("Material 3 Slider: token-driven track + handle + state layer."),
            ui::h_row(move |cx| {
                vec![
                    slider
                        .clone()
                        .a11y_label("Material 3 Slider")
                        .test_id("ui-gallery-material3-slider")
                        .into_element(cx),
                    cx.text(format!("value={value_now:.3}"))
                        .test_id("ui-gallery-material3-slider-value"),
                    material3::Slider::new(value.clone())
                        .disabled(true)
                        .a11y_label("Disabled Material 3 Slider")
                        .test_id("ui-gallery-material3-slider-disabled")
                        .into_element(cx),
                ]
            })
            .gap(Space::N4)
            .items_center()
            .into_element(cx),
            ui::h_row(move |cx| {
                vec![
                    material3::Slider::new(value_for_ticks_row.clone())
                        .with_tick_marks(true)
                        .tick_marks_count(5)
                        .a11y_label("Material 3 Slider (tick marks)")
                        .test_id("ui-gallery-material3-slider-tick-marks")
                        .into_element(cx),
                    cx.text("tick_marks=5"),
                ]
            })
            .gap(Space::N4)
            .items_center()
            .into_element(cx),
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N3)
    .items_start()
    .into_element(cx)
    .into()
}

// endregion: example
