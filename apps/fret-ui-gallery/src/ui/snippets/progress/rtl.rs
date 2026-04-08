pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::{UiChild, UiCx};
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

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    cx.keyed("ui_gallery.progress.rtl", |cx| {
        shadcn::DirectionProvider::new(shadcn::LayoutDirection::Rtl)
            .into_element(cx, |cx| {
                let label_row = ui::h_flex(|cx| {
                    vec![
                        shadcn::FieldLabel::new("تقدم الرفع")
                            .refine_layout(LayoutRefinement::default().order(1))
                            .test_id("ui-gallery-progress-rtl-title")
                            .into_element(cx),
                        shadcn::FieldLabel::new("٦٦%")
                            .refine_layout(LayoutRefinement::default().order(0).mr_auto())
                            .test_id("ui-gallery-progress-rtl-value")
                            .into_element(cx),
                    ]
                })
                .layout(LayoutRefinement::default().w_full())
                .items_center()
                .test_id("ui-gallery-progress-rtl-row")
                .into_element(cx);

                let field = shadcn::Field::new(vec![
                    label_row,
                    shadcn::Progress::from_value(66.0)
                        .a11y_label("تقدم الرفع")
                        .mirror_in_rtl(true)
                        .into_element(cx),
                ])
                .refine_layout(LayoutRefinement::default().w_full().max_w(Px(384.0)))
                .into_element(cx);

                centered(field).into_element(cx)
            })
            .test_id("ui-gallery-progress-rtl")
    })
}

// endregion: example
