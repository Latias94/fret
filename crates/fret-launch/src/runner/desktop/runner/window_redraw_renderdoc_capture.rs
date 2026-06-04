use super::RenderDocCapture;

pub(super) fn begin_window_redraw_renderdoc_capture(
    renderdoc: Option<&mut RenderDocCapture>,
) -> bool {
    renderdoc.is_some_and(|capture| capture.begin_capture_if_requested())
}

pub(super) fn end_window_redraw_renderdoc_capture(
    renderdoc: Option<&mut RenderDocCapture>,
    capturing: bool,
) {
    if capturing && let Some(capture) = renderdoc {
        capture.end_capture();
    }
}
