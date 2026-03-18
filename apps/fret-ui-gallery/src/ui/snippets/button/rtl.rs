pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn wrap_row<H: UiHost, F>(children: F) -> impl IntoUiElement<H> + use<H, F>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
{
    fret_ui_kit::ui::h_flex(children)
        .gap(Space::N2)
        .wrap()
        .w_full()
        .items_center()
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    wrap_row(|cx| {
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
                    shadcn::Button::new("إرسال")
                        .variant(shadcn::ButtonVariant::Outline)
                        .trailing_icon(IconId::new_static("lucide.arrow-left"))
                        .test_id("ui-gallery-button-rtl-submit")
                        .into_element(cx),
                    shadcn::Button::new("")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Icon)
                        .a11y_label("إضافة")
                        .icon(IconId::new_static("lucide.plus"))
                        .test_id("ui-gallery-button-rtl-add")
                        .into_element(cx),
                    shadcn::Button::new("جاري التحميل")
                        .variant(shadcn::ButtonVariant::Secondary)
                        .disabled(true)
                        .test_id("ui-gallery-button-rtl-loading")
                        .leading_child(shadcn::Spinner::new().into_element(cx))
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
    .into_element(cx)
    .test_id("ui-gallery-button-rtl-row")
}
// endregion: example
