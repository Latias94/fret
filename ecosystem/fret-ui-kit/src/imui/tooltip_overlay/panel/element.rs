use std::sync::Arc;

use fret_core::{Point, SemanticsRole};
use fret_runtime::Model;
use fret_ui::element::{AnyElement, SemanticsDecoration};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::layout::{tooltip_panel_column_props, tooltip_panel_props};
use crate::imui::ImUiFacade;

pub(super) struct TooltipPanelElementInput {
    pub(super) origin: Point,
    pub(super) panel_id_model: Model<Option<GlobalElementId>>,
    pub(super) panel_test_id: Option<Arc<str>>,
}

pub(super) fn tooltip_panel_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    input: TooltipPanelElementInput,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> AnyElement {
    let mut build = Some(f);
    cx.named("fret-ui-kit.imui.tooltip.panel", |cx| {
        let current_panel_id = cx.root_id();
        let _ = cx.app.models_mut().update(&input.panel_id_model, |value| {
            *value = Some(current_panel_id)
        });

        let panel_props = tooltip_panel_props(cx, input.origin);
        let mut panel = cx.container(panel_props, move |cx| {
            vec![cx.column(tooltip_panel_column_props(), move |cx| {
                let mut out = Vec::new();
                let mut ui = ImUiFacade {
                    cx,
                    out: &mut out,
                    build_focus: None,
                };
                if let Some(build) = build.take() {
                    build(&mut ui);
                }
                out
            })]
        });

        let mut semantics = SemanticsDecoration::default().role(SemanticsRole::Tooltip);
        if let Some(test_id) = input.panel_test_id.as_ref() {
            semantics = semantics.test_id(test_id.clone());
        }
        panel = panel.attach_semantics(semantics);
        panel
    })
}
