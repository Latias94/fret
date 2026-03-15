pub const SOURCE: &str = include_str!("radio.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_runtime::CommandId;
use fret_ui_kit::declarative::ModelWatchExt as _;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Default, Clone)]
struct MenubarRadioState {
    profile: Option<Arc<str>>,
    theme: Option<Arc<str>>,
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let width = LayoutRefinement::default().w_px(Px(288.0)).min_w_0();
    let state = cx.local_model(|| MenubarRadioState {
        profile: Some(Arc::<str>::from("benoit")),
        theme: Some(Arc::<str>::from("system")),
    });
    let state_now = cx.watch_model(&state).layout().cloned().unwrap_or_default();

    let profiles = shadcn::MenubarMenu::new("Profiles").entries([
        shadcn::MenubarEntry::RadioGroup(
            shadcn::MenubarRadioGroup::from_value(state_now.profile.clone())
                .on_value_change({
                    let state = state.clone();
                    move |host, _action_cx, value| {
                        let _ = host
                            .models_mut()
                            .update(&state, |st| st.profile = Some(value));
                    }
                })
                .item(
                    shadcn::MenubarRadioItemSpec::new("andy", "Andy")
                        .action(CommandId::new("ui_gallery.menubar.radio.profile.andy")),
                )
                .item(
                    shadcn::MenubarRadioItemSpec::new("benoit", "Benoit")
                        .action(CommandId::new("ui_gallery.menubar.radio.profile.benoit")),
                )
                .item(
                    shadcn::MenubarRadioItemSpec::new("luis", "Luis")
                        .action(CommandId::new("ui_gallery.menubar.radio.profile.luis")),
                ),
        ),
        shadcn::MenubarEntry::Separator,
        shadcn::MenubarEntry::Item(
            shadcn::MenubarItem::new("Edit...")
                .inset(true)
                .action(CommandId::new("ui_gallery.menubar.radio.profile.edit")),
        ),
        shadcn::MenubarEntry::Item(
            shadcn::MenubarItem::new("Add Profile...")
                .inset(true)
                .action(CommandId::new("ui_gallery.menubar.radio.profile.add")),
        ),
    ]);

    let themes = shadcn::MenubarMenu::new("Theme").entries([shadcn::MenubarEntry::RadioGroup(
        shadcn::MenubarRadioGroup::from_value(state_now.theme.clone())
            .on_value_change({
                let state = state.clone();
                move |host, _action_cx, value| {
                    let _ = host
                        .models_mut()
                        .update(&state, |st| st.theme = Some(value));
                }
            })
            .item(
                shadcn::MenubarRadioItemSpec::new("light", "Light")
                    .action(CommandId::new("ui_gallery.menubar.radio.theme.light")),
            )
            .item(
                shadcn::MenubarRadioItemSpec::new("dark", "Dark")
                    .action(CommandId::new("ui_gallery.menubar.radio.theme.dark")),
            )
            .item(
                shadcn::MenubarRadioItemSpec::new("system", "System")
                    .action(CommandId::new("ui_gallery.menubar.radio.theme.system")),
            ),
    )]);

    shadcn::Menubar::new([profiles, themes])
        .refine_layout(width)
        .into_element(cx)
}
// endregion: example
