use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::ImUiFacade;

mod layout;
mod sort;
mod z_order;

use layout::floating_layer_shell;
use sort::sort_floating_layer_windows;
use z_order::FloatWindowLayerZOrder;

#[derive(Debug, Clone, Copy)]
struct FloatWindowLayerMarker {
    layer: GlobalElementId,
}

pub(super) fn register_floating_layer_child<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    child_id: GlobalElementId,
) {
    if let Some(marker) = cx.inherited_state::<FloatWindowLayerMarker>() {
        cx.state_for(marker.layer, FloatWindowLayerZOrder::default, |st| {
            st.ensure_present(child_id);
        });
    }
}

pub(in crate::imui) fn float_layer_bring_to_front_if_activated<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    window_id: GlobalElementId,
) {
    if !cx.take_transient_for(window_id, super::KEY_FLOAT_WINDOW_ACTIVATE) {
        return;
    }
    let Some(marker) = cx.inherited_state::<FloatWindowLayerMarker>() else {
        return;
    };
    cx.state_for(marker.layer, FloatWindowLayerZOrder::default, |st| {
        st.bring_to_front(window_id);
    });
}

pub(in crate::imui) fn floating_layer_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: &str,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> AnyElement {
    cx.named(id, |cx| {
        let layer_id = cx.root_id();
        cx.state_for(
            layer_id,
            || FloatWindowLayerMarker { layer: layer_id },
            |st| st.layer = layer_id,
        );

        let mut windows: Vec<AnyElement> = Vec::new();
        {
            let mut ui = ImUiFacade {
                cx,
                out: &mut windows,
                build_focus: None,
            };
            f(&mut ui);
        }

        let z_order = cx.state_for(layer_id, FloatWindowLayerZOrder::default, |st| {
            for w in windows.iter() {
                st.ensure_present(w.id);
            }
            st.prune_missing(&windows);
            st.snapshot()
        });

        let windows_sorted = sort_floating_layer_windows(windows, &z_order);

        floating_layer_shell(cx, layer_id, windows_sorted)
    })
}
