use fret_core::Point;
use fret_ui::element::{
    AnyElement, ContainerProps, HitTestGateProps, InsetStyle, InteractivityGateProps, LayoutStyle,
    Overflow, PositionStyle,
};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::imui::FloatingAreaOptions;

pub(super) fn floating_area_shell<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    area_id: GlobalElementId,
    final_position: Point,
    options: &FloatingAreaOptions,
    out: Vec<AnyElement>,
) -> AnyElement {
    let mut props = ContainerProps::default();
    props.layout = LayoutStyle {
        position: PositionStyle::Absolute,
        inset: InsetStyle {
            left: Some(final_position.x).into(),
            top: Some(final_position.y).into(),
            ..Default::default()
        },
        overflow: Overflow::Visible,
        ..Default::default()
    };

    if options.no_inputs {
        let layout = props.layout;
        let mut gate = cx.interactivity_gate_props(
            InteractivityGateProps {
                layout,
                present: true,
                interactive: false,
            },
            |_cx| out,
        );
        gate.id = area_id;
        return gate;
    }

    if options.hit_test_passthrough {
        let layout = props.layout;
        let mut gate = cx.hit_test_gate_props(
            HitTestGateProps {
                layout,
                hit_test: false,
            },
            |_cx| out,
        );
        gate.id = area_id;
        return gate;
    }

    let mut area = cx.container(props, move |_cx| out);
    area.id = area_id;
    area
}
