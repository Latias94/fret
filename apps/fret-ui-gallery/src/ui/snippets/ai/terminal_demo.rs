pub const SOURCE: &str = include_str!("terminal_demo.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui::Invalidation;
use fret_ui::element::{AnyElement, Length, SemanticsProps, SpacerProps};
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::text as decl_text;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::prelude::*;
use std::sync::Arc;

fn state_marker(cx: &mut AppComponentCx<'_>, test_id: &'static str) -> AnyElement {
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
    let output = cx.local_model_keyed("output", || {
        "Building...\n✓ compiled crates\n✓ ran tests\n\n$ echo \"hello\"".to_string()
    });

    let empty_now = cx
        .get_model_cloned(&output, Invalidation::Paint)
        .map(|v| v.trim().is_empty())
        .unwrap_or(false);

    let empty_marker = empty_now.then(|| state_marker(cx, "ui-ai-terminal-demo-output-empty-true"));

    let terminal = ui_ai::Terminal::new(output.clone())
        .on_clear(Arc::new({
            let output = output.clone();
            move |host, _action_cx| {
                let _ = host.models_mut().update(&output, |v| v.clear());
            }
        }))
        .test_id_root("ui-ai-terminal-demo-root")
        .test_id_copy("ui-ai-terminal-demo-copy")
        .copied_marker_test_id("ui-ai-terminal-demo-copied")
        .test_id_clear("ui-ai-terminal-demo-clear")
        .test_id_viewport("ui-ai-terminal-demo-viewport")
        .refine_layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx);

    ui::v_flex(move |cx| {
        let mut out = vec![
            decl_text::text_section_chrome_label(cx, "Terminal (AI Elements)"),
            decl_text::text_paragraph(
                cx,
                "Chrome-only viewer: apps own streaming + clear behavior.",
            ),
            terminal,
        ];
        if let Some(marker) = empty_marker {
            out.push(marker);
        }
        out
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .into_element(cx)
}
// endregion: example
