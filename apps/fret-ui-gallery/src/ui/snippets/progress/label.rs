pub const SOURCE: &str = include_str!("label.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn centered<B>(body: B) -> impl UiChild + use<B>
where
    B: UiChild,
{
    ui::h_flex(move |cx| [body.into_element(cx)])
        .layout(LayoutRefinement::default().w_full())
        .justify_center()
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    cx.keyed("ui_gallery.progress.label", |cx| {
        let label_row = ui::h_flex(|cx| {
            vec![
                shadcn::FieldLabel::new("Upload progress")
                    .test_id("ui-gallery-progress-label-title")
                    .into_element(cx),
                shadcn::FieldLabel::new("66%")
                    .refine_layout(LayoutRefinement::default().ml_auto())
                    .test_id("ui-gallery-progress-label-value")
                    .into_element(cx),
            ]
        })
        .layout(LayoutRefinement::default().w_full())
        .items_center()
        .test_id("ui-gallery-progress-label-row")
        .into_element(cx);

        let field = shadcn::Field::new(vec![
            label_row,
            shadcn::Progress::from_value(66.0)
                .a11y_label("Upload progress")
                .into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(384.0)))
        .into_element(cx);

        centered(field).test_id("ui-gallery-progress-label")
    })
}

// endregion: example
