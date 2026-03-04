pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret_core::ImageId;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    avatar_image: Model<Option<ImageId>>,
) -> AnyElement {
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

    let heading = ui::text(cx, "@nextjs")
        .text_sm()
        .font_semibold()
        .into_element(cx)
        .test_id("ui-gallery-hover-card-demo-content-title");
    let body = ui::text(
        cx,
        "The React Framework – created and maintained by @vercel.",
    )
    .text_sm()
    .wrap(TextWrap::WordBreak)
    .into_element(cx)
    .test_id("ui-gallery-hover-card-demo-content-desc");
    let joined = ui::text(cx, "Joined December 2021")
        .text_xs()
        .text_color(ColorRef::Color(muted_fg))
        .into_element(cx)
        .test_id("ui-gallery-hover-card-demo-content-joined");

    let text_block = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N1)
            .items_start(),
        |_cx| vec![heading, body, joined],
    );

    let row = stack::hstack(
        cx,
        stack::HStackProps::default()
            .gap(Space::N4)
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .items_start()
            .justify_between(),
        |_cx| vec![avatar, text_block],
    );

    let content = shadcn::HoverCardContent::new(vec![row])
        .refine_layout(LayoutRefinement::default().max_w(Px(320.0)))
        .into_element(cx)
        .test_id("ui-gallery-hover-card-demo-content");

    shadcn::HoverCard::new(trigger, content)
        .into_element(cx)
        .test_id("ui-gallery-hover-card-demo")
}
// endregion: example
