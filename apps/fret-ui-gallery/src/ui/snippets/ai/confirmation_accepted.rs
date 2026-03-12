pub const SOURCE: &str = include_str!("confirmation_accepted.rs");

// region: example
use fret_ui::Theme;
use fret_ui_ai as ui_ai;
use fret_ui_kit::ColorRef;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::{Items, LayoutRefinement, Space, ui};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let theme = Theme::global(&*cx.app).snapshot();
    let success = theme
        .color_by_key("success")
        .unwrap_or_else(|| theme.color_token("primary"));
    let destructive = theme.color_token("destructive");

    let request = ui::h_row(|cx| {
        vec![
            cx.text("This tool wants to delete the file"),
            shadcn::raw::typography::inline_code(cx, "/tmp/example.txt"),
            cx.text(". Do you approve this action?"),
        ]
    })
    .gap(Space::N0p5)
    .items(Items::Center)
    .into_element(cx);

    ui_ai::Confirmation::new(ui_ai::ToolUiPartState::ApprovalResponded)
        .approval(ui_ai::ToolUiPartApproval::new("approval-accepted").approved(true))
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .min_w_0()
                .max_w(fret_core::Px(672.0)),
        )
        .children([ui_ai::ConfirmationTitle::new([
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
            .into_element(cx),
        ])
        .into_element(cx)])
        .into_element(cx)
}
// endregion: example
