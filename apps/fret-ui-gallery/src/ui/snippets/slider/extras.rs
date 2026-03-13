pub const SOURCE: &str = include_str!("extras.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let last_commit = cx.local_model_keyed("ui-gallery-slider-extras-last-commit", Vec::<f32>::new);
    let max_width_xs = LayoutRefinement::default().w_full().max_w(Px(320.0));

    let on_value_commit = {
        let last_commit_for_cb = last_commit.clone();
        let slider = shadcn::Slider::new_controllable(cx, None, || vec![75.0])
            .range(0.0, 100.0)
            .test_id("ui-gallery-slider-on-value-commit")
            .a11y_label("Slider")
            .refine_layout(max_width_xs.clone())
            .on_value_commit(move |host, _cx, values| {
                let _ = host.models_mut().update(&last_commit_for_cb, |v| {
                    *v = values;
                });
            })
            .into_element(cx);

        let last_commit_values: Vec<f32> = cx
            .get_model_cloned(&last_commit, Invalidation::Paint)
            .unwrap_or_default();
        let last_commit_text = if last_commit_values.is_empty() {
            "<none>".to_string()
        } else {
            format!("{last_commit_values:?}")
        };
        let meta = shadcn::raw::typography::muted(format!("onValueCommit: {last_commit_text}"))
            .into_element(cx);

        ui::v_flex(|_cx| vec![slider, meta])
            .gap(Space::N3)
            .layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
            .into_element(cx)
    };

    let disabled = shadcn::Slider::new_controllable(cx, None, || vec![50.0])
        .range(0.0, 100.0)
        .step(1.0)
        .disabled(true)
        .test_id("ui-gallery-slider-disabled")
        .a11y_label("Disabled slider")
        .refine_layout(max_width_xs.clone())
        .into_element(cx);

    let rtl = shadcn::Slider::new_controllable(cx, None, || vec![75.0])
        .range(0.0, 100.0)
        .step(1.0)
        .dir(LayoutDirection::Rtl)
        .test_id("ui-gallery-slider-rtl")
        .a11y_label("RTL slider")
        .refine_layout(max_width_xs.clone())
        .into_element(cx);

    let inverted = shadcn::Slider::new_controllable(cx, None, || vec![25.0])
        .range(0.0, 100.0)
        .step(1.0)
        .inverted(true)
        .test_id("ui-gallery-slider-inverted")
        .a11y_label("Inverted slider")
        .refine_layout(max_width_xs)
        .into_element(cx);

    ui::v_flex(|cx| {
            vec![
                shadcn::raw::typography::muted(
                    "Extras are Fret-specific demos and regression gates (not part of upstream shadcn SliderDemo).",
                ).into_element(cx),
                on_value_commit,
                disabled,
                rtl,
                inverted,
            ]
        })
            .gap(Space::N4)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()).into_element(cx)
    .test_id("ui-gallery-slider-extras")
}
// endregion: example
