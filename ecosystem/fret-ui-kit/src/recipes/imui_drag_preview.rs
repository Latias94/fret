//! Immediate drag preview helpers built on top of the typed `imui` drag source seam.
//!
//! This module intentionally lives in `recipes`, not in `imui` itself:
//! - `fret-ui-kit::imui` owns typed drag/drop publication and readout,
//! - this module owns source-side preview presentation policy,
//! - and app code still authors the actual preview content.

use std::collections::HashMap;
use std::sync::Arc;

use fret_core::{Point, PointerId, Px};
use fret_runtime::{DragSessionId, FrameId, Model};
use fret_ui::element::{
    AnyElement, ContainerProps, InsetStyle, LayoutStyle, Length, Overflow, PositionStyle, SizeStyle,
};
use fret_ui::{ElementContext, UiHost};

use crate::IntoUiElement;
use crate::OverlayPresence;
use crate::declarative::ModelWatchExt;
use crate::imui::{DragSourceResponse, UiWriterImUiFacadeExt};
use crate::primitives::tooltip as radix_tooltip;

type CrossWindowDragPreviewRenderer<H> =
    Arc<dyn for<'a> Fn(&mut ElementContext<'a, H>) -> AnyElement + 'static>;

struct CrossWindowDragPreviewGhostDescriptor<H: UiHost> {
    id: Arc<str>,
    pointer_id: PointerId,
    session_id: DragSessionId,
    stale_frame: Option<FrameId>,
    options: DragPreviewGhostOptions,
    render: CrossWindowDragPreviewRenderer<H>,
}

struct CrossWindowDragPreviewGhostStore<H: UiHost> {
    descriptors: HashMap<DragSessionId, CrossWindowDragPreviewGhostDescriptor<H>>,
}

impl<H: UiHost> Default for CrossWindowDragPreviewGhostStore<H> {
    fn default() -> Self {
        Self {
            descriptors: HashMap::new(),
        }
    }
}

struct CrossWindowDragPreviewGhostStoreGlobal<H: UiHost> {
    model: Option<Model<CrossWindowDragPreviewGhostStore<H>>>,
}

impl<H: UiHost> Default for CrossWindowDragPreviewGhostStoreGlobal<H> {
    fn default() -> Self {
        Self { model: None }
    }
}

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

fn cross_window_drag_preview_store_model<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
) -> Model<CrossWindowDragPreviewGhostStore<H>> {
    cx.app.with_global_mut_untracked(
        CrossWindowDragPreviewGhostStoreGlobal::<H>::default,
        |st, app| {
            if let Some(model) = st.model.clone() {
                return model;
            }

            let model = app
                .models_mut()
                .insert(CrossWindowDragPreviewGhostStore::default());
            st.model = Some(model.clone());
            model
        },
    )
}

fn remove_cross_window_drag_preview_sessions<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    store: &Model<CrossWindowDragPreviewGhostStore<H>>,
    session_ids: &[DragSessionId],
) {
    if session_ids.is_empty() {
        return;
    }

    let _ = cx.app.models_mut().update(store, |st| {
        for session_id in session_ids {
            st.descriptors.remove(session_id);
        }
    });
}

fn sync_drag_preview_ghost_overlay<H: UiHost>(
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

/// Publishes a cross-window drag preview descriptor for later window-root rendering.
///
/// Call this from the drag source site every frame while the source is authored. Then call
/// [`render_cross_window_drag_preview_ghosts`] once near the root of each participating window.
pub fn publish_cross_window_drag_preview_ghost<H, W, F, P>(
    ui: &mut W,
    id: &str,
    source: DragSourceResponse,
    preview: F,
) -> bool
where
    H: UiHost + 'static,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
    F: for<'a> Fn(&mut ElementContext<'a, H>) -> P + 'static,
    P: IntoUiElement<H>,
{
    publish_cross_window_drag_preview_ghost_with_options(
        ui,
        id,
        source,
        DragPreviewGhostOptions::default(),
        preview,
    )
}

