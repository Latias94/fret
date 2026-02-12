use fret_app::{App, Model};
use fret_core::{AppWindowId, Point};
use fret_ui::action::{ActionCx, PointerDownCx, PointerMoveCx, UiPointerActionHost};
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Invalidation, UiTree};
use std::sync::Arc;

use super::status_bar;

pub(super) fn wrap_content_if_enabled(
    cx: &mut ElementContext<'_, App>,
    inspector_enabled: &Model<bool>,
    inspector_last_pointer: &Model<Option<Point>>,
    content: Vec<AnyElement>,
) -> Vec<AnyElement> {
    if !cx
        .get_model_copied(inspector_enabled, Invalidation::Layout)
        .unwrap_or(false)
    {
        return content;
    }

    cx.observe_model(inspector_last_pointer, Invalidation::Paint);

    let mut props = fret_ui::element::PointerRegionProps::default();
    props.layout.size.width = fret_ui::element::Length::Fill;
    props.layout.size.height = fret_ui::element::Length::Fill;

    let on_pointer_move = {
        let inspector_last_pointer = inspector_last_pointer.clone();
        Arc::new(
            move |host: &mut dyn UiPointerActionHost, cx: ActionCx, mv: PointerMoveCx| {
                let _ = host.models_mut().update(&inspector_last_pointer, |v| {
                    *v = Some(mv.position);
                });
                host.request_redraw(cx.window);
                false
            },
        )
    };
    let on_pointer_down = {
        let inspector_last_pointer = inspector_last_pointer.clone();
        Arc::new(
            move |host: &mut dyn UiPointerActionHost, cx: ActionCx, down: PointerDownCx| {
                let _ = host.models_mut().update(&inspector_last_pointer, |v| {
                    *v = Some(down.position);
                });
                host.request_redraw(cx.window);
                false
            },
        )
    };

    vec![cx.pointer_region(props, move |cx| {
        cx.pointer_region_on_pointer_move(on_pointer_move);
        cx.pointer_region_on_pointer_down(on_pointer_down);
        content
    })]
}

pub(super) fn compute_inspector_status(
    app: &mut App,
    ui: &UiTree<App>,
    window: AppWindowId,
    pointer: Option<Point>,
) -> status_bar::InspectorStatus {
    let hit = pointer.map(|p| ui.debug_hit_test(p));
    let hit_node = hit.as_ref().and_then(|h| h.hit);
    let hit_layers = hit
        .as_ref()
        .map(|h| h.active_layer_roots.len())
        .unwrap_or(0);
    let hit_barrier = hit.as_ref().and_then(|h| h.barrier_root);

    let (focused_node, focused_element, hovered_pressable, pressed_pressable) = app
        .with_global_mut_untracked(fret_ui::ElementRuntime::new, |runtime, _| {
            let state = runtime.diagnostics_snapshot(window);
            (
                ui.focus(),
                state.as_ref().and_then(|s| s.focused_element),
                state.as_ref().and_then(|s| s.hovered_pressable),
                state.as_ref().and_then(|s| s.pressed_pressable),
            )
        });

    let hit_element = hit_node.and_then(|node| {
        app.with_global_mut_untracked(fret_ui::ElementRuntime::new, |runtime, _| {
            runtime.element_for_node(window, node)
        })
    });

    let hit_path = hit_element.and_then(|element| {
        app.with_global_mut_untracked(fret_ui::ElementRuntime::new, |runtime, _| {
            runtime.debug_path_for_element(window, element)
        })
    });
    let focused_path = focused_element.and_then(|element| {
        app.with_global_mut_untracked(fret_ui::ElementRuntime::new, |runtime, _| {
            runtime.debug_path_for_element(window, element)
        })
    });

    let cursor = if let Some(pos) = pointer {
        Arc::<str>::from(format!("cursor=({:.1},{:.1})", pos.x.0, pos.y.0))
    } else {
        Arc::<str>::from("cursor=<none>")
    };

    let hit = Arc::<str>::from(format!(
        "hit={:?} el={} layers={} barrier={:?} {}",
        hit_node,
        hit_element
            .map(|id| format!("{:#x}", id.0))
            .unwrap_or_else(|| "<none>".to_string()),
        hit_layers,
        hit_barrier,
        hit_path.as_deref().unwrap_or(""),
    ));

    let focus = Arc::<str>::from(format!(
        "focus={:?} el={} hovered={} pressed={} {}",
        focused_node,
        focused_element
            .map(|id| format!("{:#x}", id.0))
            .unwrap_or_else(|| "<none>".to_string()),
        hovered_pressable
            .map(|id| format!("{:#x}", id.0))
            .unwrap_or_else(|| "<none>".to_string()),
        pressed_pressable
            .map(|id| format!("{:#x}", id.0))
            .unwrap_or_else(|| "<none>".to_string()),
        focused_path.as_deref().unwrap_or(""),
    ));

    let text = if let Some(node) = hit_node {
        let bounds = ui.debug_node_bounds(node);
        let constraints = ui.debug_text_constraints_snapshot(node);
        Arc::<str>::from(format!(
            "text node={:?} bounds={bounds:?} measured={:?} prepared={:?}",
            node, constraints.measured, constraints.prepared,
        ))
    } else {
        Arc::<str>::from("text node=<none>")
    };

    (cursor, hit, focus, text)
}
