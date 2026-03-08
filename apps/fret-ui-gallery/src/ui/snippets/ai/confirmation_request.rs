pub const SOURCE: &str = include_str!("confirmation_request.rs");

// region: example
use fret_ui::Theme;
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::declarative::text as decl_text;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, Space};
use fret_ui_shadcn::{ButtonVariant, prelude::*};

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let theme = Theme::global(&*cx.app).snapshot();
    let success = theme
        .color_by_key("success")
        .unwrap_or_else(|| theme.color_token("primary"));
    let destructive = theme.color_token("destructive");

    let query_block = cx.container(
        decl_style::container_props(
            &theme,
            ChromeRefinement::default()
                .rounded(Radius::Sm)
                .bg(ColorRef::Color(theme.color_token("muted")))
                .p(Space::N2),
            LayoutRefinement::default().mt(Space::N2).w_full().min_w_0(),
        ),
        move |cx| {
            vec![decl_text::text_code_wrap(
                cx,
                "SELECT * FROM users WHERE role = 'admin'",
            )]
        },
    );

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
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .min_w_0()
                .max_w(fret_core::Px(672.0)),
        )
        .children([
            ui_ai::ConfirmationTitle::new([
                ui_ai::ConfirmationRequest::new([
                    cx.text("This tool wants to execute a query on the production database:"),
                    query_block,
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
