use std::collections::HashMap;
use std::sync::Arc;

use fret_core::window::WindowMetricsService;
use fret_core::{Point, Px};
use fret_ui::element::{
    AnyElement, ContainerProps, InsetStyle, LayoutStyle, Length, Overflow, PointerRegionProps,
    PositionStyle,
};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::{FloatingAreaContext, FloatingAreaOptions, FloatingAreaResponse, ImUiFacade};

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct FloatingWindowChromeResponse {
    pub(super) size: Option<fret_core::Size>,
    pub(super) resizing: bool,
    pub(super) collapsed: bool,
}

const FLOAT_WINDOW_DRAG_KIND_MASK: u64 = 0x4000_0000_0000_0000;
const FLOAT_WINDOW_RESIZE_KIND_BASE: u64 =
    super::fnv1a64(b"fret-ui-kit.imui.float_window.resize.v1");

pub(super) const KEY_FLOAT_WINDOW_ACTIVATE: u64 =
    super::fnv1a64(b"fret-ui-kit.imui.float_window.activate.v1");
pub(super) const KEY_FLOAT_WINDOW_TOGGLE_COLLAPSED: u64 =
    super::fnv1a64(b"fret-ui-kit.imui.float_window.toggle_collapsed.v1");

pub(super) type OnFloatingAreaLeftDoubleClick =
    Arc<dyn Fn(&mut dyn fret_ui::action::UiPointerActionHost, fret_ui::action::ActionCx) + 'static>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FloatWindowResizeHandle {
    Left,
    Right,
    Top,
    Bottom,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Debug)]
pub(super) struct FloatingAreaState {
    pub(super) position: Point,
    pub(super) last_drag_position: Option<Point>,
    pub(super) test_id: Arc<str>,
}

#[derive(Debug)]
pub(super) struct FloatWindowState {
    pub(super) size: fret_core::Size,
    pub(super) last_resize_position: Option<Point>,
    pub(super) title_bar_test_id: Arc<str>,
    pub(super) close_button_test_id: Arc<str>,
    pub(super) resize_left_test_id: Arc<str>,
    pub(super) resize_right_test_id: Arc<str>,
    pub(super) resize_top_test_id: Arc<str>,
    pub(super) resize_bottom_test_id: Arc<str>,
    pub(super) resize_top_left_test_id: Arc<str>,
    pub(super) resize_top_right_test_id: Arc<str>,
    pub(super) resize_bottom_left_test_id: Arc<str>,
    pub(super) resize_corner_test_id: Arc<str>,
}

#[derive(Debug, Clone, Copy)]
struct FloatWindowLayerMarker {
    layer: GlobalElementId,
}

#[derive(Debug, Default)]
struct FloatWindowLayerZOrder {
    order: Vec<GlobalElementId>,
    dirty: bool,
    snapshot: FloatWindowLayerZOrderSnapshot,
}

impl FloatWindowLayerZOrder {
    fn ensure_present(&mut self, window: GlobalElementId) {
        if self.order.contains(&window) {
            return;
        }
        self.order.push(window);
        self.dirty = true;
    }

    fn bring_to_front(&mut self, window: GlobalElementId) {
        self.ensure_present(window);
        let Some(idx) = self.order.iter().position(|w| *w == window) else {
            return;
        };
        if idx + 1 == self.order.len() {
            return;
        }
        self.order.remove(idx);
        self.order.push(window);
        self.dirty = true;
    }

    fn prune_missing(&mut self, windows: &[AnyElement]) {
        let before = self.order.len();
        self.order.retain(|id| windows.iter().any(|w| w.id == *id));
        if self.order.len() != before {
            self.dirty = true;
        }
    }

    fn snapshot(&mut self) -> FloatWindowLayerZOrderSnapshot {
        if !self.dirty {
            return self.snapshot.clone();
        }

        let order: Arc<[GlobalElementId]> = self.order.clone().into();
        let mut rank = HashMap::with_capacity(order.len());
        for (ix, id) in order.iter().enumerate() {
            rank.insert(*id, ix);
        }

        self.snapshot = FloatWindowLayerZOrderSnapshot {
            order,
            rank: Arc::new(rank),
        };
        self.dirty = false;
        self.snapshot.clone()
    }
}

#[derive(Debug, Clone, Default)]
struct FloatWindowLayerZOrderSnapshot {
    #[allow(dead_code)]
    order: Arc<[GlobalElementId]>,
    rank: Arc<HashMap<GlobalElementId, usize>>,
}

pub(super) fn float_window_drag_kind_for_element(
    element: GlobalElementId,
) -> fret_runtime::DragKindId {
    fret_runtime::DragKindId(FLOAT_WINDOW_DRAG_KIND_MASK | element.0)
}

