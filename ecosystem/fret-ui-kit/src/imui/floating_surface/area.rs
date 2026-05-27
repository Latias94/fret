use std::sync::Arc;

use fret_core::Point;
use fret_core::window::WindowMetricsService;
use fret_ui::element::{
    AnyElement, ContainerProps, InsetStyle, LayoutStyle, Overflow, PositionStyle,
};
use fret_ui::{ElementContext, UiHost};

use super::super::{FloatingAreaContext, FloatingAreaOptions, FloatingAreaResponse, ImUiFacade};
use super::kinds::float_window_drag_kind_for_element;
use super::state::FloatingAreaState;

pub(in crate::imui) fn floating_area_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: &str,
    initial_position: Point,
    options: FloatingAreaOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>, FloatingAreaContext),
) -> (AnyElement, FloatingAreaResponse) {
    cx.named(id, |cx| {
        let area_id = cx.root_id();
        super::layer::register_floating_layer_child(cx, area_id);

        let drag_kind = float_window_drag_kind_for_element(area_id);
        let drag_snapshot = cx
            .app
            .find_drag_pointer_id(|d| {
                d.kind == drag_kind && d.source_window == cx.window && d.current_window == cx.window
            })
            .and_then(|pointer_id| cx.app.drag(pointer_id))
            .filter(|drag| drag.kind == drag_kind)
            .map(|drag| (drag.dragging, drag.position, drag.start_position));
        let dragging = drag_snapshot
            .map(|(dragging, _, _)| dragging)
            .unwrap_or(false);

        let scale_factor = cx
            .app
            .global::<WindowMetricsService>()
            .and_then(|svc| svc.scale_factor(cx.window))
            .unwrap_or(1.0);
        let (position, test_id) = cx.state_for(
            area_id,
            || FloatingAreaState {
                position: initial_position,
                last_drag_position: None,
                test_id: options
                    .test_id
                    .clone()
                    .unwrap_or_else(|| Arc::from(format!("{}{id}", options.test_id_prefix))),
            },
            |st| {
                if let Some(test_id) = options.test_id.clone() {
                    st.test_id = test_id;
                }

                if let Some((dragging, current, start)) = drag_snapshot {
                    if dragging {
                        let prev = st.last_drag_position.unwrap_or(start);
                        st.position = super::super::point_add(
                            st.position,
                            super::super::point_sub(current, prev),
                        );
                        st.position =
                            super::super::snap_point_to_device_pixels(scale_factor, st.position);
                        st.last_drag_position = Some(current);
                    } else {
                        st.last_drag_position = None;
                    }
                } else {
                    st.last_drag_position = None;
                }
                (st.position, st.test_id.clone())
            },
        );

        let ctx = FloatingAreaContext {
            id: area_id,
            position,
            drag_kind,
        };

        let mut out: Vec<AnyElement> = Vec::new();
        {
            let mut ui = ImUiFacade {
                cx,
                out: &mut out,
                build_focus: None,
            };
            f(&mut ui, ctx);
        }

        let (final_position, final_test_id) = cx.state_for(
            area_id,
            || FloatingAreaState {
                position,
                last_drag_position: None,
                test_id: test_id.clone(),
            },
            |st| (st.position, st.test_id.clone()),
        );

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

        let area = if options.no_inputs {
            let layout = props.layout;
            let mut gate = cx.interactivity_gate_props(
                fret_ui::element::InteractivityGateProps {
                    layout,
                    present: true,
                    interactive: false,
                },
                |_cx| out,
            );
            gate.id = area_id;
            gate
        } else if options.hit_test_passthrough {
            let layout = props.layout;
            let mut gate = cx.hit_test_gate_props(
                fret_ui::element::HitTestGateProps {
                    layout,
                    hit_test: false,
                },
                |_cx| out,
            );
            gate.id = area_id;
            gate
        } else {
            let mut area = cx.container(props, move |_cx| out);
            area.id = area_id;
            area
        };
        let area = area.test_id(final_test_id);

        let response = FloatingAreaResponse {
            id: area_id,
            rect: cx.last_bounds_for_element(area_id),
            position: final_position,
            dragging,
            drag_kind,
        };

        (area, response)
    })
}
