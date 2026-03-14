pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::ImageId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>, avatar_image: Model<Option<ImageId>>) -> impl UiChild + use<> {
    // Align with upstream shadcn/ui HoverCard demo composition:
    // - trigger: link-style button (`@nextjs`)
    // - content: `w-80` (320px), avatar + text block
    let trigger = shadcn::Button::new("@nextjs")
        .variant(shadcn::ButtonVariant::Link)
        .test_id("ui-gallery-hover-card-demo-trigger")
        .into_element(cx);

    let theme = Theme::global(&*cx.app).snapshot();
    let muted_fg = theme.color_token("muted-foreground");

    let avatar_image_el = shadcn::AvatarImage::model(avatar_image.clone()).into_element(cx);
    let avatar_fallback = shadcn::AvatarFallback::new("VC")
        .when_image_missing_model(avatar_image)
        .delay_ms(120)
        .into_element(cx);
    let avatar = shadcn::Avatar::new([avatar_image_el, avatar_fallback]).into_element(cx);

    let heading = ui::text("@nextjs")
        .text_sm()
        .font_semibold()
        .into_element(cx)
        .test_id("ui-gallery-hover-card-demo-content-title");
    let body = ui::text_block("The React Framework – created and maintained by @vercel.")
        .text_sm()
        .wrap(TextWrap::WordBreak)
        .into_element(cx)
        .test_id("ui-gallery-hover-card-demo-content-desc");
    let joined = ui::text("Joined December 2021")
        .text_xs()
        .text_color(ColorRef::Color(muted_fg))
        .into_element(cx)
        .test_id("ui-gallery-hover-card-demo-content-joined");

    let text_block = ui::v_flex(|_cx| vec![heading, body, joined])
        // In a horizontal flex row, ensure the text column participates in flex sizing so wrapped
        // text does not collapse to its min-content width (which can be a single grapheme).
        .layout(LayoutRefinement::default().w_full().flex_1().min_w_0())
        .gap(Space::N1)
        // Keep children (text runs) stretched to the column width so `TextWrap` uses the expected
        // wrap width rather than collapsing to min-content.
        .items_stretch()
        .into_element(cx);
    let text_block = text_block.test_id("ui-gallery-hover-card-demo-text-block");

    let row = ui::h_flex(|_cx| vec![avatar, text_block])
        .gap(Space::N4)
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .items_start()
        .justify_between()
        .into_element(cx);
    let row = row.test_id("ui-gallery-hover-card-demo-row");

    let content = shadcn::HoverCardContent::new(vec![row])
        .refine_layout(LayoutRefinement::default().max_w(Px(320.0)))
        .into_element(cx)
        .test_id("ui-gallery-hover-card-demo-content");

    shadcn::HoverCard::new(cx, trigger, content)
        .into_element(cx)
        .test_id("ui-gallery-hover-card-demo")
}
// endregion: example
