use super::*;

pub(crate) fn bounds_for_viewport(viewport: WebViewport) -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(viewport.w), Px(viewport.h)),
    )
}
