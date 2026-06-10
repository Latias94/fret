use std::collections::HashMap;
use std::sync::Arc;

use fret_core::{Point, PointerId};
use fret_runtime::{DragSessionId, FrameId, Model};
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use crate::IntoUiElement;
use crate::imui::{DragSourceResponse, UiWriterImUiFacadeExt};

use super::same_window::{DragPreviewGhostOptions, sync_drag_preview_ghost_overlay};

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
