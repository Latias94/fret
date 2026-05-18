pub const SOURCE: &str = include_str!("controlled_state.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui::Invalidation;
use fret_ui_kit::declarative::text as decl_text;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let open = cx.local_model_keyed("open", || false);
    let open_now = cx
        .get_model_copied(&open, Invalidation::Layout)
        .unwrap_or(false);

    ui::v_flex(|cx| {
            vec![
                decl_text::text_control_readout(cx,
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
                        shadcn::CollapsibleContent::new(vec![decl_text::text_paragraph(
                            cx,
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
