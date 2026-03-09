pub const SOURCE: &str = include_str!("radio.rs");

// region: example
use fret_runtime::CommandId;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Default, Clone)]
struct Models {
    theme_mode: Option<Model<Option<Arc<str>>>>,
}

fn trigger_surface<H: UiHost>(cx: &mut ElementContext<'_, H>, label: &'static str) -> AnyElement {
    shadcn::Button::new(label)
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Sm)
        .into_element(cx)
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let state = cx.with_state(Models::default, |st| st.clone());

    let theme_mode = match state.theme_mode {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(Some(Arc::<str>::from("system")));
            cx.with_state(Models::default, |st| st.theme_mode = Some(model.clone()));
            model
        }
    };

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
                    shadcn::ContextMenuRadioGroup::new(theme_mode.clone())
                        .item(
                            shadcn::ContextMenuRadioItemSpec::new("system", "System")
                                .action(CommandId::new(
                                    "ui_gallery.context_menu.radio.theme.system",
                                )),
                        )
                        .item(
                            shadcn::ContextMenuRadioItemSpec::new("light", "Light")
                                .action(CommandId::new(
                                    "ui_gallery.context_menu.radio.theme.light",
                                )),
                        )
                        .item(
                            shadcn::ContextMenuRadioItemSpec::new("dark", "Dark").action(
                                CommandId::new("ui_gallery.context_menu.radio.theme.dark"),
                            ),
                        ),
                )]
            },
        )
        .test_id("ui-gallery-context-menu-radio")
}
// endregion: example
