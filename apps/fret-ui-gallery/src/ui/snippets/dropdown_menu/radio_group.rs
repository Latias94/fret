pub const SOURCE: &str = include_str!("radio_group.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Default, Clone)]
struct Models {
    position: Option<Model<Option<Arc<str>>>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let state = cx.with_state(Models::default, |st| st.clone());

    let position = match state.position {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(Some(Arc::<str>::from("bottom")));
            cx.with_state(Models::default, |st| st.position = Some(model.clone()));
            model
        }
    };

    shadcn::DropdownMenu::new_controllable(cx, None, false)
        .into_element_parts(
            cx,
            |cx| {
                shadcn::DropdownMenuTrigger::new(
                    shadcn::Button::new("Open")
                        .variant(shadcn::ButtonVariant::Outline)
                        .test_id("ui-gallery-dropdown-menu-radio-group-trigger")
                        .into_element(cx),
                )
            },
            shadcn::DropdownMenuContent::new()
                .align(shadcn::DropdownMenuAlign::Start)
                .side_offset(Px(4.0))
                // shadcn/ui docs: `DropdownMenuContent className="w-32"`.
                .min_width(Px(128.0)),
            |_cx| {
                [shadcn::DropdownMenuGroup::new([
                    shadcn::DropdownMenuLabel::new("Panel Position").into(),
                    shadcn::DropdownMenuRadioGroup::new(position.clone())
                        .item(
                            shadcn::DropdownMenuRadioItemSpec::new("top", "Top")
                                .test_id("ui-gallery-dropdown-menu-radio-group-top"),
                        )
                        .item(
                            shadcn::DropdownMenuRadioItemSpec::new("bottom", "Bottom")
                                .test_id("ui-gallery-dropdown-menu-radio-group-bottom"),
                        )
                        .item(
                            shadcn::DropdownMenuRadioItemSpec::new("right", "Right")
                                .test_id("ui-gallery-dropdown-menu-radio-group-right"),
                        )
                        .into(),
                ])
                .into()]
            },
        )
        .test_id("ui-gallery-dropdown-menu-radio-group")
}
// endregion: example
