use fret_core::Px;
use fret_ui::element::{AnyElement, ContainerProps, InsetStyle, Length, Overflow, PositionStyle};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::ImUiFacade;

mod z_order;

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

        let mut indexed: Vec<(usize, usize, AnyElement)> = windows
            .into_iter()
            .enumerate()
            .map(|(original, w)| {
                let idx = z_order.rank.get(&w.id).copied().unwrap_or(usize::MAX);
                (idx, original, w)
            })
            .collect();

        indexed.sort_by_key(|(idx, original, _)| (*idx, *original));
        let windows_sorted: Vec<AnyElement> = indexed.into_iter().map(|(_, _, w)| w).collect();

        let mut props = ContainerProps::default();
        props.layout.position = PositionStyle::Absolute;
        props.layout.inset = InsetStyle {
            left: Some(Px(0.0)).into(),
            right: Some(Px(0.0)).into(),
            top: Some(Px(0.0)).into(),
            bottom: Some(Px(0.0)).into(),
        };
        props.layout.overflow = Overflow::Visible;
        props.layout.size.width = Length::Fill;
        props.layout.size.height = Length::Fill;

        let mut layer = cx.container(props, move |_cx| windows_sorted);
        layer.id = layer_id;
        layer
    })
}
