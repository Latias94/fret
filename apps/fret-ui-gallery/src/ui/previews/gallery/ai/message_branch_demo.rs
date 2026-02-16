use super::super::super::super::*;

pub(in crate::ui) fn preview_ai_message_branch_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::{ChromeRefinement, LayoutRefinement, Radius, Space};

    let branch = |cx: &mut ElementContext<'_, App>, index: usize, label: &'static str| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .gap(Space::N2),
            move |cx| {
                vec![
                    cx.text("")
                        .attach_semantics(SemanticsDecoration::default().test_id(
                            Arc::<str>::from(format!("ui-ai-message-branch-active-marker-{index}")),
                        )),
                    cx.container(
                        fret_ui_kit::declarative::style::container_props(
                            theme,
                            ChromeRefinement::default()
                                .border_1()
                                .rounded(Radius::Md)
                                .p(Space::N3),
                            LayoutRefinement::default().w_full().min_w_0(),
                        ),
                        move |cx| vec![cx.text(label)],
                    ),
                ]
            },
        )
    };

    let branches = [
        branch(cx, 0, "Branch 0: original answer"),
        branch(cx, 1, "Branch 1: alternative phrasing"),
        branch(cx, 2, "Branch 2: deeper explanation"),
    ];

    let message_branch = ui_ai::MessageBranch::new(branches)
        .test_id_root("ui-ai-message-branch-root")
        .prev_test_id("ui-ai-message-branch-prev")
        .next_test_id("ui-ai-message-branch-next")
        .into_element(cx);

    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N4),
        move |cx| {
            vec![
                cx.text("MessageBranch (AI Elements)"),
                cx.text("Prev/Next cycles through branches; only active branch is mounted."),
                message_branch,
            ]
        },
    )]
}
