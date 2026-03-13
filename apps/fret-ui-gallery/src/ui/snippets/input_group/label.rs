pub const SOURCE: &str = include_str!("label.rs");

// region: example
use fret_core::Px;
use fret_icons::IconId;
use fret_ui_kit::primitives::control_registry::ControlId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::time::Duration;

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let username = cx.local_model_keyed("username", String::new);
    let email = cx.local_model_keyed("email", String::new);

    let max_w = LayoutRefinement::default().w_full().max_w(Px(420.0));
    let email_id = ControlId::from("ui-gallery-input-group-label-email");

    shadcn::TooltipProvider::new()
        .delay(Duration::ZERO)
        .timeout_duration(Duration::from_millis(400))
        .with(cx, |cx| {
            let at_group = shadcn::InputGroup::new(username)
                .a11y_label("Username")
                .placeholder("shadcn")
                .control_test_id("ui-gallery-input-group-label-at-control")
                .trailing([shadcn::InputGroupText::new("@").into_element(cx)])
                .refine_layout(LayoutRefinement::default().w_full().min_w_0())
                .into_element(cx);

            let help_tooltip = {
                let button = shadcn::InputGroupButton::new("")
                    .a11y_label("Help")
                    .variant(shadcn::ButtonVariant::Ghost)
                    .size(shadcn::InputGroupButtonSize::IconXs)
                    .icon(IconId::new_static("lucide.info"))
                    .test_id("ui-gallery-input-group-label-help")
                    .into_element(cx);
                let content = shadcn::TooltipContent::new(vec![shadcn::TooltipContent::text(
                    cx,
                    "We'll use this to send you notifications",
                )]);

                shadcn::Tooltip::new(cx, button, content)
                    .arrow(true)
                    .side(shadcn::TooltipSide::Top)
                    .into_element(cx)
            };

            let header = ui::h_flex(|cx| {
                vec![
                    shadcn::Label::new("Email")
                        .for_control(email_id.clone())
                        .test_id("ui-gallery-input-group-label-email-label")
                        .into_element(cx),
                    help_tooltip,
                ]
            })
            .layout(LayoutRefinement::default().w_full())
            .justify_between()
            .items_center()
            .into_element(cx);

            let email_group = shadcn::InputGroup::new(email)
                .placeholder("shadcn@vercel.com")
                .control_test_id("ui-gallery-input-group-label-email-control")
                .control_id(email_id.clone())
                .block_start([header])
                .block_start_border_bottom(true)
                .refine_layout(LayoutRefinement::default().w_full().min_w_0())
                .into_element(cx);

            vec![
                ui::v_flex(|_cx| vec![at_group, email_group])
                    .gap(Space::N4)
                    .layout(max_w)
                    .into_element(cx)
                    .test_id("ui-gallery-input-group-label"),
            ]
        })
        .into_iter()
        .next()
        .expect("tooltip provider returns one root element")
}
// endregion: example