pub(super) fn float_window_resize_kind_for_element(
    element: GlobalElementId,
    handle: FloatWindowResizeHandle,
) -> fret_runtime::DragKindId {
    let handle_tag = match handle {
        FloatWindowResizeHandle::Left => 1,
        FloatWindowResizeHandle::Right => 2,
        FloatWindowResizeHandle::Top => 3,
        FloatWindowResizeHandle::Bottom => 4,
        FloatWindowResizeHandle::TopLeft => 5,
        FloatWindowResizeHandle::TopRight => 6,
        FloatWindowResizeHandle::BottomLeft => 7,
        FloatWindowResizeHandle::BottomRight => 8,
    };
    fret_runtime::DragKindId(
        FLOAT_WINDOW_RESIZE_KIND_BASE ^ element.0.wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ handle_tag,
    )
}

pub(super) fn float_layer_bring_to_front_if_activated<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    window_id: GlobalElementId,
) {
    if !cx.take_transient_for(window_id, KEY_FLOAT_WINDOW_ACTIVATE) {
        return;
    }
    let Some(marker) = cx.inherited_state::<FloatWindowLayerMarker>() else {
        return;
    };
    cx.state_for(marker.layer, FloatWindowLayerZOrder::default, |st| {
        st.bring_to_front(window_id);
    });
}

pub(super) fn floating_layer_element<H: UiHost>(
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

pub(super) fn floating_area_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: &str,
    initial_position: Point,
    options: FloatingAreaOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>, FloatingAreaContext),
) -> (AnyElement, FloatingAreaResponse) {
    cx.named(id, |cx| {
        let area_id = cx.root_id();
        if let Some(marker) = cx.inherited_state::<FloatWindowLayerMarker>() {
            cx.state_for(marker.layer, FloatWindowLayerZOrder::default, |st| {
                st.ensure_present(area_id);
            });
        }

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
                        st.position =
                            super::point_add(st.position, super::point_sub(current, prev));
                        st.position = super::snap_point_to_device_pixels(scale_factor, st.position);
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

pub(super) fn floating_area_drag_surface_element<H: UiHost, Setup, Build>(
    cx: &mut ElementContext<'_, H>,
    area: FloatingAreaContext,
    props: PointerRegionProps,
    on_left_double_click: Option<OnFloatingAreaLeftDoubleClick>,
    enable_drag: bool,
    enable_activation: bool,
    setup: Setup,
    build: Build,
) -> AnyElement
where
    Setup: FnOnce(&mut ElementContext<'_, H>, GlobalElementId),
    Build: for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
{
    let mut build = Some(build);
    let mut setup = Some(setup);
    let on_left_double_click_for_down = on_left_double_click.clone();
    cx.pointer_region(props, move |cx| {
        let region_id = cx.root_id();
        float_layer_bring_to_front_if_activated(cx, area.id);

        cx.key_clear_on_key_down_for(region_id);
        if let Some(setup) = setup.take() {
            setup(cx, region_id);
        }
        cx.key_add_on_key_down_for(region_id, Arc::new(|_host, _acx, _down| false));

        let drag_kind = area.drag_kind;
        cx.pointer_region_on_pointer_down(Arc::new(move |host, acx, down| {
            if !super::prepare_pointer_region_drag_on_left_down(
                host,
                acx,
                down,
                enable_drag.then_some(drag_kind),
                None,
            ) {
                return false;
            }
            if down.click_count == 2
                && let Some(on_left_double_click) = on_left_double_click_for_down.as_ref()
            {
                on_left_double_click(
                    host,
                    fret_ui::action::ActionCx {
                        window: acx.window,
                        target: area.id,
                    },
                );
            }
            if enable_activation {
                host.record_transient_event(
                    fret_ui::action::ActionCx {
                        window: acx.window,
                        target: area.id,
                    },
                    KEY_FLOAT_WINDOW_ACTIVATE,
                );
            }
            host.notify(acx);
            false
        }));

        let drag_threshold = super::drag_threshold_for(cx);
        cx.pointer_region_on_pointer_move(Arc::new(move |host, acx, mv| {
            if !enable_drag {
                return false;
            }
            super::handle_pointer_region_drag_move_with_threshold(
                host,
                acx,
                mv,
                drag_kind,
                drag_threshold,
            )
        }));

        cx.pointer_region_on_pointer_up(Arc::new(move |host, acx, up| {
            if !enable_drag {
                return false;
            }
            super::finish_pointer_region_drag(host, acx, up.pointer_id, drag_kind)
        }));

        let mut out = Vec::new();
        if let Some(build) = build.take() {
            let mut ui = ImUiFacade {
                cx,
                out: &mut out,
                build_focus: None,
            };
            build(&mut ui);
        }
        out
    })
}
