use super::super::super::super::*;

pub(in crate::ui) fn preview_ai_chain_of_thought_demo(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    use fret_runtime::Model;
    use fret_ui::Invalidation;
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::{LayoutRefinement, Space};

    #[derive(Default)]
    struct DemoModels {
        open: Option<Model<bool>>,
    }

    let open = cx.with_state(DemoModels::default, |st| st.open.clone());
    let open = match open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(DemoModels::default, |st| st.open = Some(model.clone()));
            model
        }
    };

    let is_open = cx
        .get_model_copied(&open, Invalidation::Layout)
        .unwrap_or(false);

    let shell = ui_ai::ChainOfThought::new()
        .open_model(open)
        .test_id_root("ui-ai-chain-of-thought-root")
        .into_element_with_children(cx, move |cx| {
            let mut children = vec![
                ui_ai::ChainOfThoughtHeader::new()
                    .test_id("ui-ai-chain-of-thought-header")
                    .into_element(cx),
            ];

            if is_open {
                children.push(cx.text("").test_id("ui-ai-chain-of-thought-open-true"));
            }

            children.push(
                ui_ai::ChainOfThoughtContent::new([
                    ui_ai::ChainOfThoughtStep::new("Collect context")
                        .status(ui_ai::ChainOfThoughtStepStatus::Complete)
                        .children([cx.text("Parse prompt and attachments.")])
                        .into_element(cx),
                    ui_ai::ChainOfThoughtStep::new("Plan")
                        .status(ui_ai::ChainOfThoughtStepStatus::Active)
                        .children([cx.text("Draft an execution plan.")])
                        .into_element(cx),
                    ui_ai::ChainOfThoughtStep::new("Execute")
                        .status(ui_ai::ChainOfThoughtStepStatus::Pending)
                        .children([cx.text("Run tool calls and stream output.")])
                        .into_element(cx),
                ])
                .test_id("ui-ai-chain-of-thought-content-marker")
                .into_element(cx),
            );

            children
        });

    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N4),
        move |cx| {
            vec![
                cx.text("Chain of Thought (AI Elements)"),
                cx.text("Click the header to toggle the disclosure."),
                shell,
            ]
        },
    )]
}
