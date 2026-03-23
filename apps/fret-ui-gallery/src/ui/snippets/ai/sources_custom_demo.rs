pub const SOURCE: &str = include_str!("sources_custom_demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_icons::ids;
use fret_icons_lucide::generated_ids::lucide;
use fret_markdown::OnLinkActivate;
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::icon as decl_icon;
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let on_open_url: OnLinkActivate = Arc::new(move |_host, _acx, _reason, link| {
        tracing::info!("sources_custom_demo open_url: {}", link.href.as_ref());
    });

    let sources = [
        ("https://stripe.com/docs/api", "Stripe API Documentation"),
        ("https://docs.github.com/en/rest", "GitHub REST API"),
        (
            "https://docs.aws.amazon.com/sdk-for-javascript/",
            "AWS SDK for JavaScript",
        ),
    ];
    let count = sources.len();

    let items = sources
        .into_iter()
        .map(|(href, title)| {
            let external_icon =
                decl_icon::icon_with(cx, lucide::EXTERNAL_LINK, Some(Px(16.0)), None);
            ui_ai::Source::new(title)
                .href(href)
                .on_open_url(on_open_url.clone())
                .children([cx.text(title), external_icon])
                .into_element(cx)
        })
        .collect::<Vec<_>>();

    let trigger_chevron = decl_icon::icon_with(cx, ids::ui::CHEVRON_DOWN, Some(Px(16.0)), None);

    ui_ai::Sources::new()
        .test_id("ui-ai-sources-custom-demo-root")
        .into_element_parts(
            ui_ai::SourcesTrigger::new(count)
                .title("Using {count} citations")
                .children([cx.text(format!("Using {count} citations")), trigger_chevron]),
            ui_ai::SourcesContent::new(items),
            cx,
        )
}
// endregion: example
