pub const SOURCE: &str = include_str!("radio_group.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::declarative::ModelWatchExt as _;
use fret_ui_shadcn::facade as shadcn;
use std::sync::Arc;

#[derive(Default, Clone)]
struct PanelSettings {
    position: Option<Arc<str>>,
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let settings = cx.local_model(|| PanelSettings {
        position: Some(Arc::<str>::from("bottom")),
    });
    let settings_now = cx
        .watch_model(&settings)
        .layout()
        .cloned()
        .unwrap_or_default();

    super::preview_frame_with(cx, |cx| {
        shadcn::DropdownMenu::uncontrolled(cx).build_parts(
            cx,
            shadcn::DropdownMenuTrigger::build(
                shadcn::Button::new("Open")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-dropdown-menu-radio-group-trigger"),
            ),
            shadcn::DropdownMenuContent::new()
                .align(shadcn::DropdownMenuAlign::Start)
                .side_offset(Px(4.0))
                // new-york-v4 dropdown-menu-radio-group: `DropdownMenuContent className="w-56"`.
                .min_width(Px(224.0)),
            |_cx| {
                [shadcn::DropdownMenuGroup::new([
                    shadcn::DropdownMenuLabel::new("Panel Position").into(),
                    shadcn::DropdownMenuRadioGroup::from_value(settings_now.position.clone())
                        .on_value_change({
                            let settings = settings.clone();
                            move |host, _action_cx, value| {
                                let _ = host
                                    .models_mut()
                                    .update(&settings, |state| state.position = Some(value));
                            }
                        })
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
    })
    .into_element(cx)
}
// endregion: example
