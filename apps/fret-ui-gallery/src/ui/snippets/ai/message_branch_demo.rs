pub const SOURCE: &str = include_str!("message_branch_demo.rs");

// region: example
use fret_ui::element::SemanticsDecoration;
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, Radius, Space};
use fret_ui_shadcn::prelude::*;
use std::sync::Arc;

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();

    let branch = move |cx: &mut ElementContext<'_, H>, index: usize, label: &'static str| {
        let theme = theme.clone();
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .gap(Space::N2),
            move |cx| {
                vec![
                    cx.text("").attach_semantics(
                        SemanticsDecoration::default().test_id(Arc::<str>::from(format!(
                            "ui-ai-message-branch-active-marker-{index}"
                        ))),
                    ),
                    cx.container(
                        decl_style::container_props(
                            &theme,
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

    stack::vstack(
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
    )
}
// endregion: example

