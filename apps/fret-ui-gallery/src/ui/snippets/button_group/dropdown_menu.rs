// region: example
use fret_core::{Corners, Px};
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default)]
struct Models {
    open: Option<Model<bool>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let open = cx.with_state(Models::default, |st| st.open.clone());
    let open = match open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| st.open = Some(model.clone()));
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

    let dropdown_trigger = shadcn::Button::new("")
        .a11y_label("More")
        .variant(shadcn::ButtonVariant::Outline)
        .refine_style(ChromeRefinement::default().pl(Space::N2))
        .children([shadcn::icon::icon(cx, icon_id("lucide.chevron-down"))])
        .toggle_model(open.clone())
        .border_left_width_override(Px(0.0))
        .corner_radii_override(corners_last)
        .into_element(cx);

    let dropdown = shadcn::DropdownMenu::new(open.clone())
        .align(shadcn::DropdownMenuAlign::End)
        .into_element(
            cx,
            |_cx| dropdown_trigger,
            |_cx| {
                vec![
                    shadcn::DropdownMenuEntry::Group(shadcn::DropdownMenuGroup::new([
                        shadcn::DropdownMenuEntry::Item(
                            shadcn::DropdownMenuItem::new("Mute Conversation")
                                .leading_icon(icon_id("lucide.volume-x")),
                        ),
                        shadcn::DropdownMenuEntry::Item(
                            shadcn::DropdownMenuItem::new("Mark as Read")
                                .leading_icon(icon_id("lucide.check")),
                        ),
                        shadcn::DropdownMenuEntry::Item(
                            shadcn::DropdownMenuItem::new("Report Conversation")
                                .leading_icon(icon_id("lucide.alert-triangle")),
                        ),
                        shadcn::DropdownMenuEntry::Item(
                            shadcn::DropdownMenuItem::new("Block User")
                                .leading_icon(icon_id("lucide.user-round-x")),
                        ),
                        shadcn::DropdownMenuEntry::Item(
                            shadcn::DropdownMenuItem::new("Share Conversation")
                                .leading_icon(icon_id("lucide.share")),
                        ),
                        shadcn::DropdownMenuEntry::Item(
                            shadcn::DropdownMenuItem::new("Copy Conversation")
                                .leading_icon(icon_id("lucide.copy")),
                        ),
                    ])),
                    shadcn::DropdownMenuEntry::Separator,
                    shadcn::DropdownMenuEntry::Group(shadcn::DropdownMenuGroup::new([
                        shadcn::DropdownMenuEntry::Item(
                            shadcn::DropdownMenuItem::new("Delete Conversation")
                                .variant(
                                    shadcn::dropdown_menu::DropdownMenuItemVariant::Destructive,
                                )
                                .leading_icon(icon_id("lucide.trash")),
                        ),
                    ])),
                ]
            },
        );

    shadcn::ButtonGroup::new([
        shadcn::Button::new("Follow")
            .variant(shadcn::ButtonVariant::Outline)
            .into(),
        dropdown.into(),
    ])
    .into_element(cx)
    .test_id("ui-gallery-button-group-dropdown")
}

// endregion: example
