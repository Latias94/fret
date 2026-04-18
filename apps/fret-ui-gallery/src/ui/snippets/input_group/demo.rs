pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_icons::IconId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::time::Duration;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let search = cx.local_model_keyed("search", String::new);
    let website = cx.local_model_keyed("website", String::new);
    let ask = cx.local_model_keyed("ask", String::new);
    let handle = cx.local_model_keyed("handle", String::new);

    let max_w = LayoutRefinement::default().w_full().max_w(Px(320.0));

    shadcn::TooltipProvider::new()
        .delay(Duration::ZERO)
        .timeout_duration(Duration::from_millis(400))
        .with(cx, |cx| {
            let search_group = shadcn::InputGroup::new(search)
                .a11y_label("Search")
                .placeholder("Search...")
                .leading([icon::icon(cx, IconId::new_static("lucide.search"))])
                .trailing([shadcn::InputGroupText::new("12 results").into_element(cx)])
                .refine_layout(LayoutRefinement::default().w_full().min_w_0())
                .into_element(cx);

            let info_tooltip = {
                let trigger = shadcn::InputGroupButton::new("")
                    .a11y_label("Info")
                    .variant(shadcn::ButtonVariant::Ghost)
                    .size(shadcn::InputGroupButtonSize::IconXs)
                    .icon(IconId::new_static("lucide.info"))
                    .into_element(cx);
                let content = shadcn::TooltipContent::build(cx, |_cx| {
                    [shadcn::TooltipContent::text(
                        "This is content in a tooltip.",
                    )]
                });

                shadcn::Tooltip::new(cx, trigger, content)
                    .arrow(true)
                    .side(shadcn::TooltipSide::Top)
                    .into_element(cx)
            };

            let website_group = shadcn::InputGroup::new(website)
                .a11y_label("Website")
                .placeholder("example.com")
                .leading([shadcn::InputGroupText::new("https://").into_element(cx)])
                .trailing([info_tooltip])
                .trailing_has_button(true)
                .refine_layout(LayoutRefinement::default().w_full().min_w_0())
                .into_element(cx);

            let mode_dropdown = {
                let trigger = shadcn::InputGroupButton::new("Auto")
                    .variant(shadcn::ButtonVariant::Ghost)
                    .trailing_icon(IconId::new_static("lucide.chevron-down"))
                    .into_element(cx);

                shadcn::DropdownMenu::uncontrolled(cx)
                    .compose()
                    .trigger(trigger)
                    .content(
                        shadcn::DropdownMenuContent::new()
                            .side(shadcn::DropdownMenuSide::Top)
                            .align(shadcn::DropdownMenuAlign::Start)
                            .side_offset(Px(8.0)),
                    )
                    .entries([shadcn::DropdownMenuGroup::new([
                        shadcn::DropdownMenuEntry::Item(shadcn::DropdownMenuItem::new("Auto")),
                        shadcn::DropdownMenuEntry::Item(shadcn::DropdownMenuItem::new("Agent")),
                        shadcn::DropdownMenuEntry::Item(shadcn::DropdownMenuItem::new("Manual")),
                    ])
                    .into()])
                    .into_element(cx)
            };

            let compose_group = shadcn::InputGroup::new(ask)
                .textarea()
                .a11y_label("Ask, Search or Chat")
                .placeholder("Ask, Search or Chat...")
                .block_end([
                    shadcn::InputGroupButton::new("")
                        .a11y_label("Add")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::InputGroupButtonSize::IconXs)
                        .icon(IconId::new_static("lucide.plus"))
                        .into_element(cx),
                    mode_dropdown,
                    shadcn::InputGroupText::new("52% used")
                        .refine_layout(LayoutRefinement::default().ml_auto())
                        .into_element(cx),
                    shadcn::Separator::new()
                        .orientation(shadcn::SeparatorOrientation::Vertical)
                        .refine_layout(LayoutRefinement::default().h_px(Px(16.0)))
                        .into_element(cx),
                    shadcn::InputGroupButton::new("")
                        .a11y_label("Send")
                        .variant(shadcn::ButtonVariant::Default)
                        .size(shadcn::InputGroupButtonSize::IconXs)
                        .icon(IconId::new_static("lucide.arrow-up"))
                        .disabled(true)
                        .into_element(cx),
                ])
                .block_end_border_top(true)
                .textarea_min_height(Px(120.0))
                .refine_layout(LayoutRefinement::default().w_full().min_w_0())
                .into_element(cx);

            let verified_group = shadcn::InputGroup::new(handle)
                .a11y_label("Handle")
                .placeholder("@shadcn")
                .trailing([icon::icon(cx, IconId::new_static("lucide.check"))])
                .refine_layout(LayoutRefinement::default().w_full().min_w_0())
                .into_element(cx);

            vec![
                ui::v_stack(move |_cx| {
                    vec![search_group, website_group, compose_group, verified_group]
                })
                .gap(Space::N4)
                .items_start()
                .layout(max_w)
                .into_element(cx)
                .test_id("ui-gallery-input-group-demo"),
            ]
        })
        .into_iter()
        .next()
        .expect("tooltip provider returns one root element")
}
// endregion: example
