use super::super::*;
use super::finish_command_paint;

pub(super) fn cmd_open_conversion_picker<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut CommandCx<'_, H>,
    _snapshot: &ViewSnapshot,
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

    super::super::searcher_picker::open_searcher_picker_request(
        canvas,
        cx.app,
        super::super::searcher_picker::conversion_searcher_picker_request(
            context.from,
            context.to,
            context.at,
            context.candidates,
        ),
    );

    finish_command_paint(cx)
}
