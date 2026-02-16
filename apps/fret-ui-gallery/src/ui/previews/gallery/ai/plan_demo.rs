use super::super::super::super::*;

pub(in crate::ui) fn preview_ai_plan_demo(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    use fret_runtime::Model;
    use fret_ui::Invalidation;
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::{LayoutRefinement, Space};
    use fret_ui_shadcn::{Button, ButtonSize, ButtonVariant};

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

    let body = ui_ai::Plan::new()
        .open_model(open)
        .test_id_root("ui-ai-plan-root")
        .into_element_with_children(cx, move |cx, _controller| {
            let marker = is_open
                .then(|| cx.text("").test_id("ui-ai-plan-open-true"))
                .unwrap_or_else(|| cx.text(""));

            vec![
                ui_ai::PlanHeader::new([
                    ui_ai::PlanTitle::new("Plan").into_element(cx),
                    ui_ai::PlanTrigger::default()
                        .test_id("ui-ai-plan-trigger")
                        .into_element(cx),
                ])
                .into_element(cx),
                marker,
                ui_ai::PlanContent::new([
                    cx.text("1) Identify the smallest surface."),
                    cx.text("2) Land a gate (diag script)."),
                    cx.text("3) Add evidence anchors."),
                ])
                .test_id("ui-ai-plan-content-marker")
                .into_element(cx),
                ui_ai::PlanFooter::new([Button::new("Mark done")
                    .variant(ButtonVariant::Secondary)
                    .size(ButtonSize::Sm)
                    .disabled(true)
                    .into_element(cx)])
                .into_element(cx),
            ]
        });

    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N4),
        move |cx| {
            vec![
                cx.text("Plan (AI Elements)"),
                cx.text("Toggle the chevron button to expand/collapse."),
                body,
            ]
        },
    )]
}
