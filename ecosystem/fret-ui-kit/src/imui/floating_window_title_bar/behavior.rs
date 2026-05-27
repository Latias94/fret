use std::sync::Arc;

use fret_core::KeyCode;
use fret_runtime::Model;
use fret_ui::action::UiActionHostExt as _;
use fret_ui::element::{AnyElement, PressableProps};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

pub(super) fn title_bar_double_click_toggle_handler(
    can_collapse: bool,
) -> Option<super::super::OnFloatingAreaLeftDoubleClick> {
    can_collapse.then(|| {
        let handler: super::super::OnFloatingAreaLeftDoubleClick = Arc::new(
            |host: &mut dyn fret_ui::action::UiPointerActionHost,
             acx: fret_ui::action::ActionCx| {
                host.record_transient_event(acx, super::super::KEY_FLOAT_WINDOW_TOGGLE_COLLAPSED);
                host.notify(acx);
            },
        );
        handler
    })
}

pub(super) fn install_title_bar_key_behavior<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    region_id: GlobalElementId,
    can_close: bool,
    open_model: Option<Model<bool>>,
) {
    cx.key_clear_on_key_down_for(region_id);
    if can_close && let Some(open) = open_model {
        cx.key_on_key_down_for(
            region_id,
            Arc::new(move |host, acx, down| {
                if down.key != KeyCode::Escape || down.repeat {
                    return false;
                }
                let _ = host.update_model(&open, |v: &mut bool| {
                    *v = false;
                });
                host.notify(acx);
                true
            }),
        );
    }
}

pub(super) fn title_bar_close_button<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    props: PressableProps,
    open: Model<bool>,
) -> AnyElement {
    cx.pressable(props, move |cx, _state| {
        cx.pressable_on_activate(Arc::new(move |host, acx, _reason| {
            let _ = host.update_model(&open, |v: &mut bool| {
                *v = false;
            });
            host.notify(acx);
        }));
        vec![super::floating_window_close_glyph_text(cx)]
    })
}
