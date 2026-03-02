pub const SOURCE: &str = include_str!("parts.rs");

// region: example
use fret_core::Px;
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

    let trigger_open = open.clone();
    let close_open = open.clone();

    shadcn::Sheet::new(open.clone())
        .side(shadcn::SheetSide::Right)
        .size(Px(420.0))
        .into_element_parts(
            cx,
            move |cx| {
                let trigger = shadcn::Button::new("Open (Parts)")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-sheet-parts-trigger")
                    .toggle_model(trigger_open.clone())
                    .into_element(cx);
                shadcn::SheetTrigger::new(trigger)
            },
            shadcn::SheetPortal::new(),
            shadcn::SheetOverlay::new(),
            move |cx| {
                shadcn::SheetContent::new([
                    shadcn::SheetHeader::new([
                        shadcn::SheetTitle::new("Parts sheet").into_element(cx),
                        shadcn::SheetDescription::new(
                            "Part surface adapter for shadcn-style authoring.",
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::SheetFooter::new([shadcn::Button::new("Close")
                        .variant(shadcn::ButtonVariant::Outline)
                        .test_id("ui-gallery-sheet-parts-close")
                        .toggle_model(close_open.clone())
                        .into_element(cx)])
                    .into_element(cx),
                ])
                .into_element(cx)
                .test_id("ui-gallery-sheet-parts-overlay-content")
            },
        )
        .test_id("ui-gallery-sheet-parts-overlay")
}
// endregion: example
