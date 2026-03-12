pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let value = cx.local_model_keyed("value", || Some(Arc::<str>::from("account")));

    let list = shadcn::TabsList::new()
        .trigger(shadcn::TabsTrigger::new("account", "Account"))
        .trigger(shadcn::TabsTrigger::new("password", "Password"));

    shadcn::TabsRoot::new(value)
        .refine_layout(LayoutRefinement::default().w_px(Px(400.0)).min_w_0())
        .list(list)
        .contents([
            shadcn::TabsContent::new(
                "account",
                [ui::text("Make changes to your account here.").into_element(cx)],
            ),
            shadcn::TabsContent::new(
                "password",
                [ui::text("Change your password here.").into_element(cx)],
            ),
        ])
        .into_element(cx)
}
// endregion: example
