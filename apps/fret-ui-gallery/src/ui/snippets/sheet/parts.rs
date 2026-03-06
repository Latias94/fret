pub const SOURCE: &str = include_str!("parts.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    open: Option<Model<bool>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let open = cx.with_state(Models::default, |st| st.open.clone());
    let open = match open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| st.open = Some(model.clone()));
            model
        }
    };

    let trigger = shadcn::SheetTrigger::build(
        shadcn::Button::new("Open (Parts)")
            .variant(shadcn::ButtonVariant::Outline)
            .test_id("ui-gallery-sheet-parts-trigger"),
    );

    let content = shadcn::SheetContent::new([
        shadcn::SheetHeader::new([
            shadcn::SheetTitle::new("Parts sheet").into_element(cx),
            shadcn::SheetDescription::new("Part surface adapter for shadcn-style authoring.")
                .into_element(cx),
        ])
        .into_element(cx),
        shadcn::SheetFooter::new([shadcn::SheetClose::from_scope()
            .into_element(cx)
            .test_id("ui-gallery-sheet-parts-close")])
        .into_element(cx),
    ])
    .into_element(cx)
    .test_id("ui-gallery-sheet-parts-overlay-content");

    shadcn::Sheet::new(open)
        .compose()
        .trigger(trigger)
        .portal(shadcn::SheetPortal::new())
        .overlay(shadcn::SheetOverlay::new())
        .content(content)
        .into_element(cx)
        .test_id("ui-gallery-sheet-parts-overlay")
}
// endregion: example
