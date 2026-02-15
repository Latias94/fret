use super::super::super::super::*;

pub(in crate::ui) fn preview_ai_shimmer_demo(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::{LayoutRefinement, Space};

    let shimmer = ui_ai::Shimmer::new(Arc::<str>::from("Streaming"))
        .test_id("ui-ai-shimmer-root")
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .h_px(Px(48.0))
                .min_w_0(),
        )
        .into_element(cx);

    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N4),
        move |cx| {
            vec![
                cx.text("Shimmer (AI Elements)"),
                cx.text("Animated shimmer used as a lightweight streaming indicator."),
                shimmer,
            ]
        },
    )]
}
