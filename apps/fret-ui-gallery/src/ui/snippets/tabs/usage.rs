pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_kit::declarative::text as decl_text;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::tabs_uncontrolled(cx, Some("account"), |cx| {
        [
            shadcn::TabsItem::new(
                "account",
                "Account",
                ui::children![cx; decl_text::text_paragraph(cx, "Make changes to your account here.")],
            ),
            shadcn::TabsItem::new(
                "password",
                "Password",
                ui::children![cx; decl_text::text_paragraph(cx, "Change your password here.")],
            ),
        ]
    })
    .refine_layout(LayoutRefinement::default().w_px(Px(400.0)).min_w_0())
    .into_element(cx)
}
// endregion: example
