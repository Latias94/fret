use std::sync::Arc;

use fret_core::Corners;
use fret_ui::element::{AnyElement, ContainerProps, LayoutStyle, Length, Overflow, SizeStyle};
use fret_ui::{ElementContext, Theme, UiHost};

use super::super::spec::{DisclosureKind, DisclosureSpec};
use super::resolve_disclosure_palette;

mod children;
mod metrics;

use children::{HeaderChildrenRequest, header_children};
use metrics::{header_border_edges, header_indicator, header_row_padding};

pub(in crate::imui::disclosure_controls) fn header_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    spec: &DisclosureSpec,
    label: Arc<str>,
    open_now: bool,
    state: fret_ui::element::PressableState,
) -> AnyElement {
    let theme = Theme::global(&*cx.app);
    let palette = resolve_disclosure_palette(theme, spec, state);
    let border = theme.color_token("border");
    let indicator = header_indicator(spec, open_now);
    let row_padding = header_row_padding(spec);
    let border_edges = header_border_edges(spec.kind);

    let mut row_props = ContainerProps::default();
    row_props.layout = LayoutStyle {
        size: SizeStyle {
            width: Length::Fill,
            height: Length::Auto,
            ..Default::default()
        },
        overflow: Overflow::Visible,
        ..Default::default()
    };
    row_props.padding = row_padding.into();
    row_props.background = palette.background;
    row_props.border = border_edges;
    row_props.border_color = (spec.kind == DisclosureKind::CollapsingHeader).then_some(border);
    row_props.corner_radii = Corners::all(super::super::super::control_chrome::CONTROL_RADIUS);

    cx.container(row_props, move |cx| {
        header_children(
            cx,
            HeaderChildrenRequest {
                label,
                indicator,
                foreground: palette.foreground,
            },
        )
    })
}
