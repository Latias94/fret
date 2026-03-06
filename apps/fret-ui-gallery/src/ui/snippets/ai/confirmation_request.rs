pub const SOURCE: &str = include_str!("confirmation_request.rs");

// region: example
use fret_ui::Theme;
use fret_ui_ai as ui_ai;
use fret_ui_kit::ColorRef;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::text as decl_text;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::{ButtonVariant, prelude::*};

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let theme = Theme::global(&*cx.app).snapshot();
    let success = theme
        .color_by_key("success")
        .unwrap_or_else(|| theme.color_token("primary"));
    let destructive = theme.color_token("destructive");

    let accepted = [
        decl_icon::icon_with(
            cx,
            fret_icons::IconId::new_static("lucide.check"),
            Some(fret_core::Px(16.0)),
            Some(ColorRef::Color(success)),
        ),
        cx.text("You approved this tool execution"),
    ];

    let rejected = [
        decl_icon::icon_with(
            cx,
            fret_icons::IconId::new_static("lucide.x"),
            Some(fret_core::Px(16.0)),
            Some(ColorRef::Color(destructive)),
        ),
        cx.text("You rejected this tool execution"),
    ];

    ui_ai::Confirmation::new(ui_ai::ToolUiPartState::ApprovalRequested)
        .approval(ui_ai::ToolUiPartApproval::new("approval-request"))
        .children([
            ui_ai::ConfirmationTitle::new([
                ui_ai::ConfirmationRequest::new([
                    cx.text("This tool wants to execute a query on the production database:"),
                    ui::v_flex(|cx| {
                        vec![decl_text::text_code_wrap(
                            cx,
                            "SELECT * FROM users WHERE role = 'admin'",
                        )]
                    })
                    .layout(LayoutRefinement::default().mt(Space::N2).w_full().min_w_0())
                    .into_element(cx),
                ])
                .into_element(cx),
                ui_ai::ConfirmationAccepted::new(accepted).into_element(cx),
                ui_ai::ConfirmationRejected::new(rejected).into_element(cx),
            ])
            .into_element(cx),
            ui_ai::ConfirmationActions::new([
                ui_ai::ConfirmationAction::new("Reject")
                    .variant(ButtonVariant::Outline)
                    .into_element(cx),
                ui_ai::ConfirmationAction::new("Approve")
                    .variant(ButtonVariant::Default)
                    .into_element(cx),
            ])
            .into_element(cx),
        ])
        .into_element(cx)
}
// endregion: example
