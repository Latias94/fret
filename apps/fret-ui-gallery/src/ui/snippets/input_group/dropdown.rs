pub const SOURCE: &str = include_str!("dropdown.rs");

// region: example
use fret_core::Px;
use fret_icons::IconId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let value = cx.local_model_keyed("value", String::new);
    let open = cx.local_model_keyed("open", || false);

    let trigger = shadcn::InputGroupButton::new("")
        .a11y_label("More")
        .test_id("ui-gallery-input-group-dropdown-leading-button")
        .variant(shadcn::ButtonVariant::Ghost)
        .size(shadcn::InputGroupButtonSize::IconXs)
        .children([fret_ui_shadcn::icon::icon(
            cx,
            IconId::new_static("lucide.more-horizontal"),
        )])
        .into_element(cx);

    let dropdown = shadcn::DropdownMenu::new(open)
        .align(shadcn::DropdownMenuAlign::End)
        .side_offset(Px(8.0))
        .align_offset(Px(-4.0))
        .into_element_parts(
            cx,
            |_cx| shadcn::DropdownMenuTrigger::new(trigger),
            shadcn::DropdownMenuContent::new(),
            |_cx| {
                vec![shadcn::DropdownMenuEntry::Group(
                    shadcn::DropdownMenuGroup::new([
                        shadcn::DropdownMenuEntry::Item(shadcn::DropdownMenuItem::new("Settings")),
                        shadcn::DropdownMenuEntry::Item(shadcn::DropdownMenuItem::new("Copy path")),
                        shadcn::DropdownMenuEntry::Item(shadcn::DropdownMenuItem::new(
                            "Open location",
                        )),
                    ]),
                )]
            },
        );

    shadcn::InputGroup::new(value)
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
        .test_id("ui-gallery-input-group-dropdown")
        .into_element_parts(cx, |_cx| {
            vec![
                shadcn::InputGroupPart::input(
                    shadcn::InputGroupInput::new()
                        .a11y_label("File name")
                        .placeholder("Enter file name")
                        .test_id("ui-gallery-input-group-dropdown-control"),
                ),
                shadcn::InputGroupPart::addon(
                    shadcn::InputGroupAddon::new([dropdown])
                        .align(shadcn::InputGroupAddonAlign::InlineEnd)
                        .has_button(true),
                ),
            ]
        })
}
// endregion: example
