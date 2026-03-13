pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::tabs_uncontrolled(cx, Some("account"), |cx| {
        [
            shadcn::TabsItem::new(
                "account",
                "Account",
                ui::children![cx; ui::text("Make changes to your account here.")],
            ),
            shadcn::TabsItem::new(
                "password",
                "Password",
                ui::children![cx; ui::text("Change your password here.")],
            ),
        ]
    })
    .refine_layout(LayoutRefinement::default().w_px(Px(400.0)).min_w_0())
    .into_element(cx)
}
// endregion: example
