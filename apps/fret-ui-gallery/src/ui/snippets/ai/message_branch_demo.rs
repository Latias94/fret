pub const SOURCE: &str = include_str!("message_branch_demo.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui::element::{AnyElement, Length, SemanticsProps, SpacerProps};
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::declarative::text as decl_text;
use fret_ui_kit::ui;
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, Radius, Space};
use fret_ui_shadcn::prelude::*;
use std::sync::Arc;

fn state_marker(cx: &mut AppComponentCx<'_>, test_id: String) -> AnyElement {
    cx.semantics(
        SemanticsProps {
            role: fret_core::SemanticsRole::Generic,
            test_id: Some(Arc::<str>::from(test_id)),
            ..Default::default()
        },
        |cx| {
            vec![cx.spacer(SpacerProps {
                layout: fret_ui::element::LayoutStyle {
                    size: fret_ui::element::SizeStyle {
                        width: Length::Px(Px(0.0)),
                        height: Length::Px(Px(0.0)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                min: Px(0.0),
            })]
        },
    )
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let theme = Theme::global(&*cx.app).clone();

    let branch = move |cx: &mut AppComponentCx<'_>, index: usize, label: &'static str| {
        let theme = theme.clone();
        ui::v_flex(move |cx| {
            vec![
                state_marker(cx, format!("ui-ai-message-branch-active-marker-{index}")),
                cx.container(
                    decl_style::container_props(
                        &theme,
                        ChromeRefinement::default()
                            .border_1()
                            .rounded(Radius::Md)
                            .p(Space::N3),
                        LayoutRefinement::default().w_full().min_w_0(),
                    ),
                    move |cx| vec![decl_text::text_paragraph(cx, label)],
                ),
            ]
        })
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .gap(Space::N2)
        .into_element(cx)
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

    ui::v_flex(move |cx| {
        vec![
            decl_text::text_section_chrome_label(cx, "MessageBranch (AI Elements)"),
            decl_text::text_paragraph(
                cx,
                "Prev/Next cycles through branches; only active branch is mounted.",
            ),
            message_branch,
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .into_element(cx)
}
// endregion: example
