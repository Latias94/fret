pub const SOURCE: &str = include_str!("parts.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    open: Option<Model<bool>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let state = cx.with_state(Models::default, |st| st.clone());
    let open = match state.open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| st.open = Some(model.clone()));
            model
        }
    };

    let trigger = shadcn::DialogTrigger::new(
        shadcn::Button::new("Open Dialog (Parts)")
            .variant(shadcn::ButtonVariant::Outline)
            .test_id("ui-gallery-dialog-parts-trigger")
            .into_element(cx),
    );

    let content = shadcn::DialogContent::new([
        shadcn::DialogClose::from_scope()
            .into_element(cx)
            .test_id("ui-gallery-dialog-parts-close"),
        shadcn::DialogHeader::new([
            shadcn::DialogTitle::new("Parts dialog").into_element(cx),
            shadcn::DialogDescription::new("Part surface adapter for shadcn-style authoring.")
                .into_element(cx),
        ])
        .into_element(cx),
    ])
    .into_element(cx)
    .test_id("ui-gallery-dialog-parts-content");

    shadcn::Dialog::new(open)
        .compose()
        .trigger(trigger)
        .portal(shadcn::DialogPortal::new())
        .overlay(shadcn::DialogOverlay::new())
        .content(content)
        .into_element(cx)
}
// endregion: example
