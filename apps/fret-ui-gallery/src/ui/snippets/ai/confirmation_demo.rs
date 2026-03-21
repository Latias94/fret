pub const SOURCE: &str = include_str!("confirmation_demo.rs");

// region: example
use fret::app::UiCxActionsExt as _;
use fret::{UiChild, UiCx};
use fret_ui::Invalidation;
use fret_ui::Theme;
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::ui;
use fret_ui_kit::ColorRef;
use fret_ui_kit::{Items, LayoutRefinement, Space};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

mod act {
    fret::actions!([
        RequestApproval = "ui-gallery.ai.confirmation.request_approval.v1",
        RejectApproval = "ui-gallery.ai.confirmation.reject_approval.v1",
        ApproveApproval = "ui-gallery.ai.confirmation.approve_approval.v1",
    ]);
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let state = cx.local_model_keyed("ui-gallery-ai-confirmation-demo-state", || {
        ui_ai::ToolUiPartState::InputAvailable
    });
    let approval = cx.local_model_keyed("ui-gallery-ai-confirmation-demo-approval", || {
        None::<ui_ai::ToolUiPartApproval>
    });

    ui::v_flex(move |cx| {
        cx.actions().models::<act::RequestApproval>({
            let state = state.clone();
            let approval = approval.clone();
            move |models| {
                let state_updated = models
                    .update(&state, |value| {
                        *value = ui_ai::ToolUiPartState::ApprovalRequested
                    })
                    .is_ok();
                let approval_updated = models
                    .update(&approval, |value| {
                        *value = Some(ui_ai::ToolUiPartApproval::new("approval-1"));
                    })
                    .is_ok();
                state_updated && approval_updated
            }
        });

        cx.actions().models::<act::RejectApproval>({
            let state = state.clone();
            let approval = approval.clone();
            move |models| {
                let state_updated = models
                    .update(&state, |value| {
                        *value = ui_ai::ToolUiPartState::OutputDenied
                    })
                    .is_ok();
                let approval_updated = models
                    .update(&approval, |value| {
                        if let Some(current) = value.clone() {
                            *value = Some(current.approved(false));
                        }
                    })
                    .is_ok();
                state_updated && approval_updated
            }
        });

        cx.actions().models::<act::ApproveApproval>({
            let state = state.clone();
            let approval = approval.clone();
            move |models| {
                let state_updated = models
                    .update(&state, |value| {
                        *value = ui_ai::ToolUiPartState::OutputAvailable
                    })
                    .is_ok();
                let approval_updated = models
                    .update(&approval, |value| {
                        if let Some(current) = value.clone() {
                            *value = Some(current.approved(true));
                        }
                    })
                    .is_ok();
                state_updated && approval_updated
            }
        });

        let state_now = cx
            .get_model_copied(&state, Invalidation::Layout)
            .unwrap_or(ui_ai::ToolUiPartState::InputAvailable);
        let approval_now = cx
            .get_model_cloned(&approval, Invalidation::Layout)
            .unwrap_or(None);

        let theme = Theme::global(&*cx.app).snapshot();
        let success = theme
            .color_by_key("success")
            .unwrap_or_else(|| theme.color_token("primary"));
        let destructive = theme.color_token("destructive");

        let request_btn = shadcn::Button::new("Delete Example File")
            .variant(shadcn::ButtonVariant::Secondary)
            .size(shadcn::ButtonSize::Sm)
            .test_id("ui-ai-confirmation-requested-btn")
            .action(act::RequestApproval)
            .into_element(cx);

        let reject_btn = ui_ai::ConfirmationAction::new("Reject")
            .variant(shadcn::ButtonVariant::Outline)
            .test_id("ui-ai-confirmation-reject")
            .action(act::RejectApproval)
            .into_element(cx);

        let approve_btn = ui_ai::ConfirmationAction::new("Approve")
            .variant(shadcn::ButtonVariant::Default)
            .test_id("ui-ai-confirmation-approve")
            .action(act::ApproveApproval)
            .into_element(cx);

        let request = ui::h_row(|cx| {
            vec![
                cx.text("This tool wants to delete the file"),
                shadcn::raw::typography::inline_code("/tmp/example.txt").into_element(cx),
                cx.text(". Do you approve this action?"),
            ]
        })
        .gap(Space::N1)
        .items(Items::Center)
        .into_element(cx);

        let confirmation = match approval_now {
            Some(approval) => ui_ai::Confirmation::new(state_now)
                .approval(approval)
                .test_id("ui-ai-confirmation-root")
                .refine_layout(LayoutRefinement::default().w_full().min_w_0())
                .children([
                    ui_ai::ConfirmationRequest::new([request])
                        .test_id("ui-ai-confirmation-request")
                        .into_element(cx),
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
                    ui_ai::ConfirmationActions::new([reject_btn, approve_btn])
                        .test_id("ui-ai-confirmation-actions")
                        .into_element(cx),
                ])
                .into_element(cx),
            None => ui_ai::Confirmation::new(state_now)
                .test_id("ui-ai-confirmation-root")
                .into_element(cx),
        };

        let result = match state_now {
            ui_ai::ToolUiPartState::OutputAvailable => Some(
                ui_ai::MessageResponse::new("Successfully deleted /tmp/example.txt")
                    .streaming(false)
                    .test_id_prefix("ui-ai-confirmation-demo-output-")
                    .into_element(cx),
            ),
            ui_ai::ToolUiPartState::OutputDenied => Some(
                ui_ai::MessageResponse::new("Deletion cancelled because approval was rejected.")
                    .streaming(false)
                    .test_id_prefix("ui-ai-confirmation-demo-output-")
                    .into_element(cx),
            ),
            _ => None,
        };

        let mut out = vec![
            cx.text("Confirmation (AI Elements)"),
            cx.text(
                "Docs-aligned tool approval workflow: request a destructive action, confirm or reject it, then inspect the final output state.",
            ),
            request_btn,
            confirmation,
        ];
        if let Some(result) = result {
            out.push(result);
        }
        out
    })
    .layout(
        LayoutRefinement::default()
            .w_full()
            .min_w_0()
            .max_w(fret_core::Px(672.0)),
    )
    .gap(Space::N4)
    .into_element(cx)
}
// endregion: example
