pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret_core::{Corners, Px};
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Default)]
struct Models {
    menu_open: Option<Model<bool>>,
    label_value: Option<Model<Option<Arc<str>>>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let (menu_open, label_value) = cx.with_state(Models::default, |st| {
        (st.menu_open.clone(), st.label_value.clone())
    });

    let menu_open = match menu_open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| st.menu_open = Some(model.clone()));
            model
        }
    };

    let label_value = match label_value {
        Some(model) => model,
        None => {
            let model = cx
                .app
                .models_mut()
                .insert(Some(Arc::<str>::from("personal")));
            cx.with_state(Models::default, |st| st.label_value = Some(model.clone()));
            model
        }
    };

    let icon_id = |id: &'static str| fret_icons::IconId::new_static(id);

    let radius = fret_ui::Theme::global(&*cx.app).metric_token("metric.radius.md");
    let corners_last = Corners {
        top_left: Px(0.0),
        bottom_left: Px(0.0),
        top_right: radius,
        bottom_right: radius,
    };

    let menu_trigger = shadcn::Button::new("")
        .a11y_label("More Options")
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Icon)
        .icon(icon_id("lucide.more-horizontal"))
        .toggle_model(menu_open.clone())
        .border_left_width_override(Px(0.0))
        .corner_radii_override(corners_last)
        .into_element(cx);

    let menu = shadcn::DropdownMenu::new(menu_open.clone())
        .align(shadcn::DropdownMenuAlign::End)
        .into_element(
            cx,
            |_cx| menu_trigger,
            |_cx| {
                vec![
                    shadcn::DropdownMenuEntry::Group(shadcn::DropdownMenuGroup::new([
                        shadcn::DropdownMenuEntry::Item(
                            shadcn::DropdownMenuItem::new("Mark as Read")
                                .leading_icon(icon_id("lucide.mail-check")),
                        ),
                        shadcn::DropdownMenuEntry::Item(
                            shadcn::DropdownMenuItem::new("Archive")
                                .leading_icon(icon_id("lucide.archive")),
                        ),
                    ])),
                    shadcn::DropdownMenuEntry::Separator,
                    shadcn::DropdownMenuEntry::Group(shadcn::DropdownMenuGroup::new([
                        shadcn::DropdownMenuEntry::Item(
                            shadcn::DropdownMenuItem::new("Snooze")
                                .leading_icon(icon_id("lucide.clock")),
                        ),
                        shadcn::DropdownMenuEntry::Item(
                            shadcn::DropdownMenuItem::new("Add to Calendar")
                                .leading_icon(icon_id("lucide.calendar-plus")),
                        ),
                        shadcn::DropdownMenuEntry::Item(
                            shadcn::DropdownMenuItem::new("Add to List")
                                .leading_icon(icon_id("lucide.list-filter")),
                        ),
                        shadcn::DropdownMenuEntry::Item(
                            shadcn::DropdownMenuItem::new("Label As...")
                                .leading_icon(icon_id("lucide.tag"))
                                .submenu([shadcn::DropdownMenuEntry::RadioGroup(
                                    shadcn::DropdownMenuRadioGroup::new(label_value.clone())
                                        .item(shadcn::DropdownMenuRadioItemSpec::new(
                                            "personal", "Personal",
                                        ))
                                        .item(shadcn::DropdownMenuRadioItemSpec::new(
                                            "work", "Work",
                                        ))
                                        .item(shadcn::DropdownMenuRadioItemSpec::new(
                                            "other", "Other",
                                        )),
                                )]),
                        ),
                    ])),
                    shadcn::DropdownMenuEntry::Separator,
                    shadcn::DropdownMenuEntry::Group(shadcn::DropdownMenuGroup::new([
                        shadcn::DropdownMenuEntry::Item(
                            shadcn::DropdownMenuItem::new("Trash")
                                .variant(
                                    shadcn::dropdown_menu::DropdownMenuItemVariant::Destructive,
                                )
                                .leading_icon(icon_id("lucide.trash")),
                        ),
                    ])),
                ]
            },
        );

    let back = shadcn::ButtonGroup::new([shadcn::Button::new("")
        .a11y_label("Go Back")
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Icon)
        .icon(icon_id("lucide.arrow-left"))
        .into()]);

    let actions = shadcn::ButtonGroup::new([
        shadcn::Button::new("Archive")
            .variant(shadcn::ButtonVariant::Outline)
            .into(),
        shadcn::Button::new("Report")
            .variant(shadcn::ButtonVariant::Outline)
            .into(),
    ]);

    let snooze = shadcn::ButtonGroup::new([
        shadcn::Button::new("Snooze")
            .variant(shadcn::ButtonVariant::Outline)
            .into(),
        menu.into(),
    ]);

    shadcn::ButtonGroup::new([back.into(), actions.into(), snooze.into()])
        .a11y_label("Button group")
        .into_element(cx)
        .test_id("ui-gallery-button-group-demo")
}

// endregion: example