/// Publishes a cross-window drag preview descriptor for later window-root rendering.
///
/// This recipe-level helper intentionally keeps the shell choreography out of `imui`:
/// - the source publishes preview intent + renderer once a drag session exists,
/// - the active `current_window` becomes the only paint owner,
/// - and the preview content remains entirely app-authored.
pub fn publish_cross_window_drag_preview_ghost_with_options<H, W, F, P>(
    ui: &mut W,
    id: &str,
    source: DragSourceResponse,
    options: DragPreviewGhostOptions,
    preview: F,
) -> bool
where
    H: UiHost + 'static,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
    F: for<'a> Fn(&mut ElementContext<'a, H>) -> P + 'static,
    P: IntoUiElement<H>,
{
    ui.with_cx_mut(|cx| {
        let store = cross_window_drag_preview_store_model(cx);

        let Some(session_id) = source.session_id() else {
            return false;
        };
        let Some(pointer_id) = source.pointer_id() else {
            return false;
        };
        if cx
            .app
            .drag(pointer_id)
            .filter(|drag| drag.session_id == session_id && drag.dragging)
            .is_none()
        {
            return false;
        }

        let render: CrossWindowDragPreviewRenderer<H> = Arc::new(move |cx| {
            let preview = preview(cx);
            IntoUiElement::into_element(preview, cx)
        });
        let enabled = options.enabled;

        let descriptor = CrossWindowDragPreviewGhostDescriptor {
            id: Arc::from(id),
            pointer_id,
            session_id,
            stale_frame: None,
            options,
            render,
        };
        let _ = cx.app.models_mut().update(&store, |st| {
            st.descriptors.insert(session_id, descriptor);
        });
        enabled && source.active()
    })
}

/// Renders any published cross-window drag preview ghosts for the current window.
///
/// Contract:
/// - call once per window root,
/// - only `drag.current_window` paints a given ghost,
/// - stale descriptors are pruned as soon as the drag session disappears.
pub fn render_cross_window_drag_preview_ghosts<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
) -> bool {
    let store = cross_window_drag_preview_store_model(cx);

    let descriptors = cx
        .app
        .models()
        .read(&store, |st| {
            st.descriptors
                .values()
                .map(|descriptor| {
                    (
                        descriptor.id.clone(),
                        descriptor.pointer_id,
                        descriptor.session_id,
                        descriptor.stale_frame,
                        descriptor.options.clone(),
                        descriptor.render.clone(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let mut rendered_any = false;
    let current_frame = cx.app.frame_id();
    let mut mark_stale = Vec::new();
    let mut stale_sessions = Vec::new();
    for (id, pointer_id, session_id, stale_frame, options, render) in descriptors {
        let drag = cx
            .app
            .drag(pointer_id)
            .filter(|drag| drag.session_id == session_id && drag.dragging);
        let origin = drag
            .filter(|drag| options.enabled && drag.current_window == cx.window)
            .map(|drag| {
                Point::new(
                    drag.position.x + options.offset.x,
                    drag.position.y + options.offset.y,
                )
            });
        if drag.is_none() && stale_frame.is_none() {
            mark_stale.push(session_id);
        }
        if drag.is_none() && stale_frame.is_some_and(|frame| frame != current_frame) {
            stale_sessions.push(session_id);
        }
        let overlay_key = format!(
            "fret-ui-kit.imui.drag-preview.cross-window.overlay.{id}.{}",
            session_id.0
        );
        rendered_any |= sync_drag_preview_ghost_overlay(
            cx,
            overlay_key.as_str(),
            origin,
            &options,
            move |cx| render(cx),
        );
    }
    if !mark_stale.is_empty() {
        let _ = cx.app.models_mut().update(&store, |st| {
            for session_id in &mark_stale {
                if let Some(descriptor) = st.descriptors.get_mut(session_id)
                    && descriptor.stale_frame.is_none()
                {
                    descriptor.stale_frame = Some(current_frame);
                }
            }
        });
    }
    remove_cross_window_drag_preview_sessions(cx, &store, &stale_sessions);

    rendered_any
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
            ghost_anchor_position(
                DragSourceResponse {
                    active: false,
                    cross_window: false,
                    position: Some(position),
                    pointer_id: None,
                    session_id: None,
                },
                &options,
            ),
            None
        );
        assert_eq!(
            ghost_anchor_position(
                DragSourceResponse {
                    active: true,
                    cross_window: true,
                    position: Some(position),
                    pointer_id: None,
                    session_id: None,
                },
                &options,
            ),
            None
        );
        assert_eq!(
            ghost_anchor_position(
                DragSourceResponse {
                    active: true,
                    cross_window: false,
                    position: None,
                    pointer_id: None,
                    session_id: None,
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
                DragSourceResponse {
                    active: true,
                    cross_window: false,
                    position: Some(Point::new(Px(120.0), Px(40.0))),
                    pointer_id: None,
                    session_id: None,
                },
                &options,
            ),
            Some(Point::new(Px(138.0), Px(34.0)))
        );
    }
}
