pub const SOURCE: &str = include_str!("extras.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let last_commit = cx.local_model_keyed("ui-gallery-slider-extras-last-commit", Vec::<f32>::new);
    let max_width_sm = LayoutRefinement::default().w_full().max_w(Px(320.0));

    let on_value_commit = {
        let last_commit_for_cb = last_commit.clone();
        let slider = shadcn::Slider::new_controllable(cx, None, || vec![75.0])
            .range(0.0, 100.0)
            .test_id_prefix("ui-gallery-slider-on-value-commit")
            .a11y_label("Slider")
            .refine_layout(max_width_sm.clone())
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
            .layout(max_width_sm.clone())
            .into_element(cx)
    };

    let inverted = shadcn::Slider::new_controllable(cx, None, || vec![25.0])
        .range(0.0, 100.0)
        .step(1.0)
        .inverted(true)
        .test_id_prefix("ui-gallery-slider-inverted")
        .a11y_label("Inverted slider")
        .refine_layout(max_width_sm)
        .into_element(cx);

    ui::v_flex(|cx| {
        vec![
            shadcn::raw::typography::muted(
                "Fret follow-ups: `on_value_commit(...)` for commit-only side effects and `inverted(true)` for mirrored value progression.",
            )
            .into_element(cx),
            on_value_commit,
            inverted,
        ]
    })
    .gap(Space::N4)
    .items_start()
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
    .test_id("ui-gallery-slider-extras")
}
// endregion: example
