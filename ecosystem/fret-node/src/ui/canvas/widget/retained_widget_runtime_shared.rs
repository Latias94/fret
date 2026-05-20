use super::*;

pub(super) fn sync_runtime_theme<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    theme: fret_ui::ThemeSnapshot,
    services: Option<&mut dyn fret_core::UiServices>,
) {
    match services {
        Some(services) => {
            canvas.sync_style_from_color_mode(theme, Some(&mut *services));
            canvas.sync_skin(Some(&mut *services));
            canvas.sync_paint_overrides(Some(services));
        }
        None => {
            canvas.sync_style_from_color_mode(theme, None);
            canvas.sync_skin(None);
            canvas.sync_paint_overrides(None);
        }
    }
}
