mod apply;
mod evaluate;
mod preflight;

use crate::ui::presenter::NodeGraphPresenter;

use super::*;

pub(super) fn refresh_focused_port_hints<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
) {
    super::focus_session::clear_focused_port_hints(&mut canvas.interaction);

    let Some(input) = preflight::collect_hint_refresh_input(canvas, host) else {
        return;
    };

    let presenter: &mut dyn NodeGraphPresenter = &mut *canvas.presenter;
    let outcome = canvas
        .graph
        .read_ref(host, |graph| {
            evaluate::evaluate_focused_port_hints(presenter, graph, &input)
        })
        .ok()
        .unwrap_or_default();

    apply::apply_focused_port_hints(canvas, input.target, outcome);
}
