use super::super::*;
use super::finish_command_paint;

pub(super) fn cmd_open_conversion_picker<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut CommandCx<'_, H>,
    snapshot: &ViewSnapshot,
) -> bool {
    let Some(context) = canvas.interaction.last_conversion.clone() else {
        canvas.show_toast(
            cx.app,
            cx.window,
            DiagnosticSeverity::Info,
            "no recent conversion candidates",
        );
        return true;
    };

    let bounds = canvas.interaction.last_bounds.unwrap_or_default();
    let invoked_at = Point::new(Px(context.at.x), Px(context.at.y));

    canvas.dismiss_command_context_menu();
    canvas.open_searcher_overlay(
        invoked_at,
        bounds,
        snapshot,
        ContextMenuTarget::ConnectionConvertPicker {
            from: context.from,
            to: context.to,
            at: context.at,
        },
        context.candidates,
        SearcherRowsMode::Flat,
    );

    finish_command_paint(cx)
}
