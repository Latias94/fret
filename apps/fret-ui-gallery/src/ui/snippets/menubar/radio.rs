// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Default, Clone)]
struct Models {
    profile: Option<Model<Option<Arc<str>>>>,
    theme: Option<Model<Option<Arc<str>>>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let state = cx.with_state(Models::default, |st| st.clone());
    let width = LayoutRefinement::default().w_px(Px(288.0)).min_w_0();

    let profile = match state.profile {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(Some(Arc::<str>::from("benoit")));
            cx.with_state(Models::default, |st| st.profile = Some(model.clone()));
            model
        }
    };

    let theme = match state.theme {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(Some(Arc::<str>::from("system")));
            cx.with_state(Models::default, |st| st.theme = Some(model.clone()));
            model
        }
    };

    let profiles = shadcn::MenubarMenu::new("Profiles").entries([
        shadcn::MenubarEntry::RadioGroup(
            shadcn::MenubarRadioGroup::new(profile.clone())
                .item(shadcn::MenubarRadioItemSpec::new("andy", "Andy"))
                .item(shadcn::MenubarRadioItemSpec::new("benoit", "Benoit"))
                .item(shadcn::MenubarRadioItemSpec::new("luis", "Luis")),
        ),
        shadcn::MenubarEntry::Separator,
        shadcn::MenubarEntry::Item(shadcn::MenubarItem::new("Edit...").inset(true)),
        shadcn::MenubarEntry::Item(shadcn::MenubarItem::new("Add Profile...").inset(true)),
    ]);

    let themes = shadcn::MenubarMenu::new("Theme").entries([shadcn::MenubarEntry::RadioGroup(
        shadcn::MenubarRadioGroup::new(theme.clone())
            .item(shadcn::MenubarRadioItemSpec::new("light", "Light"))
            .item(shadcn::MenubarRadioItemSpec::new("dark", "Dark"))
            .item(shadcn::MenubarRadioItemSpec::new("system", "System")),
    )]);

    shadcn::Menubar::new([profiles, themes])
        .refine_layout(width)
        .into_element(cx)
}
// endregion: example

