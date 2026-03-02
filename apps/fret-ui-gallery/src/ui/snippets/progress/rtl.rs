// region: example
use crate::ui::doc_layout;
use fret_app::App;
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default, Clone)]
struct ProgressModels {
    value: Option<Model<f32>>,
}

fn ensure_value(cx: &mut ElementContext<'_, App>) -> Model<f32> {
    let state = cx.with_state(ProgressModels::default, |st| st.clone());
    match state.value {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(66.0);
            cx.with_state(ProgressModels::default, |st| st.value = Some(model.clone()));
            model
        }
    }
}

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let value = ensure_value(cx);

    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    cx.keyed("ui_gallery.progress.rtl", |cx| {
        doc_layout::rtl(cx, |cx| {
            let label_row = stack::hstack(
                cx,
                stack::HStackProps::default()
                    .layout(LayoutRefinement::default().w_full())
                    .items_center(),
                |cx| {
                    vec![
                        shadcn::FieldLabel::new("٦٦%").into_element(cx),
                        shadcn::FieldLabel::new("تقدم الرفع")
                            .refine_layout(LayoutRefinement::default().ml_auto())
                            .into_element(cx),
                    ]
                },
            );

            let field = shadcn::Field::new(vec![
                label_row,
                shadcn::Progress::new(value.clone())
                    .mirror_in_rtl(true)
                    .into_element(cx),
            ])
            .refine_layout(LayoutRefinement::default().w_full().max_w(Px(384.0)))
            .into_element(cx);

            centered(cx, field)
        })
        .test_id("ui-gallery-progress-rtl")
    })
}

// endregion: example
