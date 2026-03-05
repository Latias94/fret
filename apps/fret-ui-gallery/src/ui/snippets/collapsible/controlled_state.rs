pub const SOURCE: &str = include_str!("controlled_state.rs");

// region: example
use fret_ui::Invalidation;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    open: Option<Model<bool>>,
}

fn open_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<bool> {
    let state = cx.with_state(Models::default, |st| st.clone());
    match state.open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| st.open = Some(model.clone()));
            model
        }
    }
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let open = open_model(cx);
    let open_now = cx
        .get_model_copied(&open, Invalidation::Layout)
        .unwrap_or(false);

    ui::v_flex(|cx| {
            vec![
                shadcn::typography::muted(
                    cx,
                    if open_now {
                        "open=true (controlled)"
                    } else {
                        "open=false (controlled)"
                    },
                ),
                shadcn::Collapsible::new(open.clone()).into_element_with_open_model(
                    cx,
                    |cx, open, is_open| {
                        shadcn::Button::new(if is_open { "Collapse" } else { "Expand" })
                            .variant(shadcn::ButtonVariant::Outline)
                            .toggle_model(open)
                            .test_id("ui-gallery-collapsible-controlled-trigger")
                            .into_element(cx)
                    },
                    |cx| {
                        shadcn::CollapsibleContent::new(vec![cx.text(
                            "This panel is controlled by `Model<bool>` and mirrors shadcn open/onOpenChange behavior.",
                        )])
                        .refine_layout(LayoutRefinement::default().w_full().mt(Space::N2))
                        .into_element(cx)
                        .test_id("ui-gallery-collapsible-controlled-content")
                    },
                ),
            ]
        })
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full().max_w(Px(420.0))).into_element(cx)
    .test_id("ui-gallery-collapsible-controlled")
}
// endregion: example
