pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn wrap_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    children: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    fret_ui_kit::ui::h_flex(children)
        .gap(Space::N2)
        .wrap()
        .w_full()
        .items_center()
        .into_element(cx)
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    wrap_row(cx, |cx| {
        vec![with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
            ui::h_flex(|cx| {
                vec![
                    shadcn::Button::new("زر")
                        .variant(shadcn::ButtonVariant::Outline)
                        .test_id("ui-gallery-button-rtl-default")
                        .into_element(cx),
                    shadcn::Button::new("حذف")
                        .variant(shadcn::ButtonVariant::Destructive)
                        .test_id("ui-gallery-button-rtl-destructive")
                        .into_element(cx),
                    shadcn::Button::new("")
                        .variant(shadcn::ButtonVariant::Outline)
                        .children([
                            ui::text("إرسال").font_medium().nowrap().into_element(cx),
                            fret_ui_shadcn::icon::icon_with(
                                cx,
                                IconId::new_static("lucide.arrow-right"),
                                None,
                                None,
                            ),
                        ])
                        .test_id("ui-gallery-button-rtl-submit")
                        .into_element(cx),
                    shadcn::Button::new("")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Icon)
                        .a11y_label("إضافة")
                        .icon(IconId::new_static("lucide.plus"))
                        .test_id("ui-gallery-button-rtl-add")
                        .into_element(cx),
                    shadcn::Button::new("")
                        .variant(shadcn::ButtonVariant::Secondary)
                        .disabled(true)
                        .test_id("ui-gallery-button-rtl-loading")
                        .children([
                            shadcn::Spinner::new().into_element(cx),
                            ui::text("جاري التحميل")
                                .font_medium()
                                .nowrap()
                                .into_element(cx),
                        ])
                        .into_element(cx),
                ]
            })
            .gap(Space::N2)
            .wrap()
            .items_center()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .into_element(cx)
            .test_id("ui-gallery-button-rtl-row-inner")
        })]
    })
    .test_id("ui-gallery-button-rtl-row")
}
// endregion: example
