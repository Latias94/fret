use std::sync::Arc;
use std::{cell::RefCell, rc::Rc};

use fret_core::{
    Corners, Edges, KeyCode, Point, Px, Rect, SemanticsRole, Size, TextOverflow, TextWrap,
};
use fret_ui::element::{
    AnyElement, ColumnProps, ContainerProps, CrossAlign, InsetEdge, Length, PointerRegionProps,
    PositionStyle, PressableProps, SemanticsProps, SpacingEdges, SpacingLength, TextProps,
};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::core::{CanvasPoint, Graph};
use crate::ops::{GraphOp, GraphTransaction};
use crate::runtime::events::ConnectDragKind;
use crate::ui::{InsertNodeCandidate, NodeGraphPresenter, NodeGraphStyle, NodeGraphSurfaceBinding};

use super::NodeGraphDeclarativeInsertNodePickerRequest;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeGraphDeclarativeInsertNodePickerOpenOutcome {
    Opened { candidate_count: usize },
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeGraphDeclarativeInsertNodePickerPlanError {
    NoActiveSession,
    CandidateOutOfRange { index: usize, len: usize },
    CandidateDisabled { index: usize },
    InvalidCanvasPosition,
    Rejected(Arc<str>),
    EmptyTransaction,
}

#[derive(Debug, Clone)]
pub struct NodeGraphDeclarativeInsertNodePickerSession {
    pub request: NodeGraphDeclarativeInsertNodePickerRequest,
    pub candidates: Vec<InsertNodeCandidate>,
}

#[derive(Debug, Default, Clone)]
pub struct NodeGraphDeclarativeInsertNodePickerState {
    active: Option<NodeGraphDeclarativeInsertNodePickerSession>,
}

pub trait NodeGraphDeclarativeInsertNodePickerCandidateProvider {
    fn list_insert_node_candidates(
        &mut self,
        graph: &Graph,
        request: &NodeGraphDeclarativeInsertNodePickerRequest,
    ) -> Vec<InsertNodeCandidate>;

    fn plan_insert_node_candidate(
        &mut self,
        graph: &Graph,
        request: &NodeGraphDeclarativeInsertNodePickerRequest,
        candidate: &InsertNodeCandidate,
    ) -> Result<Vec<GraphOp>, Arc<str>>;
}

pub type NodeGraphDeclarativeInsertNodePickerStateRef =
    Rc<RefCell<NodeGraphDeclarativeInsertNodePickerState>>;

pub type NodeGraphDeclarativeInsertNodePickerCandidateProviderRef =
    Rc<RefCell<Box<dyn NodeGraphDeclarativeInsertNodePickerCandidateProvider>>>;

#[derive(Clone)]
pub struct NodeGraphDeclarativeInsertNodePickerOverlayBinding {
    pub state: NodeGraphDeclarativeInsertNodePickerStateRef,
    pub provider: NodeGraphDeclarativeInsertNodePickerCandidateProviderRef,
}

impl NodeGraphDeclarativeInsertNodePickerOverlayBinding {
    pub fn new(
        state: NodeGraphDeclarativeInsertNodePickerStateRef,
        provider: NodeGraphDeclarativeInsertNodePickerCandidateProviderRef,
    ) -> Self {
        Self { state, provider }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct NodeGraphDeclarativeInsertNodePickerOverlayState {
    active_index: usize,
}

pub(super) struct NodeGraphDeclarativeInsertNodePickerOverlayParams {
    pub(super) binding: NodeGraphSurfaceBinding,
    pub(super) picker: NodeGraphDeclarativeInsertNodePickerOverlayBinding,
    pub(super) bounds: Rect,
    pub(super) style_tokens: NodeGraphStyle,
    pub(super) focus_target: GlobalElementId,
}

impl<T> NodeGraphDeclarativeInsertNodePickerCandidateProvider for T
where
    T: NodeGraphPresenter + ?Sized,
{
    fn list_insert_node_candidates(
        &mut self,
        graph: &Graph,
        request: &NodeGraphDeclarativeInsertNodePickerRequest,
    ) -> Vec<InsertNodeCandidate> {
        match &request.connect_end.kind {
            ConnectDragKind::New { from, .. } => {
                self.list_insertable_nodes_for_connection(graph, *from)
            }
            ConnectDragKind::Reconnect { fixed, .. } => {
                self.list_insertable_nodes_for_connection(graph, *fixed)
            }
            ConnectDragKind::ReconnectMany { .. } => self.list_insertable_nodes(graph),
        }
    }

    fn plan_insert_node_candidate(
        &mut self,
        graph: &Graph,
        request: &NodeGraphDeclarativeInsertNodePickerRequest,
        candidate: &InsertNodeCandidate,
    ) -> Result<Vec<GraphOp>, Arc<str>> {
        let at = request_canvas_point(request)
            .map_err(|_| Arc::<str>::from("insert-node picker request has invalid canvas point"))?;
        self.plan_create_node(graph, candidate, at)
    }
}

impl NodeGraphDeclarativeInsertNodePickerState {
    pub fn active(&self) -> Option<&NodeGraphDeclarativeInsertNodePickerSession> {
        self.active.as_ref()
    }

    pub fn is_open(&self) -> bool {
        self.active.is_some()
    }

    pub fn open_with_candidates(
        &mut self,
        request: NodeGraphDeclarativeInsertNodePickerRequest,
        candidates: Vec<InsertNodeCandidate>,
    ) -> NodeGraphDeclarativeInsertNodePickerOpenOutcome {
        if candidates.is_empty() {
            self.active = None;
            return NodeGraphDeclarativeInsertNodePickerOpenOutcome::Empty;
        }

        let candidate_count = candidates.len();
        self.active = Some(NodeGraphDeclarativeInsertNodePickerSession {
            request,
            candidates,
        });
        NodeGraphDeclarativeInsertNodePickerOpenOutcome::Opened { candidate_count }
    }

    pub fn open_from_provider(
        &mut self,
        graph: &Graph,
        request: NodeGraphDeclarativeInsertNodePickerRequest,
        provider: &mut dyn NodeGraphDeclarativeInsertNodePickerCandidateProvider,
    ) -> NodeGraphDeclarativeInsertNodePickerOpenOutcome {
        let candidates = provider.list_insert_node_candidates(graph, &request);
        self.open_with_candidates(request, candidates)
    }

    pub fn cancel(&mut self) -> bool {
        self.active.take().is_some()
    }

    pub fn close_after_commit(&mut self) -> bool {
        self.cancel()
    }

    pub fn plan_candidate_with_provider(
        &self,
        graph: &Graph,
        provider: &mut dyn NodeGraphDeclarativeInsertNodePickerCandidateProvider,
        index: usize,
    ) -> Result<GraphTransaction, NodeGraphDeclarativeInsertNodePickerPlanError> {
        let Some(session) = self.active.as_ref() else {
            return Err(NodeGraphDeclarativeInsertNodePickerPlanError::NoActiveSession);
        };
        let Some(candidate) = session.candidates.get(index) else {
            return Err(
                NodeGraphDeclarativeInsertNodePickerPlanError::CandidateOutOfRange {
                    index,
                    len: session.candidates.len(),
                },
            );
        };
        if !candidate.enabled {
            return Err(NodeGraphDeclarativeInsertNodePickerPlanError::CandidateDisabled { index });
        }

        let ops = provider
            .plan_insert_node_candidate(graph, &session.request, candidate)
            .map_err(NodeGraphDeclarativeInsertNodePickerPlanError::Rejected)?;
        if ops.is_empty() {
            return Err(NodeGraphDeclarativeInsertNodePickerPlanError::EmptyTransaction);
        }
        Ok(GraphTransaction { label: None, ops }.with_label("Insert Node"))
    }
}

pub(super) fn push_insert_node_picker_overlay<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    out: &mut Vec<AnyElement>,
    params: NodeGraphDeclarativeInsertNodePickerOverlayParams,
) {
    let Some(session) = params.picker.state.borrow().active().cloned() else {
        return;
    };
    if session.candidates.is_empty() {
        return;
    }

    let overlay_state = cx.local_model_keyed(
        "insert_node_picker.overlay_state",
        NodeGraphDeclarativeInsertNodePickerOverlayState::default,
    );
    let active_index = cx
        .app
        .models()
        .read(&overlay_state, |state| {
            state
                .active_index
                .min(session.candidates.len().saturating_sub(1))
        })
        .ok()
        .unwrap_or(0);

    let style = params.style_tokens;
    let panel_size = insert_node_picker_panel_size(&style, session.candidates.len());
    let panel_rect =
        insert_node_picker_panel_rect(params.bounds, session.request.screen_position, panel_size);
    let picker_state = params.picker.state.clone();
    let picker_provider = params.picker.provider.clone();
    let binding = params.binding.clone();
    let focus_target = params.focus_target;

    out.push(cx.semantics_with_id(
        insert_node_picker_semantics(active_index, &session.candidates),
        move |cx, picker_root| {
            let key_binding = binding.clone();
            let key_picker_state = picker_state.clone();
            let key_picker_provider = picker_provider.clone();
            let key_overlay_state = overlay_state.clone();
            cx.key_on_key_down_for(
                picker_root,
                Arc::new(move |host, action_cx, down| match down.key {
                    KeyCode::Escape => {
                        if key_picker_state.borrow_mut().cancel() {
                            host.request_focus(focus_target);
                            host.request_redraw(action_cx.window);
                            host.notify(action_cx);
                        }
                        true
                    }
                    KeyCode::ArrowDown => {
                        update_insert_node_picker_active_index(
                            host.models_mut(),
                            &key_overlay_state,
                            1,
                        );
                        host.notify(action_cx);
                        true
                    }
                    KeyCode::ArrowUp => {
                        update_insert_node_picker_active_index(
                            host.models_mut(),
                            &key_overlay_state,
                            -1,
                        );
                        host.notify(action_cx);
                        true
                    }
                    KeyCode::Enter | KeyCode::NumpadEnter | KeyCode::Space => {
                        let index = host
                            .models_mut()
                            .read(&key_overlay_state, |state| state.active_index)
                            .ok()
                            .unwrap_or(0);
                        let committed = commit_insert_node_picker_candidate(
                            host,
                            action_cx,
                            &key_binding,
                            &key_picker_state,
                            &key_picker_provider,
                            index,
                        );
                        if committed {
                            host.request_focus(focus_target);
                        }
                        committed
                    }
                    _ => false,
                }),
            );

            vec![cx.pointer_region(PointerRegionProps::default(), move |cx| {
                cx.pointer_region_on_pointer_down(Arc::new(move |host, _action_cx, _down| {
                    host.request_focus(picker_root);
                    true
                }));

                vec![cx.container(
                    insert_node_picker_container(panel_rect, params.bounds, &style),
                    move |cx| {
                        vec![cx.column(insert_node_picker_column(), move |cx| {
                            session
                                .candidates
                                .iter()
                                .cloned()
                                .enumerate()
                                .map(|(index, candidate)| {
                                    let row_binding = binding.clone();
                                    let row_picker_state = picker_state.clone();
                                    let row_picker_provider = picker_provider.clone();
                                    let row_overlay_state = overlay_state.clone();
                                    insert_node_picker_candidate_row(
                                        cx,
                                        &style,
                                        picker_root,
                                        focus_target,
                                        row_binding,
                                        row_picker_state,
                                        row_picker_provider,
                                        row_overlay_state,
                                        index,
                                        active_index,
                                        candidate,
                                    )
                                })
                                .collect::<Vec<_>>()
                        })]
                    },
                )]
            })]
        },
    ));
}

fn insert_node_picker_semantics(
    active_index: usize,
    candidates: &[InsertNodeCandidate],
) -> SemanticsProps {
    SemanticsProps {
        role: SemanticsRole::ListBox,
        label: Some(Arc::from("Insert node")),
        test_id: Some(Arc::from("node_graph.insert_node_picker")),
        value: candidates
            .get(active_index)
            .map(|candidate| candidate.label.clone()),
        focusable: true,
        ..Default::default()
    }
}

fn insert_node_picker_candidate_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    style: &NodeGraphStyle,
    picker_root: GlobalElementId,
    focus_target: GlobalElementId,
    binding: NodeGraphSurfaceBinding,
    picker_state: NodeGraphDeclarativeInsertNodePickerStateRef,
    picker_provider: NodeGraphDeclarativeInsertNodePickerCandidateProviderRef,
    overlay_state: fret_runtime::Model<NodeGraphDeclarativeInsertNodePickerOverlayState>,
    index: usize,
    active_index: usize,
    candidate: InsertNodeCandidate,
) -> AnyElement {
    let mut props = PressableProps::default();
    props.enabled = candidate.enabled;
    props.layout.size.width = Length::Fill;
    props.layout.size.height = Length::Px(Px(style.paint.context_menu_item_height.max(18.0)));
    props.a11y.role = Some(SemanticsRole::ListBoxOption);
    props.a11y.label = Some(candidate.label.clone());
    props.a11y.test_id = Some(Arc::from(format!(
        "node_graph.insert_node_picker.candidate.{index}"
    )));

    cx.pressable(props, move |cx, _state| {
        let pointer_overlay_state = overlay_state.clone();
        cx.pressable_on_pointer_down(Arc::new(move |host, action_cx, _down| {
            let _ = host
                .models_mut()
                .update(&pointer_overlay_state, |state| state.active_index = index);
            host.request_focus(picker_root);
            host.notify(action_cx);
            fret_ui::action::PressablePointerDownResult::Continue
        }));

        cx.pressable_on_activate(Arc::new(move |host, action_cx, _reason| {
            let _ = host
                .models_mut()
                .update(&overlay_state, |state| state.active_index = index);
            let _ = commit_insert_node_picker_candidate(
                host,
                action_cx,
                &binding,
                &picker_state,
                &picker_provider,
                index,
            );
        }));
        cx.pressable_on_activate_focus(Arc::new(move |host, _action_cx, _reason| {
            host.request_focus(focus_target);
        }));

        let mut label = TextProps::new(candidate.label.clone());
        label.style = Some(style.geometry.context_menu_text_style.clone());
        label.color = Some(if candidate.enabled {
            style.paint.context_menu_text
        } else {
            style.paint.context_menu_text_disabled
        });
        label.wrap = TextWrap::None;
        label.overflow = TextOverflow::Clip;

        vec![cx.container(
            candidate_row_container(style, index == active_index),
            move |cx| vec![cx.text_props(label)],
        )]
    })
}

fn commit_insert_node_picker_candidate(
    host: &mut dyn fret_ui::action::UiActionHost,
    action_cx: fret_ui::action::ActionCx,
    binding: &NodeGraphSurfaceBinding,
    picker_state: &NodeGraphDeclarativeInsertNodePickerStateRef,
    picker_provider: &NodeGraphDeclarativeInsertNodePickerCandidateProviderRef,
    index: usize,
) -> bool {
    let graph = host
        .models_mut()
        .read(&binding.store_model(), |store| store.graph().clone())
        .ok();
    let Some(graph) = graph else {
        return false;
    };
    let tx = {
        let state = picker_state.borrow();
        let mut provider = picker_provider.borrow_mut();
        state.plan_candidate_with_provider(&graph, &mut **provider, index)
    };
    let Ok(tx) = tx else {
        return false;
    };
    if binding.dispatch_transaction_action_host(host, &tx).is_err() {
        return false;
    }
    picker_state.borrow_mut().close_after_commit();
    host.request_redraw(action_cx.window);
    host.notify(action_cx);
    true
}

fn update_insert_node_picker_active_index(
    models: &mut fret_runtime::ModelStore,
    overlay_state: &fret_runtime::Model<NodeGraphDeclarativeInsertNodePickerOverlayState>,
    delta: isize,
) {
    let _ = models.update(overlay_state, |state| {
        state.active_index = state.active_index.saturating_add_signed(delta);
    });
}

fn insert_node_picker_panel_size(style: &NodeGraphStyle, candidates: usize) -> Size {
    let padding = style.paint.context_menu_padding.max(0.0);
    let item_height = style.paint.context_menu_item_height.max(18.0);
    Size::new(
        Px(style.paint.context_menu_width.max(120.0)),
        Px((padding * 2.0) + item_height * candidates.max(1) as f32),
    )
}

fn insert_node_picker_panel_rect(bounds: Rect, desired: Point, size: Size) -> Rect {
    let max_x = (bounds.origin.x.0 + bounds.size.width.0 - size.width.0).max(bounds.origin.x.0);
    let max_y = (bounds.origin.y.0 + bounds.size.height.0 - size.height.0).max(bounds.origin.y.0);
    Rect::new(
        Point::new(
            Px(desired.x.0.clamp(bounds.origin.x.0, max_x)),
            Px(desired.y.0.clamp(bounds.origin.y.0, max_y)),
        ),
        size,
    )
}

fn insert_node_picker_container(
    rect: Rect,
    bounds: Rect,
    style: &NodeGraphStyle,
) -> ContainerProps {
    let mut props = ContainerProps::default();
    props.layout.position = PositionStyle::Absolute;
    props.layout.inset.left = InsetEdge::Px(Px(rect.origin.x.0 - bounds.origin.x.0));
    props.layout.inset.top = InsetEdge::Px(Px(rect.origin.y.0 - bounds.origin.y.0));
    props.layout.size.width = Length::Px(rect.size.width);
    props.layout.size.height = Length::Px(rect.size.height);
    props.padding = SpacingEdges::all(SpacingLength::Px(Px(style
        .paint
        .context_menu_padding
        .max(0.0))));
    props.background = Some(style.paint.context_menu_background);
    props.border = Edges::all(Px(1.0));
    props.border_color = Some(style.paint.context_menu_border);
    props.corner_radii = Corners::all(Px(style.paint.context_menu_corner_radius.max(0.0)));
    props.snap_to_device_pixels = true;
    props
}

fn insert_node_picker_column() -> ColumnProps {
    let mut props = ColumnProps::default();
    props.layout.size.width = Length::Fill;
    props.layout.size.height = Length::Fill;
    props.align = CrossAlign::Stretch;
    props
}

fn candidate_row_container(style: &NodeGraphStyle, active: bool) -> ContainerProps {
    let mut props = ContainerProps::default();
    props.layout.size.width = Length::Fill;
    props.layout.size.height = Length::Fill;
    props.background = active.then_some(style.paint.context_menu_hover_background);
    props.padding = SpacingEdges::all(SpacingLength::Px(Px(4.0)));
    props
}

fn request_canvas_point(
    request: &NodeGraphDeclarativeInsertNodePickerRequest,
) -> Result<CanvasPoint, NodeGraphDeclarativeInsertNodePickerPlanError> {
    canvas_point_from_core_point(request.canvas_position)
        .ok_or(NodeGraphDeclarativeInsertNodePickerPlanError::InvalidCanvasPosition)
}

fn canvas_point_from_core_point(point: Point) -> Option<CanvasPoint> {
    let x = point.x.0;
    let y = point.y.0;
    (x.is_finite() && y.is_finite()).then_some(CanvasPoint { x, y })
}
