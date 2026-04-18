pub const SOURCE: &str = include_str!("interactive_links.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui::Invalidation;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let last_activated = cx.local_model_keyed("last_activated", || None::<Arc<str>>);
    let last_activated_now = cx
        .get_model_cloned(&last_activated, Invalidation::Paint)
        .unwrap_or_default();

    let paragraph = shadcn::raw::typography::p_rich([
        shadcn::raw::typography::inline_text(
            "The king thought long and hard, and finally came up with ",
        ),
        shadcn::raw::typography::inline_link("a brilliant plan", "https://example.com/kings-plan"),
        shadcn::raw::typography::inline_text(
            ". Activate the link to capture the last href without dropping to low-level selectable-text hooks.",
        ),
    ])
    .on_activate_link(Arc::new({
        let last_activated = last_activated.clone();
        move |host, action_cx, _reason, activation| {
            let _ = host
                .models_mut()
                .update(&last_activated, |value| *value = Some(activation.tag.clone()));
            host.notify(action_cx);
            host.request_redraw(action_cx.window);
        }
    }))
    .into_element(cx)
    .test_id("ui-gallery-typography-interactive-links-paragraph");

    let status = match last_activated_now {
        Some(href) => shadcn::Badge::new(format!("Activated: {}", href.as_ref()))
            .variant(shadcn::BadgeVariant::Secondary)
            .into_element(cx)
            .test_id("ui-gallery-typography-interactive-links-status-active"),
        None => shadcn::raw::typography::muted("Activate the inline link to record the last href.")
            .into_element(cx)
            .test_id("ui-gallery-typography-interactive-links-status-idle"),
    };

    ui::v_flex(move |_cx| vec![paragraph, status])
        .gap(Space::N2)
        .items_start()
        .layout(LayoutRefinement::default().w_full().max_w(Px(560.0)))
        .into_element(cx)
        .test_id("ui-gallery-typography-interactive-links")
}
// endregion: example
