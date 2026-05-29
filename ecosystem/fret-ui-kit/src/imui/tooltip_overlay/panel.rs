use std::sync::Arc;

use fret_core::{Px, Rect, SemanticsRole, Size};
use fret_ui::element::{AnyElement, SemanticsDecoration};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use layout::{tooltip_panel_column_props, tooltip_panel_props};

use crate::imui::ImUiFacade;
use crate::overlay;
use crate::primitives::popper::{self, PopperContentPlacement};

mod layout;

pub(super) struct TooltipPanelBuildOptions {
    pub(super) trigger_id: GlobalElementId,
    pub(super) trigger_rect: Option<Rect>,
    pub(super) panel_size: Size,
    pub(super) placement: PopperContentPlacement,
    pub(super) window_margin: Px,
    pub(super) panel_id_model: fret_runtime::Model<Option<GlobalElementId>>,
    pub(super) panel_test_id: Option<Arc<str>>,
}

pub(super) fn tooltip_overlay_children<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    root_name: &str,
    options: TooltipPanelBuildOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> Vec<AnyElement> {
    let mut build = Some(f);
    cx.with_root_name(root_name, |cx| {
        let Some(anchor) =
            overlay::anchor_bounds_for_element(cx, options.trigger_id).or(options.trigger_rect)
        else {
            return Vec::new();
        };

        let outer = overlay::outer_bounds_with_window_margin_for_environment(
            cx,
            fret_ui::Invalidation::Layout,
            options.window_margin,
        );
        let layout = popper::popper_content_layout_sized(
            outer,
            anchor,
            options.panel_size,
            options.placement,
        );

        vec![cx.named("fret-ui-kit.imui.tooltip.panel", |cx| {
            let current_panel_id = cx.root_id();
            let _ = cx
                .app
                .models_mut()
                .update(&options.panel_id_model, |value| {
                    *value = Some(current_panel_id)
                });

            let panel_props = tooltip_panel_props(cx, layout.rect.origin);
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
            if let Some(test_id) = options.panel_test_id.as_ref() {
                semantics = semantics.test_id(test_id.clone());
            }
            panel = panel.attach_semantics(semantics);
            panel
        })]
    })
}
