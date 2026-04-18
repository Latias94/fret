pub const SOURCE: &str = include_str!("sources_demo.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_markdown::OnLinkActivate;
use fret_ui_ai as ui_ai;
use std::sync::Arc;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let on_open_url: OnLinkActivate = Arc::new(move |_host, _acx, _reason, link| {
        tracing::info!("sources_demo open_url: {}", link.href.as_ref());
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
            ui_ai::Source::new(title)
                .href(href)
                .on_open_url(on_open_url.clone())
                .into_element(cx)
        })
        .collect::<Vec<_>>();

    ui_ai::Sources::new()
        .test_id("ui-ai-sources-demo-root")
        .into_element_parts(
            ui_ai::SourcesTrigger::new(count),
            ui_ai::SourcesContent::new(items),
            cx,
        )
}
// endregion: example
