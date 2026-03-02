pub const SOURCE: &str = include_str!("stack_trace_large_demo.rs");

// region: example
use fret_runtime::Model;
use fret_ui::element::SemanticsProps;
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::prelude::*;
use std::sync::Arc;

#[derive(Default)]
struct DemoModels {
    opened: Option<Model<bool>>,
}

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let opened = cx.with_state(DemoModels::default, |st| st.opened.clone());
    let opened = match opened {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(DemoModels::default, |st| st.opened = Some(model.clone()));
            model
        }
    };

    let opened_for_marker = opened.clone();
    let marker = cx.semantics(
        SemanticsProps {
            role: fret_core::SemanticsRole::Text,
            test_id: Some(Arc::<str>::from("ui-ai-stack-trace-large-opened-marker")),
            ..Default::default()
        },
        move |cx| {
            let opened = cx
                .app
                .models()
                .read(&opened_for_marker, |v| *v)
                .unwrap_or(false);
            if opened {
                vec![cx.text("")]
            } else {
                Vec::new()
            }
        },
    );

    let mut trace = String::new();
    trace.push_str("Error: synthetic large stack\n");
    for index in 0..220usize {
        trace.push_str(&format!(
            "    at f{index} (src/module_{index:04}.rs:{line}:{col})\n",
            line = 10 + (index % 97),
            col = 1 + (index % 13)
        ));
    }

    let stack = ui_ai::StackTrace::new(trace)
        .default_open(false)
        .test_id_root("ui-ai-stack-trace-large-root")
        .test_id_header_trigger("ui-ai-stack-trace-large-header")
        .test_id_content("ui-ai-stack-trace-large-content")
        .test_id_frames_viewport("ui-ai-stack-trace-large-frames-viewport")
        .frame_test_id_prefix("ui-ai-stack-trace-large-frame")
        .on_file_path_click(Arc::new({
            let opened = opened.clone();
            move |host, _action_cx, _path, _line, _col| {
                let _ = host.models_mut().update(&opened, |v| *v = true);
            }
        }))
        .into_element(cx);

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N4),
        move |cx| {
            vec![
                cx.text("StackTrace (Large)"),
                cx.text("Scroll in the frames viewport and click a file path."),
                stack,
                marker,
            ]
        },
    )
}
// endregion: example

