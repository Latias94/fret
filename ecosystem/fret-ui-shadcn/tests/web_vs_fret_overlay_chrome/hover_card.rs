use super::*;
use std::cell::Cell;
use std::rc::Rc;

#[path = "hover_card/fixtures.rs"]
mod fixtures;

#[derive(Clone, Default)]
struct HoverCardDemoProbe {
    content: Rc<Cell<Option<GlobalElementId>>>,
    row: Rc<Cell<Option<GlobalElementId>>>,
    text_block: Rc<Cell<Option<GlobalElementId>>>,
    body: Rc<Cell<Option<GlobalElementId>>>,
}

fn build_shadcn_hover_card_demo_page_with_probe(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
    probe: Option<&HoverCardDemoProbe>,
) -> AnyElement {
    use fret_core::Px;
    use fret_core::TextWrap;
    use fret_ui::Theme;
    use fret_ui_kit::{ColorRef, LayoutRefinement, Space, ui};
    use fret_ui_shadcn::facade::{
        Avatar, AvatarFallback, AvatarImage, Button, ButtonVariant, HoverCard, HoverCardContent,
    };

    let theme = Theme::global(&*cx.app).snapshot();
    let muted_fg = theme.color_token("muted-foreground");

    let trigger_el = Button::new("@nextjs")
        .variant(ButtonVariant::Link)
        .into_element(cx);

    let avatar = Avatar::new([
        AvatarImage::maybe(None).into_element(cx),
        AvatarFallback::new("VC").into_element(cx),
    ])
    .into_element(cx);

    let heading = ui::text("@nextjs")
        .text_sm()
        .font_semibold()
        .into_element(cx)
        .test_id("hover-card-demo-heading");
    let body = ui::text_block("The React Framework – created and maintained by @vercel.")
        .text_sm()
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .wrap(TextWrap::WordBreak)
        .into_element(cx)
        .test_id("hover-card-demo-body");
    if let Some(probe) = probe {
        probe.body.set(Some(body.id));
    }
    let joined = ui::text("Joined December 2021")
        .text_xs()
        .text_color(ColorRef::Color(muted_fg))
        .into_element(cx)
        .test_id("hover-card-demo-joined");

    let text_block = ui::v_flex(move |_cx| vec![heading, body, joined])
        .gap(Space::N1)
        .layout(LayoutRefinement::default().w_full().flex_1().min_w_0())
        .items_stretch()
        .into_element(cx)
        .test_id("hover-card-demo-text-block");
    if let Some(probe) = probe {
        probe.text_block.set(Some(text_block.id));
    }

    let row = ui::h_flex(move |_cx| vec![avatar, text_block])
        .gap(Space::N4)
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .items_start()
        .justify_between()
        .into_element(cx)
        .test_id("hover-card-demo-row");
    if let Some(probe) = probe {
        probe.row.set(Some(row.id));
    }

    let content_el = HoverCardContent::new([row])
        .test_id("hover-card-demo-content")
        .refine_layout(LayoutRefinement::default().max_w(Px(320.0)))
        .into_element(cx);
    if let Some(probe) = probe {
        probe.content.set(Some(content_el.id));
    }

    HoverCard::new(cx, trigger_el, content_el)
        .open(Some(open.clone()))
        .into_element(cx)
}
