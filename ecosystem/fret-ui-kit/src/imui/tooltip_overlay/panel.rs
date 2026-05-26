use std::sync::Arc;

use fret_core::{Edges, Px, Rect, SemanticsRole, Size};
use fret_ui::element::{
    AnyElement, ColumnProps, ContainerProps, InsetStyle, LayoutStyle, Length, Overflow,
    PositionStyle, SemanticsDecoration, SpacingLength,
};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::imui::ImUiFacade;
use crate::overlay;
use crate::primitives::popper::{self, PopperContentPlacement};

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

fn tooltip_panel_props<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    origin: fret_core::Point,
) -> ContainerProps {
    let theme = fret_ui::Theme::global(&*cx.app);
    ContainerProps {
        layout: LayoutStyle {
            position: PositionStyle::Absolute,
            inset: InsetStyle {
                left: Some(origin.x).into(),
                top: Some(origin.y).into(),
                ..Default::default()
            },
            size: fret_ui::element::SizeStyle {
                width: Length::Auto,
                height: Length::Auto,
                ..Default::default()
            },
            overflow: Overflow::Visible,
            ..Default::default()
        },
        padding: Edges::all(Px(4.0)).into(),
        background: Some(theme.color_token("popover")),
        border: Edges::all(Px(1.0)),
        border_color: Some(theme.color_token("border")),
        corner_radii: fret_core::Corners::all(super::super::control_chrome::PANEL_RADIUS),
        ..Default::default()
    }
}

fn tooltip_panel_column_props() -> ColumnProps {
    let mut column = ColumnProps::default();
    column.layout.size.width = Length::Auto;
    column.layout.size.height = Length::Auto;
    column.gap = SpacingLength::Px(Px(4.0));
    column
}
