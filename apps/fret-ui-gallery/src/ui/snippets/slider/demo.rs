pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn controlled<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    controlled_values: Model<Vec<f32>>,
) -> AnyElement {
    let values_snapshot: Vec<f32> = cx
        .get_model_cloned(&controlled_values, Invalidation::Paint)
        .unwrap_or_default();
    let values_text = values_snapshot
        .iter()
        .map(|v| format!("{v:.1}"))
        .collect::<Vec<_>>()
        .join(", ");

    let header = ui::h_flex(|cx| {
        vec![
            shadcn::Label::new("Temperature").into_element(cx),
            shadcn::raw::typography::muted(cx, values_text),
        ]
    })
    .layout(LayoutRefinement::default().w_full())
    .items_center()
    .justify_between()
    .into_element(cx);
    let slider = shadcn::Slider::new(controlled_values)
        .range(0.0, 1.0)
        .step(0.1)
        .test_id("ui-gallery-slider-controlled")
        .a11y_label("Temperature")
        .into_element(cx);

    ui::v_flex(|_cx| vec![header, slider])
        .gap(Space::N3)
        .layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
        .into_element(cx)
}

pub fn render<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    controlled_values: Model<Vec<f32>>,
) -> AnyElement {
    let max_w_sm = LayoutRefinement::default().w_full().max_w(Px(384.0));

    let single = shadcn::Slider::new_controllable(cx, None, || vec![50.0])
        .range(0.0, 100.0)
        .step(1.0)
        .test_id("ui-gallery-slider-demo-single")
        .a11y_label("Slider")
        .refine_layout(max_w_sm.clone())
        .into_element(cx);

    let range = shadcn::Slider::new_controllable(cx, None, || vec![25.0, 50.0])
        .range(0.0, 100.0)
        .step(1.0)
        .test_id("ui-gallery-slider-demo-range")
        .a11y_label("Range slider")
        .refine_layout(max_w_sm.clone())
        .into_element(cx);

    let multiple = shadcn::Slider::new_controllable(cx, None, || vec![10.0, 20.0])
        .range(0.0, 100.0)
        .step(10.0)
        .test_id("ui-gallery-slider-demo-multiple")
        .a11y_label("Multiple thumbs slider")
        .refine_layout(max_w_sm.clone())
        .into_element(cx);

    let vertical = {
        let a = shadcn::Slider::new_controllable(cx, None, || vec![50.0])
            .range(0.0, 100.0)
            .step(1.0)
            .orientation(fret_ui_kit::primitives::slider::SliderOrientation::Vertical)
            .refine_layout(LayoutRefinement::default().h_px(Px(160.0)))
            .test_id("ui-gallery-slider-demo-vertical-a")
            .a11y_label("Vertical slider")
            .into_element(cx);

        let b = shadcn::Slider::new_controllable(cx, None, || vec![25.0])
            .range(0.0, 100.0)
            .step(1.0)
            .orientation(fret_ui_kit::primitives::slider::SliderOrientation::Vertical)
            .refine_layout(LayoutRefinement::default().h_px(Px(160.0)))
            .test_id("ui-gallery-slider-demo-vertical-b")
            .a11y_label("Vertical slider")
            .into_element(cx);

        ui::h_flex(|_cx| vec![a, b])
            .gap(Space::N6)
            .items_center()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .into_element(cx)
            .test_id("ui-gallery-slider-demo-vertical")
    };

    let controlled = controlled(cx, controlled_values).test_id("ui-gallery-slider-demo-controlled");

    fret_ui_kit::ui::h_flex(|_cx| vec![single, range, multiple, vertical, controlled])
        .gap(Space::N6)
        .wrap()
        .w_full()
        .items_start()
        .into_element(cx)
        .test_id("ui-gallery-slider-demo")
}
// endregion: example
