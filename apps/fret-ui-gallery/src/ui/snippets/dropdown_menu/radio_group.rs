pub const SOURCE: &str = include_str!("radio_group.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Default, Clone)]
struct Models {
    theme_mode: Option<Model<Option<Arc<str>>>>,
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

    shadcn::DropdownMenu::new_controllable(cx, None, false)
        .into_element_parts(
            cx,
            |cx| {
                shadcn::DropdownMenuTrigger::new(
                    shadcn::Button::new("Radio Group")
                        .variant(shadcn::ButtonVariant::Outline)
                        .test_id("ui-gallery-dropdown-menu-radio-group-trigger")
                        .into_element(cx),
                )
            },
            shadcn::DropdownMenuContent::new()
                .align(shadcn::DropdownMenuAlign::Start)
                .side_offset(Px(4.0)),
            |_cx| {
                [shadcn::DropdownMenuRadioGroup::new(theme_mode.clone())
                    .item(
                        shadcn::DropdownMenuRadioItemSpec::new("system", "System")
                            .test_id("ui-gallery-dropdown-menu-radio-group-system"),
                    )
                    .item(
                        shadcn::DropdownMenuRadioItemSpec::new("light", "Light")
                            .test_id("ui-gallery-dropdown-menu-radio-group-light"),
                    )
                    .item(
                        shadcn::DropdownMenuRadioItemSpec::new("dark", "Dark")
                            .test_id("ui-gallery-dropdown-menu-radio-group-dark"),
                    )
                    .into()]
            },
        )
        .test_id("ui-gallery-dropdown-menu-radio-group")
}
// endregion: example
