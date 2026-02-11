use super::*;

#[path = "hover_card/fixtures.rs"]
mod fixtures;

fn build_shadcn_hover_card_demo_page(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_core::Px;
    use fret_ui::Theme;
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::{ColorRef, LayoutRefinement, Space, ui};
    use fret_ui_shadcn::{
        Avatar, AvatarFallback, AvatarImage, Button, ButtonVariant, HoverCard, HoverCardContent,
    };

    let theme = Theme::global(&*cx.app).clone();
    let sm_px = theme.metric_required("font.size");
    let sm_line_height = theme.metric_required("font.line_height");
    let xs_px = theme
        .metric_by_key("component.tooltip.text_px")
        .unwrap_or(Px((sm_px.0 - 2.0).max(10.0)));
    let xs_line_height = theme
        .metric_by_key("component.tooltip.line_height")
        .unwrap_or(Px((sm_line_height.0 - 4.0).max(12.0)));
    let muted_fg = theme.color_required("muted.foreground");

    let trigger_el = Button::new("@nextjs")
        .variant(ButtonVariant::Link)
        .into_element(cx);

    let avatar = Avatar::new([
        AvatarImage::maybe(None).into_element(cx),
        AvatarFallback::new("VC").into_element(cx),
    ])
    .into_element(cx);

    let heading = ui::text(cx, "@nextjs")
        .text_size_px(sm_px)
        .line_height_px(sm_line_height)
        .font_semibold()
        .into_element(cx);
    let body = ui::text(
        cx,
        "The React Framework – created and maintained by @vercel.",
    )
    .text_size_px(sm_px)
    .line_height_px(sm_line_height)
    .into_element(cx);
    let joined = ui::text(cx, "Joined December 2021")
        .text_size_px(xs_px)
        .line_height_px(xs_line_height)
        .text_color(ColorRef::Color(muted_fg))
        .into_element(cx);

    let text_block = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N1)
            .layout(LayoutRefinement::default().w_px(Px(238.0))),
        move |_cx| vec![heading, body, joined],
    );

    let row = stack::hstack(
        cx,
        stack::HStackProps::default()
            .gap(Space::N4)
            .layout(LayoutRefinement::default().w_full()),
        move |_cx| vec![avatar, text_block],
    );

    let content_el = HoverCardContent::new([row])
        .refine_layout(LayoutRefinement::default().w_px(Px(320.0)))
        .into_element(cx);

    HoverCard::new(trigger_el, content_el)
        .open(Some(open.clone()))
        .into_element(cx)
}
