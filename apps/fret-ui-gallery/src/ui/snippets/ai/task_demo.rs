pub const SOURCE: &str = include_str!("task_demo.rs");

// region: example
use fret_runtime::Model;
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::prelude::*;

#[derive(Default)]
struct DemoModels {
    open: Option<Model<bool>>,
}

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let open = cx.with_state(DemoModels::default, |st| st.open.clone());
    let open = match open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(DemoModels::default, |st| st.open = Some(model.clone()));
            model
        }
    };

    let trigger = ui_ai::TaskTrigger::new("Indexing workspace")
        .children([cx.text("Click to expand")])
        .test_id("ui-ai-task-demo-trigger");

    let content = ui_ai::TaskContent::new([
        cx.text("Task content is app-owned; this is the collapsible chrome."),
        cx.text("• step 1: scan"),
        cx.text("• step 2: parse"),
        cx.text("• step 3: index"),
    ])
    .test_id("ui-ai-task-demo-content");

    let task = ui_ai::Task::new(trigger, content)
        .open_model(open)
        .default_open(false)
        .into_element(cx);

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N4),
        move |cx| {
            vec![
                cx.text("Task (AI Elements)"),
                cx.text("Collapsible task surface (trigger + content)."),
                task,
            ]
        },
    )
}
// endregion: example

