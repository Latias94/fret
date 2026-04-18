pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_icons::IconId;
use fret_ui_kit::ui;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        let icon = icon::icon(cx, IconId::new_static("lucide.folder-code"));
        let actions = ui::h_row(|cx| {
            vec![
                shadcn::Button::new("إنشاء مشروع")
                    .test_id("ui-gallery-empty-rtl-create-project")
                    .into_element(cx),
                shadcn::Button::new("استيراد مشروع")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-empty-rtl-import-project")
                    .into_element(cx),
            ]
        })
        .gap(Space::N2)
        .items_center()
        .into_element(cx)
        .test_id("ui-gallery-empty-rtl-actions");
        let learn_more = shadcn::Button::new("تعرف على المزيد")
            .variant(shadcn::ButtonVariant::Link)
            .size(shadcn::ButtonSize::Sm)
            .render(shadcn::ButtonRender::Link {
                href: Arc::from("https://example.com/projects"),
                target: None,
                rel: None,
            })
            .trailing_icon(IconId::new_static("lucide.arrow-up-right"))
            .on_activate(Arc::new(|_host, _acx, _reason| {}))
            .test_id("ui-gallery-empty-rtl-learn-more")
            .into_element(cx);

        shadcn::empty(|cx| {
            ui::children![
                cx;
                shadcn::empty_header(|cx| {
                    ui::children![
                        cx;
                        shadcn::empty_media(|cx| ui::children![cx; icon])
                            .variant(shadcn::EmptyMediaVariant::Icon),
                        shadcn::empty_title("لا توجد مشاريع بعد"),
                        shadcn::empty_description(
                            "لم تقم بإنشاء أي مشاريع بعد. ابدأ بإنشاء مشروعك الأول.",
                        ),
                    ]
                }),
                shadcn::empty_content(|cx| ui::children![cx; actions]),
                learn_more,
            ]
        })
        .refine_layout(LayoutRefinement::default().w_full().min_h(Px(280.0)))
        .into_element(cx)
    })
    .test_id("ui-gallery-empty-rtl")
}
// endregion: example
