use super::super::super::super::*;

pub(in crate::ui) fn preview_ai_confirmation_demo(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    use std::sync::Arc;

    use fret_runtime::Model;
    use fret_ui::Invalidation;
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::{LayoutRefinement, Space};
    use fret_ui_shadcn::{Button, ButtonSize, ButtonVariant};

    #[derive(Default)]
    struct DemoModels {
        state: Option<Model<ui_ai::ToolUiPartState>>,
        approval: Option<Model<Option<ui_ai::ToolUiPartApproval>>>,
    }

    let state = cx.with_state(DemoModels::default, |st| st.state.clone());
    let state = match state {
        Some(model) => model,
        None => {
            let model = cx
                .app
                .models_mut()
                .insert(ui_ai::ToolUiPartState::InputAvailable);
            cx.with_state(DemoModels::default, |st| st.state = Some(model.clone()));
            model
        }
    };

    let approval = cx.with_state(DemoModels::default, |st| st.approval.clone());
    let approval = match approval {
        Some(model) => model,
        None => {
            let model = cx
                .app
                .models_mut()
                .insert(None::<ui_ai::ToolUiPartApproval>);
            cx.with_state(DemoModels::default, |st| st.approval = Some(model.clone()));
            model
        }
    };

    let state_now = cx
        .get_model_copied(&state, Invalidation::Layout)
        .unwrap_or(ui_ai::ToolUiPartState::InputAvailable);
    let approval_now = cx
        .get_model_cloned(&approval, Invalidation::Layout)
        .unwrap_or(None);

    let request_btn = Button::new("Request approval")
        .variant(ButtonVariant::Secondary)
        .size(ButtonSize::Sm)
        .test_id("ui-ai-confirmation-requested-btn")
        .on_activate(Arc::new({
            let state = state.clone();
            let approval = approval.clone();
            move |host, action_cx, _reason| {
                let _ = host
                    .models_mut()
                    .update(&state, |v| *v = ui_ai::ToolUiPartState::ApprovalRequested);
                let _ = host.models_mut().update(&approval, |v| {
                    *v = Some(ui_ai::ToolUiPartApproval::new("approval-1"));
                });
                host.notify(action_cx);
            }
        }))
        .into_element(cx);

    let approve_btn = Button::new("Approve")
        .variant(ButtonVariant::Default)
        .size(ButtonSize::Sm)
        .test_id("ui-ai-confirmation-approve")
        .on_activate(Arc::new({
            let state = state.clone();
            let approval = approval.clone();
            move |host, action_cx, _reason| {
                let _ = host
                    .models_mut()
                    .update(&state, |v| *v = ui_ai::ToolUiPartState::ApprovalResponded);
                let _ = host.models_mut().update(&approval, |v| {
                    if let Some(current) = v.clone() {
                        *v = Some(current.approved(true));
                    }
                });
                host.notify(action_cx);
            }
        }))
        .into_element(cx);

    let actions = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        move |_cx| vec![approve_btn],
    );

    let accepted = cx.text("Approved");

    let mut confirmation = ui_ai::Confirmation::new(state_now)
        .children([
            ui_ai::ConfirmationTitle::new("Tool approval required").into_element(cx),
            ui_ai::ConfirmationRequest::new(state_now, [actions])
                .test_id("ui-ai-confirmation-actions")
                .into_element(cx),
            ui_ai::ConfirmationAccepted::new(approval_now.clone(), state_now, [accepted])
                .test_id("ui-ai-confirmation-accepted")
                .into_element(cx),
        ])
        .test_id("ui-ai-confirmation-root");
    if let Some(approval) = approval_now {
        confirmation = confirmation.approval(approval);
    }
    let confirmation = confirmation.into_element(cx);

    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N4),
        move |cx| {
            vec![
                cx.text("Confirmation (AI Elements)"),
                cx.text("Click to request, then approve to transition to accepted state."),
                request_btn,
                confirmation,
            ]
        },
    )]
}
