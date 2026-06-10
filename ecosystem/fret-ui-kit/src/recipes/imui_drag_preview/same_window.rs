use std::sync::Arc;

#[cfg(test)]
use fret_core::PointerId;
use fret_core::{Point, Px};
#[cfg(test)]
use fret_runtime::DragSessionId;
use fret_ui::element::{
    AnyElement, ContainerProps, InsetStyle, LayoutStyle, Length, Overflow, PositionStyle, SizeStyle,
};
use fret_ui::{ElementContext, UiHost};

use crate::IntoUiElement;
use crate::OverlayPresence;
use crate::declarative::ModelWatchExt;
use crate::imui::{DragSourceResponse, UiWriterImUiFacadeExt};
use crate::primitives::tooltip as radix_tooltip;

/// Recipe-level options for an immediate drag preview ghost.
#[derive(Debug, Clone)]
pub struct DragPreviewGhostOptions {
    pub enabled: bool,
    pub offset: Point,
    pub opacity: f32,
    pub test_id: Option<Arc<str>>,
}

impl Default for DragPreviewGhostOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            offset: Point::new(Px(12.0), Px(12.0)),
            opacity: 0.9,
            test_id: None,
        }
    }
}

pub(super) fn sync_drag_preview_ghost_overlay<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    overlay_key: &str,
    origin: Option<Point>,
    options: &DragPreviewGhostOptions,
    preview: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
) -> bool {
    cx.named(overlay_key, |cx| {
        let overlay_id = cx.root_id();
        let open = cx.local_model_keyed("open", || false);
        let show = options.enabled && origin.is_some();
        let open_now = cx.watch_model(&open).layout().copied().unwrap_or(false);
        if open_now != show {
            let _ = cx.app.models_mut().update(&open, |value| *value = show);
        }

        let Some(origin) = origin.filter(|_| options.enabled) else {
            return false;
        };

        let panel_test_id = options.test_id.clone();
        let opacity = options.opacity;
        let preview = preview(cx);
        let preview = if let Some(test_id) = panel_test_id {
            preview.test_id(test_id)
        } else {
            preview
        };

        let overlay_children = vec![cx.named("panel", move |cx| {
            let mut panel_props = ContainerProps::default();
            panel_props.layout = LayoutStyle {
                position: PositionStyle::Absolute,
                inset: InsetStyle {
                    left: Some(origin.x).into(),
                    top: Some(origin.y).into(),
                    ..Default::default()
                },
                size: SizeStyle {
                    width: Length::Auto,
                    height: Length::Auto,
                    ..Default::default()
                },
                overflow: Overflow::Visible,
                ..Default::default()
            };

            cx.container(panel_props, move |cx| {
                vec![cx.opacity(opacity, move |_cx| vec![preview])]
            })
        })];

        let request = radix_tooltip::tooltip_request(
            overlay_id,
            open,
            OverlayPresence {
                present: true,
                interactive: false,
            },
            overlay_children,
        );
        radix_tooltip::request_tooltip(cx, request);
        true
    })
}

/// Requests a same-window, click-through drag preview ghost in the tooltip overlay layer.
pub fn drag_preview_ghost<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized, P: IntoUiElement<H>>(
    ui: &mut W,
    id: &str,
    source: DragSourceResponse,
    preview: P,
) -> bool {
    drag_preview_ghost_with_options(ui, id, source, DragPreviewGhostOptions::default(), preview)
}

/// Requests a same-window, click-through drag preview ghost in the tooltip overlay layer.
///
/// The preview is only shown when:
/// - the source is active,
/// - the drag has not entered cross-window hover mode,
/// - and the source published a pointer position for the current drag frame.
pub fn drag_preview_ghost_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
    P: IntoUiElement<H>,
>(
    ui: &mut W,
    id: &str,
    source: DragSourceResponse,
    options: DragPreviewGhostOptions,
    preview: P,
) -> bool {
    ui.with_cx_mut(|cx| {
        let Some(origin) = ghost_anchor_position(source, &options) else {
            let overlay_key = format!("fret-ui-kit.imui.drag-preview.overlay.{id}");
            return sync_drag_preview_ghost_overlay(
                cx,
                overlay_key.as_str(),
                None,
                &options,
                |_cx| unreachable!("same-window ghost should not build preview when hidden"),
            );
        };

        let overlay_key = format!("fret-ui-kit.imui.drag-preview.overlay.{id}");
        sync_drag_preview_ghost_overlay(
            cx,
            overlay_key.as_str(),
            Some(origin),
            &options,
            move |cx| IntoUiElement::into_element(preview, cx),
        )
    })
}

fn ghost_anchor_position(
    source: DragSourceResponse,
    options: &DragPreviewGhostOptions,
) -> Option<Point> {
    if !options.enabled || !source.active() || source.cross_window() {
        return None;
    }

    let position = source.position()?;
    Some(Point::new(
        position.x + options.offset.x,
        position.y + options.offset.y,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_options_are_same_window_friendly() {
        let options = DragPreviewGhostOptions::default();
        assert!(options.enabled);
        assert_eq!(options.offset, Point::new(Px(12.0), Px(12.0)));
        assert!((options.opacity - 0.9).abs() <= f32::EPSILON);
        assert!(options.test_id.is_none());
    }

    #[test]
    fn ghost_anchor_requires_active_same_window_source_with_position() {
        let options = DragPreviewGhostOptions::default();
        let position = Point::new(Px(40.0), Px(24.0));

        assert_eq!(
            ghost_anchor_position(DragSourceResponse::inactive(), &options,),
            None
        );
        assert_eq!(
            ghost_anchor_position(
                DragSourceResponse::new(true, position, PointerId(1), DragSessionId(1),),
                &options,
            ),
            None
        );
        assert_eq!(
            ghost_anchor_position(
                DragSourceResponse {
                    position: None,
                    ..DragSourceResponse::new(false, position, PointerId(1), DragSessionId(1))
                },
                &options,
            ),
            None
        );
    }

    #[test]
    fn ghost_anchor_applies_offset_to_pointer_position() {
        let options = DragPreviewGhostOptions {
            offset: Point::new(Px(18.0), Px(-6.0)),
            ..Default::default()
        };

        assert_eq!(
            ghost_anchor_position(
                DragSourceResponse::new(
                    false,
                    Point::new(Px(120.0), Px(40.0)),
                    PointerId(1),
                    DragSessionId(1),
                ),
                &options,
            ),
            Some(Point::new(Px(138.0), Px(34.0)))
        );
    }
}
