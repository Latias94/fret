pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Default, Clone)]
struct Models {
    value: Option<Model<Option<Arc<str>>>>,
}

fn value_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<Option<Arc<str>>> {
    let state = cx.with_state(Models::default, |st| st.clone());
    match state.value {
        Some(model) => model,
        None => {
            let model = cx
                .app
                .models_mut()
                .insert(Some(Arc::<str>::from("account")));
            cx.with_state(Models::default, |st| st.value = Some(model.clone()));
            model
        }
    }
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let value = value_model(cx);

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
