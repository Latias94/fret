use super::super::super::super::*;

pub(in crate::ui) fn preview_ai_snippet_demo(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    use std::sync::Arc;

    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::{LayoutRefinement, Space};

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .gap(Space::N2),
            move |cx| vec![cx.text(title), body],
        )
    };

    let snippet = |cx: &mut ElementContext<'_, App>,
                   prefix: Option<&'static str>,
                   code: &'static str,
                   id: &'static str| {
        let mut children: Vec<AnyElement> = Vec::new();
        if let Some(prefix) = prefix {
            children.push(ui_ai::SnippetText::new(prefix).into_element(cx));
        }
        children.push(ui_ai::SnippetInput::new(code).into_element(cx));
        children.push(
            ui_ai::SnippetCopyButton::new(code)
                .test_id(Arc::<str>::from(format!("{id}-copy")))
                .copied_marker_test_id(Arc::<str>::from(format!("{id}-copied")))
                .into_element(cx),
        );

        ui_ai::Snippet::new(children)
            .test_id(Arc::<str>::from(id))
            .into_element(cx)
    };

    let command = snippet(
        cx,
        Some("$"),
        "cargo nextest run -p fret-ui-gallery",
        "ui-ai-snippet-demo-nextest",
    );
    let plain = snippet(
        cx,
        None,
        "FRET_DIAG=1 FRET_DIAG_GPU_SCREENSHOTS=1",
        "ui-ai-snippet-demo-env",
    );

    vec![
        cx.text("Snippet (AI Elements): inline copyable surface with feedback timeout."),
        section(cx, "With Prefix", command),
        section(cx, "Without Prefix", plain),
    ]
}
