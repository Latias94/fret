pub const SOURCE: &str = include_str!("tooltip.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_icons::IconId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::time::Duration;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let password = cx.local_model_keyed("password", String::new);
    let email = cx.local_model_keyed("email", String::new);
    let api_key = cx.local_model_keyed("api_key", String::new);

    let max_w = LayoutRefinement::default().w_full().max_w(Px(420.0));

    shadcn::TooltipProvider::new()
        .delay(Duration::ZERO)
        .timeout_duration(Duration::from_millis(400))
        .with(cx, |cx| {
            let info_icon = |cx: &mut UiCx<'_>| {
                icon::icon(cx, IconId::new_static("lucide.info"))
            };

            let password_tooltip = {
                let button = shadcn::InputGroupButton::new("")
                    .a11y_label("Info")
                    .variant(shadcn::ButtonVariant::Ghost)
                    .size(shadcn::InputGroupButtonSize::IconXs)
                    .children([info_icon(cx)])
                    .test_id("ui-gallery-input-group-tooltip-password-info")
                    .into_element(cx);
                let content = shadcn::TooltipContent::build(cx, |_cx| {
                    [shadcn::TooltipContent::text(
                        "Password must be at least 8 characters",
                    )]
                });

                shadcn::Tooltip::new(cx, button, content)
                    .arrow(true)
                    .side(shadcn::TooltipSide::Top)
                    .into_element(cx)
            };

            let email_tooltip = {
                let button = shadcn::InputGroupButton::new("")
                    .a11y_label("Help")
                    .variant(shadcn::ButtonVariant::Ghost)
                    .size(shadcn::InputGroupButtonSize::IconXs)
                    .children([icon::icon(
                        cx,
                        IconId::new_static("lucide.circle-help"),
                    )])
                    .test_id("ui-gallery-input-group-tooltip-email-help")
                    .into_element(cx);
                let content = shadcn::TooltipContent::build(cx, |_cx| {
                    [shadcn::TooltipContent::text(
                        "We'll use this to send you notifications",
                    )]
                });

                shadcn::Tooltip::new(cx, button, content)
                    .arrow(true)
                    .side(shadcn::TooltipSide::Top)
                    .into_element(cx)
            };

            let api_key_tooltip = {
                let button = shadcn::InputGroupButton::new("")
                    .a11y_label("Help")
                    .variant(shadcn::ButtonVariant::Ghost)
                    .size(shadcn::InputGroupButtonSize::IconXs)
                    .children([icon::icon(
                        cx,
                        IconId::new_static("lucide.circle-help"),
                    )])
                    .test_id("ui-gallery-input-group-tooltip-api-key-help")
                    .into_element(cx);
                let content = shadcn::TooltipContent::build(cx, |_cx| {
                    [shadcn::TooltipContent::text("Click for help with API keys")]
                });

                shadcn::Tooltip::new(cx, button, content)
                    .arrow(true)
                    .side(shadcn::TooltipSide::Left)
                    .into_element(cx)
            };

            let password_group = shadcn::InputGroup::new(password)
                .a11y_label("Password")
                .placeholder("Enter password")
                .control_test_id("ui-gallery-input-group-tooltip-password-control")
                .trailing([password_tooltip])
                .trailing_has_button(true)
                .refine_layout(LayoutRefinement::default().w_full().min_w_0())
                .into_element(cx);

            let email_group = shadcn::InputGroup::new(email)
                .a11y_label("Email")
                .placeholder("Your email address")
                .control_test_id("ui-gallery-input-group-tooltip-email-control")
                .trailing([email_tooltip])
                .trailing_has_button(true)
                .refine_layout(LayoutRefinement::default().w_full().min_w_0())
                .into_element(cx);

            let api_key_group = shadcn::InputGroup::new(api_key)
                .a11y_label("API key")
                .placeholder("Enter API key")
                .control_test_id("ui-gallery-input-group-tooltip-api-key-control")
                .trailing([api_key_tooltip])
                .trailing_has_button(true)
                .refine_layout(LayoutRefinement::default().w_full().min_w_0())
                .into_element(cx);

            vec![
                ui::v_flex(|_cx| vec![password_group, email_group, api_key_group])
                    .gap(Space::N4)
                    .layout(max_w)
                    .into_element(cx)
                    .test_id("ui-gallery-input-group-tooltip"),
            ]
        })
        .into_iter()
        .next()
        .expect("tooltip provider returns one root element")
}
// endregion: example
