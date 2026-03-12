pub const SOURCE: &str = include_str!("radio.rs");

// region: example
use fret_core::Px;
use fret_runtime::CommandId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let width = LayoutRefinement::default().w_px(Px(288.0)).min_w_0();
    let profile = cx.local_model_keyed("profile", || Some(Arc::<str>::from("benoit")));
    let theme = cx.local_model_keyed("theme", || Some(Arc::<str>::from("system")));

    let profiles = shadcn::MenubarMenu::new("Profiles").entries([
        shadcn::MenubarEntry::RadioGroup(
            shadcn::MenubarRadioGroup::new(profile.clone())
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
        shadcn::MenubarRadioGroup::new(theme.clone())
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
