pub const SOURCE: &str = include_str!("sticky_footer.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    open: Option<Model<bool>>,
}

fn lorem_block<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    prefix: &'static str,
    lines: usize,
) -> AnyElement {
    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            (0..lines)
                .map(|index| {
                    cx.text(format!(
                        "{prefix} {}: This dialog row is intentionally verbose to validate scroll behavior and footer visibility.",
                        index + 1
                    ))
                })
                .collect::<Vec<_>>()
        },
    )
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

    let open_for_trigger = open.clone();
    let close_open = open.clone();

    shadcn::Dialog::new(open.clone()).into_element(
        cx,
        move |cx| {
            shadcn::Button::new("Sticky Footer")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-dialog-sticky-footer-trigger")
                .toggle_model(open_for_trigger.clone())
                .into_element(cx)
        },
        move |cx| {
            let scroll_body = shadcn::ScrollArea::new([lorem_block(cx, "Sticky", 14)])
                .refine_layout(
                    LayoutRefinement::default()
                        .w_full()
                        .h_px(Px(220.0))
                        .min_w_0()
                        .min_h_0(),
                )
                .viewport_test_id("ui-gallery-dialog-sticky-footer-viewport")
                .into_element(cx);

            shadcn::DialogContent::new([
                shadcn::DialogClose::new(close_open.clone()).into_element(cx),
                shadcn::DialogHeader::new([
                    shadcn::DialogTitle::new("Sticky Footer").into_element(cx),
                    shadcn::DialogDescription::new(
                        "The footer remains visible while the content area scrolls.",
                    )
                    .into_element(cx),
                ])
                .into_element(cx),
                scroll_body,
                shadcn::DialogFooter::new([
                    shadcn::Button::new("Close")
                        .variant(shadcn::ButtonVariant::Outline)
                        .toggle_model(close_open.clone())
                        .into_element(cx),
                    shadcn::Button::new("Save changes").into_element(cx),
                ])
                .into_element(cx),
            ])
            .into_element(cx)
            .test_id("ui-gallery-dialog-sticky-footer-content")
        },
    )
}
// endregion: example
