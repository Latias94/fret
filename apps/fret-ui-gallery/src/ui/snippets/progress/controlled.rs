pub const SOURCE: &str = include_str!("controlled.rs");

// region: example
use fret_app::App;
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

#[derive(Default, Clone)]
struct ProgressModels {
    values: Option<Model<Vec<f32>>>,
}

fn ensure_values(cx: &mut ElementContext<'_, App>) -> Model<Vec<f32>> {
    let state = cx.with_state(ProgressModels::default, |st| st.clone());
    match state.values {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(vec![50.0]);
            cx.with_state(ProgressModels::default, |st| {
                st.values = Some(model.clone())
            });
            model
        }
    }
}

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let values = ensure_values(cx);

    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        ui::h_flex(move |_cx| [body])
            .layout(LayoutRefinement::default().w_full())
            .justify_center()
            .into_element(cx)
    };

    cx.keyed("ui_gallery.progress.controlled", |cx| {
        let body = ui::v_flex(|cx| {
            vec![
                shadcn::Progress::new_values_first(values.clone())
                    .into_element(cx)
                    .test_id("ui-gallery-progress-controlled-bar"),
                shadcn::Slider::new(values.clone())
                    .range(0.0, 100.0)
                    .step(1.0)
                    .a11y_label("Progress value")
                    .into_element(cx)
                    .test_id("ui-gallery-progress-controlled-slider"),
            ]
        })
        .gap(Space::N4)
        .layout(LayoutRefinement::default().w_full().max_w(Px(384.0)))
        .into_element(cx);

        centered(cx, body).test_id("ui-gallery-progress-controlled")
    })
}

// endregion: example
