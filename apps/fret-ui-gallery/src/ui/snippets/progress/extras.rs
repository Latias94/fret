// region: example
use fret_app::App;
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default, Clone)]
struct ProgressModels {
    label_value: Option<Model<f32>>,
    controlled_values: Option<Model<Vec<f32>>>,
}

fn ensure_models(cx: &mut ElementContext<'_, App>) -> (Model<f32>, Model<Vec<f32>>) {
    let state = cx.with_state(ProgressModels::default, |st| st.clone());
    match (state.label_value, state.controlled_values) {
        (Some(label_value), Some(controlled_values)) => (label_value, controlled_values),
        _ => {
            let models = cx.app.models_mut();
            let label_value = models.insert(66.0);
            let controlled_values = models.insert(vec![50.0]);
            cx.with_state(ProgressModels::default, |st| {
                st.label_value = Some(label_value.clone());
                st.controlled_values = Some(controlled_values.clone());
            });
            (label_value, controlled_values)
        }
    }
}

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let (label_value, controlled_values) = ensure_models(cx);

    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    let label = cx.keyed("ui_gallery.progress.label", |cx| {
        let label_row = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .items_center(),
            |cx| {
                vec![
                    shadcn::FieldLabel::new("Upload progress").into_element(cx),
                    shadcn::FieldLabel::new("66%")
                        .refine_layout(LayoutRefinement::default().ml_auto())
                        .into_element(cx),
                ]
            },
        );

        let field = shadcn::Field::new(vec![
            label_row,
            shadcn::Progress::new(label_value.clone()).into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(384.0)))
        .into_element(cx);

        centered(cx, field).test_id("ui-gallery-progress-label")
    });

    let controlled = cx.keyed("ui_gallery.progress.controlled", |cx| {
        let values = controlled_values.clone();
        let body = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N4)
                .layout(LayoutRefinement::default().w_full().max_w(Px(384.0))),
            |cx| {
                vec![
                    shadcn::Progress::new_values_first(values.clone()).into_element(cx),
                    shadcn::Slider::new(values)
                        .range(0.0, 100.0)
                        .step(1.0)
                        .a11y_label("Progress value")
                        .into_element(cx),
                ]
            },
        );

        centered(cx, body).test_id("ui-gallery-progress-controlled")
    });

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N4)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        |cx| {
            vec![
                shadcn::typography::muted(
                    cx,
                    "Extras are Fret-specific recipes and regression gates (not part of upstream shadcn ProgressDemo).",
                ),
                label,
                controlled,
            ]
        },
    )
    .test_id("ui-gallery-progress-extras")
}

// endregion: example
