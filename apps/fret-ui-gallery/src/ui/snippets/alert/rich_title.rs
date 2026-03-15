pub const SOURCE: &str = include_str!("rich_title.rs");

// region: example
use fret::{UiChild, UiCx};
use std::sync::Arc;

use fret_core::{AttributedText, DecorationLineStyle, TextPaintStyle, TextSpan, UnderlineStyle};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn rich_title_text() -> AttributedText {
    let text: Arc<str> = Arc::from("Let's try one with icon, title and a link.");
    let prefix = "Let's try one with icon, title and a ";
    let link = "link";

    let mut plain = TextSpan::new(prefix.len());
    plain.paint = TextPaintStyle::default();

    let mut underlined = TextSpan::new(link.len());
    underlined.paint = TextPaintStyle::default().with_underline(UnderlineStyle {
        color: None,
        style: DecorationLineStyle::Solid,
    });

    let suffix = TextSpan::new(".".len());

    AttributedText::new(text, Arc::<[TextSpan]>::from([plain, underlined, suffix]))
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::alert(|cx| {
        ui::children![
            cx;
            fret_ui_shadcn::icon::icon(
                cx,
                fret_icons::IconId::new_static("lucide.circle-alert"),
            ),
            shadcn::AlertTitle::new_children([cx.styled_text(rich_title_text())]),
            shadcn::AlertDescription::new(
                "Use `AlertTitle::new_children(...)` when the title needs an attributed or precomposed subtree.",
            ),
        ]
    })
    .variant(shadcn::AlertVariant::Default)
    .refine_layout(LayoutRefinement::default().max_w(Px(520.0)))
    .into_element(cx)
    .test_id("ui-gallery-alert-rich-title")
}
// endregion: example
