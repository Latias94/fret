pub const SOURCE: &str = include_str!("controlled.rs");

// region: example
use fret_app::App;
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

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
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    cx.keyed("ui_gallery.progress.controlled", |cx| {
        let body = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N4)
                .layout(LayoutRefinement::default().w_full().max_w(Px(384.0))),
            |cx| {
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
            },
        );

        centered(cx, body).test_id("ui-gallery-progress-controlled")
    })
}

// endregion: example
