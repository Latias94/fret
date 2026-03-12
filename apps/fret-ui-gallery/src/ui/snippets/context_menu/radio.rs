pub const SOURCE: &str = include_str!("radio.rs");

// region: example
use fret_runtime::CommandId;
use fret_ui_kit::declarative::ModelWatchExt as _;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

fn trigger_surface<H: UiHost>(cx: &mut ElementContext<'_, H>, label: &'static str) -> AnyElement {
    shadcn::Button::new(label)
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Sm)
        .into_element(cx)
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let theme_mode = cx.local_model(|| Some(Arc::<str>::from("system")));
    let theme_mode_now = cx.watch_model(&theme_mode).layout().cloned().flatten();

    shadcn::ContextMenu::new_controllable(cx, None, false)
        .content_test_id("ui-gallery-context-menu-radio-content")
        .into_element(
            cx,
            |cx| {
                trigger_surface(cx, "Right click for radio")
                    .test_id("ui-gallery-context-menu-radio-trigger")
            },
            |_cx| {
                vec![shadcn::ContextMenuEntry::RadioGroup(
                    shadcn::ContextMenuRadioGroup::from_value(theme_mode_now)
                        .on_value_change({
                            let theme_mode = theme_mode.clone();
                            move |host, _action_cx, value| {
                                let _ = host
                                    .models_mut()
                                    .update(&theme_mode, |selected| *selected = Some(value));
                            }
                        })
                        .item(
                            shadcn::ContextMenuRadioItemSpec::new("system", "System").action(
                                CommandId::new("ui_gallery.context_menu.radio.theme.system"),
                            ),
                        )
                        .item(
                            shadcn::ContextMenuRadioItemSpec::new("light", "Light").action(
                                CommandId::new("ui_gallery.context_menu.radio.theme.light"),
                            ),
                        )
                        .item(
                            shadcn::ContextMenuRadioItemSpec::new("dark", "Dark")
                                .action(CommandId::new("ui_gallery.context_menu.radio.theme.dark")),
                        ),
                )]
            },
        )
        .test_id("ui-gallery-context-menu-radio")
}
// endregion: example
