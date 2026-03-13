pub const SOURCE: &str = include_str!("popover.rs");

// region: example
use fret_core::{Corners, Px};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let open = cx.local_model_keyed("open", || false);
    let text = cx.local_model_keyed("text", String::new);

    let icon_id = |id: &'static str| fret_icons::IconId::new_static(id);

    let radius = fret_ui::Theme::global(&*cx.app).metric_token("metric.radius.md");
    let corners_last = Corners {
        top_left: Px(0.0),
        bottom_left: Px(0.0),
        top_right: radius,
        bottom_right: radius,
    };

    let popover = shadcn::Popover::from_open(open.clone())
        .side(shadcn::PopoverSide::Bottom)
        .align(shadcn::PopoverAlign::End)
        .into_element_with(
            cx,
            |cx| {
                shadcn::Button::new("")
                    .a11y_label("Open popover")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Icon)
                    .icon(icon_id("lucide.chevron-down"))
                    .toggle_model(open.clone())
                    .border_left_width_override(Px(0.0))
                    .corner_radii_override(corners_last)
                    .into_element(cx)
            },
            |cx| {
                shadcn::PopoverContent::new(vec![
                    shadcn::PopoverHeader::new([
                        shadcn::PopoverTitle::new("Start a new task with Copilot").into_element(cx),
                        shadcn::PopoverDescription::new("Describe your task in natural language.")
                            .into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::Field::new([
                        fret_ui_kit::primitives::visually_hidden::visually_hidden_label(
                            cx,
                            "Task Description",
                        ),
                        shadcn::Textarea::new(text)
                            .a11y_label("Task Description")
                            .placeholder("I need to...")
                            .resizable(false)
                            .into_element(cx),
                        shadcn::FieldDescription::new(
                            "Copilot will open a pull request for review.",
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .refine_style(ChromeRefinement::default().rounded(Radius::Lg))
                .into_element(cx)
            },
        );

    shadcn::ButtonGroup::new([
        shadcn::Button::new("Copilot")
            .variant(shadcn::ButtonVariant::Outline)
            .leading_icon(icon_id("lucide.bot"))
            .into(),
        popover.into(),
    ])
    .into_element(cx)
    .test_id("ui-gallery-button-group-popover")
}

// endregion: example
