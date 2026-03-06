pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let sonner = shadcn::Sonner::global(&mut *cx.app);
    let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
        sonner.toast_message(
            host,
            action_cx.window,
            "Event has been created.",
            shadcn::ToastMessageOptions::new(),
        );
        host.request_redraw(action_cx.window);
    });

    shadcn::Button::new("Show toast")
        .on_activate(on_activate)
        .test_id("ui-gallery-sonner-usage")
        .into_element(cx)
}
// endregion: example
