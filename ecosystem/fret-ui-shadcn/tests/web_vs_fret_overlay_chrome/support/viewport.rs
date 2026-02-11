use super::*;

pub(crate) fn bounds_for_viewport(viewport: WebViewport) -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(viewport.w), Px(viewport.h)),
    )
}

pub(crate) fn bounds_for_theme_viewport(theme: &WebGoldenTheme) -> Option<Rect> {
    let viewport = theme.viewport;
    let has_bounds =
        viewport.w.is_finite() && viewport.h.is_finite() && viewport.w > 0.0 && viewport.h > 0.0;
    if has_bounds {
        Some(bounds_for_viewport(viewport))
    } else {
        None
    }
}
