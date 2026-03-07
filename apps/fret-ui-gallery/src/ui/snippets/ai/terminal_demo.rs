pub const SOURCE: &str = include_str!("terminal_demo.rs");

// region: example
use fret_runtime::Model;
use fret_ui::element::{ContainerProps, LayoutStyle, Length, SemanticsProps, SizeStyle};
use fret_ui::Invalidation;
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::prelude::*;
use std::sync::Arc;

#[derive(Default)]
struct DemoModels {
    output: Option<Model<String>>,
}

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let output = cx.with_state(DemoModels::default, |st| st.output.clone());
    let output = match output {
        Some(model) => model,
        None => {
            let seed = "Building...\n✓ compiled crates\n✓ ran tests\n\n$ echo \"hello\"";
            let model = cx.app.models_mut().insert(seed.to_string());
            cx.with_state(DemoModels::default, |st| st.output = Some(model.clone()));
            model
        }
    };

    let empty_now = cx
        .get_model_cloned(&output, Invalidation::Paint)
        .map(|v| v.trim().is_empty())
        .unwrap_or(false);

    let empty_marker = empty_now.then(|| {
        cx.semantics(
            SemanticsProps {
                role: fret_core::SemanticsRole::Text,
                test_id: Some(Arc::<str>::from("ui-ai-terminal-demo-output-empty-true")),
                ..Default::default()
            },
            |cx| {
                vec![cx.container(
                    ContainerProps {
                        layout: LayoutStyle {
                            size: SizeStyle {
                                width: Length::Px(Px(0.0)),
                                height: Length::Px(Px(0.0)),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    |_cx| Vec::new(),
                )]
            },
        )
    });

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
            cx.text("Terminal (AI Elements)"),
            cx.text("Chrome-only viewer: apps own streaming + clear behavior."),
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
