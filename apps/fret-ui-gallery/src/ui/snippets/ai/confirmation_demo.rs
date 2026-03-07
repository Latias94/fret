pub const SOURCE: &str = include_str!("confirmation_demo.rs");

// region: example
use fret_runtime::Model;
use fret_ui::Invalidation;
use fret_ui::Theme;
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::ui;
use fret_ui_kit::ColorRef;
use fret_ui_kit::{Items, LayoutRefinement, Space};
use fret_ui_shadcn::{self as shadcn, prelude::*, Button, ButtonSize, ButtonVariant};
use std::sync::Arc;

#[derive(Default)]
struct DemoModels {
    state: Option<Model<ui_ai::ToolUiPartState>>,
    approval: Option<Model<Option<ui_ai::ToolUiPartApproval>>>,
}

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
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

    let reject_btn = ui_ai::ConfirmationAction::new("Reject")
        .variant(ButtonVariant::Outline)
        .test_id("ui-ai-confirmation-reject")
        .on_activate(Arc::new({
            let state = state.clone();
            let approval = approval.clone();
            move |host, action_cx, _reason| {
                let _ = host
                    .models_mut()
                    .update(&state, |v| *v = ui_ai::ToolUiPartState::OutputDenied);
                let _ = host.models_mut().update(&approval, |v| {
                    if let Some(current) = v.clone() {
                        *v = Some(current.approved(false));
                    }
                });
                host.notify(action_cx);
            }
        }))
        .into_element(cx);

    let approve_btn = ui_ai::ConfirmationAction::new("Approve")
        .variant(ButtonVariant::Default)
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

    let confirmation = match approval_now {
        Some(approval) => {
            let theme = Theme::global(&*cx.app).snapshot();
            let success = theme
                .color_by_key("success")
                .unwrap_or_else(|| theme.color_token("primary"));
            let destructive = theme.color_token("destructive");

            let request = ui::h_row(|cx| {
                vec![
                    cx.text("This tool wants to delete the file"),
                    shadcn::typography::inline_code(cx, "/tmp/example.txt"),
                    cx.text(". Do you approve this action?"),
                ]
            })
            .gap(Space::N1)
            .items(Items::Center)
            .into_element(cx);

            ui_ai::Confirmation::new(state_now)
                .approval(approval)
                .test_id("ui-ai-confirmation-root")
                .refine_layout(LayoutRefinement::default().w_full().min_w_0())
                .children([
                    ui_ai::ConfirmationTitle::new([
                        ui_ai::ConfirmationRequest::new([request]).into_element(cx),
                        ui_ai::ConfirmationAccepted::new([
                            decl_icon::icon_with(
                                cx,
                                fret_icons::IconId::new_static("lucide.check"),
                                Some(fret_core::Px(16.0)),
                                Some(ColorRef::Color(success)),
                            ),
                            cx.text("You approved this tool execution"),
                        ])
                        .test_id("ui-ai-confirmation-accepted")
                        .into_element(cx),
                        ui_ai::ConfirmationRejected::new([
                            decl_icon::icon_with(
                                cx,
                                fret_icons::IconId::new_static("lucide.x"),
                                Some(fret_core::Px(16.0)),
                                Some(ColorRef::Color(destructive)),
                            ),
                            cx.text("You rejected this tool execution"),
                        ])
                        .test_id("ui-ai-confirmation-rejected")
                        .into_element(cx),
                    ])
                    .into_element(cx),
                    ui_ai::ConfirmationActions::new([reject_btn, approve_btn])
                        .test_id("ui-ai-confirmation-actions")
                        .into_element(cx),
                ])
                .into_element(cx)
        }
        None => ui_ai::Confirmation::new(state_now)
            .test_id("ui-ai-confirmation-root")
            .into_element(cx),
    };

    ui::v_flex(move |cx| {
        vec![
            cx.text("Confirmation (AI Elements)"),
            cx.text(
                "Click to request, then approve or reject to transition between the canonical states.",
            ),
            request_btn,
            confirmation,
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0().max_w(fret_core::Px(672.0)))
    .gap(Space::N4)
    .into_element(cx)
}
// endregion: example
