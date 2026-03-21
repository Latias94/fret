//! AI Elements-aligned `Confirmation` surfaces.

use std::sync::Arc;

use fret_runtime::ActionId;
use fret_ui::element::{AnyElement, InteractivityGateProps, LayoutStyle, SemanticsDecoration};
use fret_ui::Theme;
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::typography::{description_text_refinement, muted_foreground_color};
use fret_ui_kit::ui;
use fret_ui_kit::{Items, Justify, LayoutRefinement, Space};
use fret_ui_shadcn::facade::{Alert, Button, ButtonSize, ButtonVariant};

type ActionPayloadFactory = Arc<dyn Fn() -> Box<dyn std::any::Any + Send + Sync> + 'static>;

/// Tool UI part state aligned with AI Elements `ToolUIPart["state"]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToolUiPartState {
    InputStreaming,
    InputAvailable,
    ApprovalRequested,
    ApprovalResponded,
    OutputDenied,
    OutputAvailable,
}

impl ToolUiPartState {
    pub fn is_input_state(self) -> bool {
        matches!(self, Self::InputStreaming | Self::InputAvailable)
    }

    pub fn is_response_state(self) -> bool {
        matches!(
            self,
            Self::ApprovalResponded | Self::OutputDenied | Self::OutputAvailable
        )
    }
}

/// Approval record aligned with AI Elements `ToolUIPartApproval`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolUiPartApproval {
    pub id: Arc<str>,
    pub approved: Option<bool>,
    pub reason: Option<Arc<str>>,
}

impl ToolUiPartApproval {
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        Self {
            id: id.into(),
            approved: None,
            reason: None,
        }
    }

    pub fn approved(mut self, approved: bool) -> Self {
        self.approved = Some(approved);
        self
    }

    pub fn reason(mut self, reason: impl Into<Arc<str>>) -> Self {
        self.reason = Some(reason.into());
        self
    }
}

fn hidden<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    cx.interactivity_gate_props(
        InteractivityGateProps {
            layout: LayoutStyle::default(),
            present: false,
            interactive: false,
        },
        |_cx| Vec::new(),
    )
}

const CONFIRMATION_REQUEST_SLOT_KEY: &str = "__fret_ui_ai.confirmation.request";
const CONFIRMATION_ACCEPTED_SLOT_KEY: &str = "__fret_ui_ai.confirmation.accepted";
const CONFIRMATION_REJECTED_SLOT_KEY: &str = "__fret_ui_ai.confirmation.rejected";
const CONFIRMATION_ACTIONS_SLOT_KEY: &str = "__fret_ui_ai.confirmation.actions";

fn deferred_slot<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    slot_key: &'static str,
    visible_child: AnyElement,
) -> AnyElement {
    let mut slot = cx.interactivity_gate_props(
        InteractivityGateProps {
            layout: LayoutStyle::default(),
            present: false,
            interactive: false,
        },
        move |_cx| vec![visible_child],
    );
    slot.key_context = Some(Arc::<str>::from(slot_key));
    slot
}

fn resolve_confirmation_children(
    children: Vec<AnyElement>,
    context: &ConfirmationContext,
) -> Vec<AnyElement> {
    children
        .into_iter()
        .filter_map(|child| resolve_confirmation_slot(child, context))
        .collect()
}

fn resolve_confirmation_slot(
    mut element: AnyElement,
    context: &ConfirmationContext,
) -> Option<AnyElement> {
    match element.key_context.as_deref() {
        Some(CONFIRMATION_REQUEST_SLOT_KEY) => {
            if context.state != ToolUiPartState::ApprovalRequested {
                return None;
            }
            return element
                .children
                .into_iter()
                .next()
                .and_then(|child| resolve_confirmation_slot(child, context));
        }
        Some(CONFIRMATION_ACCEPTED_SLOT_KEY) => {
            if context.approval.approved != Some(true) || !context.state.is_response_state() {
                return None;
            }
            return element
                .children
                .into_iter()
                .next()
                .and_then(|child| resolve_confirmation_slot(child, context));
        }
        Some(CONFIRMATION_REJECTED_SLOT_KEY) => {
            if context.approval.approved != Some(false) || !context.state.is_response_state() {
                return None;
            }
            return element
                .children
                .into_iter()
                .next()
                .and_then(|child| resolve_confirmation_slot(child, context));
        }
        Some(CONFIRMATION_ACTIONS_SLOT_KEY) => {
            if context.state != ToolUiPartState::ApprovalRequested {
                return None;
            }
            return element
                .children
                .into_iter()
                .next()
                .and_then(|child| resolve_confirmation_slot(child, context));
        }
        _ => {}
    }

    element.children = resolve_confirmation_children(element.children, context);
    Some(element)
}

/// Nearest `Confirmation` context in scope.
#[derive(Debug, Clone)]
pub struct ConfirmationContext {
    pub approval: ToolUiPartApproval,
    pub state: ToolUiPartState,
}

#[derive(Debug, Default, Clone)]
struct ConfirmationLocalState {
    context: Option<ConfirmationContext>,
}

pub fn use_confirmation_context<H: UiHost>(
    cx: &ElementContext<'_, H>,
) -> Option<ConfirmationContext> {
    cx.inherited_state::<ConfirmationLocalState>()
        .and_then(|st| st.context.clone())
}

/// Confirmation root aligned with AI Elements `Confirmation`.
#[derive(Debug)]
pub struct Confirmation {
    approval: Option<ToolUiPartApproval>,
    state: ToolUiPartState,
    children: Vec<AnyElement>,
    layout: LayoutRefinement,
    test_id: Option<Arc<str>>,
}

impl Confirmation {
    pub fn new(state: ToolUiPartState) -> Self {
        Self {
            approval: None,
            state,
            children: Vec::new(),
            layout: LayoutRefinement::default().w_full().min_w_0(),
            test_id: None,
        }
    }

    pub fn approval(mut self, approval: ToolUiPartApproval) -> Self {
        self.approval = Some(approval);
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(approval) = self.approval.clone() else {
            return hidden(cx);
        };
        if self.state.is_input_state() {
            return hidden(cx);
        }

        let context = ConfirmationContext {
            approval,
            state: self.state,
        };

        let Confirmation {
            approval: _,
            state: _,
            children,
            layout,
            test_id,
        } = self;
        let children = resolve_confirmation_children(children, &context);
        Self::render(cx, layout, test_id, children)
    }

    pub fn into_element_with_children<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        children: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
    ) -> AnyElement {
        if self.approval.is_none() || self.state.is_input_state() {
            return hidden(cx);
        }

        let Some(context) = self
            .approval
            .as_ref()
            .cloned()
            .map(|approval| ConfirmationContext {
                approval,
                state: self.state,
            })
        else {
            return hidden(cx);
        };

        cx.root_state(ConfirmationLocalState::default, |st| {
            st.context = Some(context.clone())
        });
        let children = resolve_confirmation_children(children(cx), &context);
        let Confirmation {
            approval: _,
            state: _,
            children: _,
            layout,
            test_id,
        } = self;
        Self::render(cx, layout, test_id, children)
    }

    fn render<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        layout: LayoutRefinement,
        test_id: Option<Arc<str>>,
        children: Vec<AnyElement>,
    ) -> AnyElement {
        let body_children = children;

        // AI Elements overrides shadcn/ui Alert's default grid layout with `flex flex-col gap-2`.
        // Our Alert implementation has its own internal gap policy, so we model the AI Elements
        // outcome by wrapping the confirmation children in a single vstack with the desired gap.
        //
        // Note: this intentionally disables Alert's "icon as first child" heuristics for this
        // surface, matching the AI Elements Confirmation composition.
        let body = ui::v_stack(move |_cx| body_children)
            .gap(Space::N2)
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .into_element(cx);

        let alert = Alert::new([body]).refine_layout(layout).into_element(cx);

        let Some(test_id) = test_id else {
            return alert;
        };
        alert.attach_semantics(SemanticsDecoration::default().test_id(test_id))
    }
}

/// Inline title aligned with AI Elements `ConfirmationTitle`.
#[derive(Debug)]
pub struct ConfirmationTitle {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
}

impl ConfirmationTitle {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            test_id: None,
        }
    }

    pub fn test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).snapshot();
        let description_style = description_text_refinement(&theme, "component.alert.description");
        let el = cx
            .foreground_scope(muted_foreground_color(&theme), move |_cx| self.children)
            .inherit_text_style(description_style);
        let Some(test_id) = self.test_id else {
            return el;
        };
        el.attach_semantics(SemanticsDecoration::default().test_id(test_id))
    }
}

/// Slot that only renders when approval is requested (`approval-requested`).
#[derive(Debug)]
pub struct ConfirmationRequest {
    state: Option<ToolUiPartState>,
    children: Vec<AnyElement>,
    layout: LayoutRefinement,
    test_id: Option<Arc<str>>,
}

impl ConfirmationRequest {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            state: None,
            children: children.into_iter().collect(),
            layout: LayoutRefinement::default().w_full().min_w_0(),
            test_id: None,
        }
    }

    pub fn state(mut self, state: ToolUiPartState) -> Self {
        self.state = Some(state);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let state = self
            .state
            .or_else(|| use_confirmation_context(cx).map(|context| context.state));
        let el = self.materialize(cx);
        match state {
            Some(ToolUiPartState::ApprovalRequested) => el,
            Some(_) => hidden(cx),
            None => deferred_slot(cx, CONFIRMATION_REQUEST_SLOT_KEY, el),
        }
    }

    fn materialize<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let mut children = self.children;
        let el = match children.len() {
            0 => ui::v_stack(|_cx| Vec::<AnyElement>::new())
                .layout(self.layout)
                .gap(Space::N0)
                .items(Items::Start)
                .into_element(cx),
            1 => children
                .pop()
                .expect("expected exactly one child after len() check"),
            _ => ui::v_stack(move |_cx| children)
                .layout(self.layout)
                .gap(Space::N0)
                .items(Items::Start)
                .into_element(cx),
        };
        let Some(test_id) = self.test_id else {
            return el;
        };
        el.attach_semantics(SemanticsDecoration::default().test_id(test_id))
    }
}

/// Slot that only renders when approved and in response states.
#[derive(Debug)]
pub struct ConfirmationAccepted {
    approval: Option<ToolUiPartApproval>,
    state: Option<ToolUiPartState>,
    children: Vec<AnyElement>,
    layout: LayoutRefinement,
    test_id: Option<Arc<str>>,
}

impl ConfirmationAccepted {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            approval: None,
            state: None,
            children: children.into_iter().collect(),
            layout: LayoutRefinement::default(),
            test_id: None,
        }
    }

    pub fn approval(mut self, approval: ToolUiPartApproval) -> Self {
        self.approval = Some(approval);
        self
    }

    pub fn state(mut self, state: ToolUiPartState) -> Self {
        self.state = Some(state);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let context = use_confirmation_context(cx);
        let approval = self
            .approval
            .clone()
            .or_else(|| context.as_ref().map(|context| context.approval.clone()));
        let state = self
            .state
            .or_else(|| context.as_ref().map(|context| context.state));
        let el = self.materialize(cx);
        match (approval.as_ref().and_then(|a| a.approved), state) {
            (Some(true), Some(state)) if state.is_response_state() => el,
            (Some(_), Some(_)) => hidden(cx),
            _ => deferred_slot(cx, CONFIRMATION_ACCEPTED_SLOT_KEY, el),
        }
    }

    fn materialize<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let mut children = self.children;
        let el = match children.len() {
            0 => ui::h_row(|_cx| Vec::<AnyElement>::new())
                .layout(self.layout)
                .gap(Space::N1)
                .items(Items::Center)
                .into_element(cx),
            1 => children
                .pop()
                .expect("expected exactly one child after len() check"),
            _ => ui::h_row(move |_cx| children)
                .layout(self.layout)
                .gap(Space::N1)
                .items(Items::Center)
                .into_element(cx),
        };
        let Some(test_id) = self.test_id else {
            return el;
        };
        el.attach_semantics(SemanticsDecoration::default().test_id(test_id))
    }
}

/// Slot that only renders when rejected and in response states.
#[derive(Debug)]
pub struct ConfirmationRejected {
    approval: Option<ToolUiPartApproval>,
    state: Option<ToolUiPartState>,
    children: Vec<AnyElement>,
    layout: LayoutRefinement,
    test_id: Option<Arc<str>>,
}

impl ConfirmationRejected {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            approval: None,
            state: None,
            children: children.into_iter().collect(),
            layout: LayoutRefinement::default(),
            test_id: None,
        }
    }

    pub fn approval(mut self, approval: ToolUiPartApproval) -> Self {
        self.approval = Some(approval);
        self
    }

    pub fn state(mut self, state: ToolUiPartState) -> Self {
        self.state = Some(state);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let context = use_confirmation_context(cx);
        let approval = self
            .approval
            .clone()
            .or_else(|| context.as_ref().map(|context| context.approval.clone()));
        let state = self
            .state
            .or_else(|| context.as_ref().map(|context| context.state));
        let el = self.materialize(cx);
        match (approval.as_ref().and_then(|a| a.approved), state) {
            (Some(false), Some(state)) if state.is_response_state() => el,
            (Some(_), Some(_)) => hidden(cx),
            _ => deferred_slot(cx, CONFIRMATION_REJECTED_SLOT_KEY, el),
        }
    }

    fn materialize<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let mut children = self.children;
        let el = match children.len() {
            0 => ui::h_row(|_cx| Vec::<AnyElement>::new())
                .layout(self.layout)
                .gap(Space::N1)
                .items(Items::Center)
                .into_element(cx),
            1 => children
                .pop()
                .expect("expected exactly one child after len() check"),
            _ => ui::h_row(move |_cx| children)
                .layout(self.layout)
                .gap(Space::N1)
                .items(Items::Center)
                .into_element(cx),
        };
        let Some(test_id) = self.test_id else {
            return el;
        };
        el.attach_semantics(SemanticsDecoration::default().test_id(test_id))
    }
}

/// Actions slot aligned with AI Elements `ConfirmationActions`.
#[derive(Debug)]
pub struct ConfirmationActions {
    state: Option<ToolUiPartState>,
    children: Vec<AnyElement>,
    layout: LayoutRefinement,
    test_id: Option<Arc<str>>,
}

impl ConfirmationActions {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            state: None,
            children: children.into_iter().collect(),
            layout: LayoutRefinement::default(),
            test_id: None,
        }
    }

    pub fn state(mut self, state: ToolUiPartState) -> Self {
        self.state = Some(state);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let state = self
            .state
            .or_else(|| use_confirmation_context(cx).map(|context| context.state));
        let el = self.materialize(cx);
        match state {
            Some(ToolUiPartState::ApprovalRequested) => el,
            Some(_) => hidden(cx),
            None => deferred_slot(cx, CONFIRMATION_ACTIONS_SLOT_KEY, el),
        }
    }

    fn materialize<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let mut el = ui::h_row(|_cx| self.children)
            .layout(self.layout)
            .gap(Space::N2)
            .justify(Justify::End)
            .items(Items::Center)
            .into_element(cx);
        if let fret_ui::element::ElementKind::Container(props) = &mut el.kind {
            if matches!(props.layout.size.width, fret_ui::element::Length::Auto) {
                props.layout.flex.align_self = Some(fret_ui::element::CrossAlign::End);
            }
        }
        let Some(test_id) = self.test_id else {
            return el;
        };
        el.attach_semantics(SemanticsDecoration::default().test_id(test_id))
    }
}

/// Action button aligned with AI Elements `ConfirmationAction`.
pub struct ConfirmationAction {
    label: Arc<str>,
    children: Vec<AnyElement>,
    variant: ButtonVariant,
    action: Option<ActionId>,
    action_payload: Option<ActionPayloadFactory>,
    on_activate: Option<fret_ui::action::OnActivate>,
    disabled: bool,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for ConfirmationAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConfirmationAction")
            .field("label", &self.label)
            .field("children_len", &self.children.len())
            .field("variant", &self.variant)
            .field("action", &self.action)
            .field("action_payload", &self.action_payload.is_some())
            .field("has_on_activate", &self.on_activate.is_some())
            .field("disabled", &self.disabled)
            .field("test_id", &self.test_id.as_deref())
            .finish()
    }
}

impl ConfirmationAction {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            children: Vec::new(),
            variant: ButtonVariant::Default,
            action: None,
            action_payload: None,
            on_activate: None,
            disabled: false,
            test_id: None,
        }
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Overrides the visible button content while keeping the base label for semantics.
    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    /// Appends one custom child to the visible button content override.
    pub fn child(mut self, child: AnyElement) -> Self {
        self.children.push(child);
        self
    }

    /// Bind a stable action ID to this confirmation action (action-first authoring).
    pub fn action(mut self, action: impl Into<fret_runtime::ActionId>) -> Self {
        self.action = Some(action.into());
        self
    }

    /// Attach a payload for parameterized confirmation actions (ADR 0312).
    pub fn action_payload<T>(mut self, payload: T) -> Self
    where
        T: std::any::Any + Send + Sync + Clone + 'static,
    {
        let payload = Arc::new(payload);
        self.action_payload = Some(Arc::new(move || Box::new(payload.as_ref().clone())));
        self
    }

    /// Like [`ConfirmationAction::action_payload`], but computes the payload lazily.
    pub fn action_payload_factory(mut self, payload: ActionPayloadFactory) -> Self {
        self.action_payload = Some(payload);
        self
    }

    pub fn on_activate(mut self, on_activate: fret_ui::action::OnActivate) -> Self {
        self.on_activate = Some(on_activate);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let children = self.children;
        let mut button = Button::new(self.label)
            .variant(self.variant)
            .size(ButtonSize::Sm);
        if !children.is_empty() {
            button = button.children(children);
        }
        if let Some(action) = self.action {
            button = button.action(action);
        }
        if let Some(payload) = self.action_payload {
            button = button.action_payload_factory(payload);
        }
        if let Some(on_activate) = self.on_activate {
            button = button.on_activate(on_activate);
        }
        if self.disabled {
            button = button.disabled(true);
        }
        if let Some(test_id) = self.test_id {
            button = button.test_id(test_id);
        }
        button.into_element(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{
        AppWindowId, Axis, Point, Px, Rect, SemanticsRole, Size, TextLineHeightPolicy,
    };
    use fret_ui::element::{CrossAlign, ElementKind, Length, TextProps};

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(200.0)),
        )
    }

    fn has_test_id(element: &AnyElement, test_id: &str) -> bool {
        if element
            .semantics_decoration
            .as_ref()
            .and_then(|d| d.test_id.as_deref())
            == Some(test_id)
        {
            return true;
        }

        element
            .children
            .iter()
            .any(|child| has_test_id(child, test_id))
    }

    #[test]
    fn confirmation_keeps_alert_role_when_stamping_test_id() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let approval = ToolUiPartApproval::new("approval-1");
        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                Confirmation::new(ToolUiPartState::ApprovalRequested)
                    .approval(approval)
                    .children([cx.text("Hello")])
                    .test_id("ui-ai-confirmation-root")
                    .into_element(cx)
            });

        assert_eq!(
            element.semantics_decoration.as_ref().and_then(|d| d.role),
            Some(SemanticsRole::Alert)
        );
        assert_eq!(
            element
                .semantics_decoration
                .as_ref()
                .and_then(|d| d.test_id.as_deref()),
            Some("ui-ai-confirmation-root")
        );
    }

    #[test]
    fn confirmation_request_defaults_to_vstack_for_multiple_children() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                ConfirmationRequest::new([cx.text("A"), cx.text("B")])
                    .state(ToolUiPartState::ApprovalRequested)
                    .into_element(cx)
            });

        let child = element
            .children
            .first()
            .expect("expected ConfirmationRequest container child");
        let ElementKind::Flex(props) = &child.kind else {
            panic!("expected ConfirmationRequest inner child to be a flex container");
        };
        assert_eq!(props.direction, Axis::Vertical);
    }

    #[test]
    fn confirmation_accepted_defaults_to_hstack_for_multiple_children() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let approval = ToolUiPartApproval::new("approval-1").approved(true);
        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                ConfirmationAccepted::new([cx.text("Ok"), cx.text("Done")])
                    .approval(approval)
                    .state(ToolUiPartState::ApprovalResponded)
                    .into_element(cx)
            });

        let child = element
            .children
            .first()
            .expect("expected ConfirmationAccepted container child");
        let ElementKind::Flex(props) = &child.kind else {
            panic!("expected ConfirmationAccepted inner child to be a flex container");
        };
        assert_eq!(props.direction, Axis::Horizontal);
    }

    #[test]
    fn confirmation_actions_default_to_self_end_without_forcing_full_width() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                ConfirmationActions::new([cx.text("Approve")])
                    .state(ToolUiPartState::ApprovalRequested)
                    .into_element(cx)
            });

        let ElementKind::Container(container) = &element.kind else {
            panic!("expected ConfirmationActions to render as a container");
        };
        let child = element
            .children
            .first()
            .expect("expected ConfirmationActions inner flex child");
        let ElementKind::Flex(props) = &child.kind else {
            panic!("expected ConfirmationActions inner child to be a flex container");
        };
        assert!(matches!(container.layout.size.width, Length::Auto));
        assert_eq!(container.layout.flex.align_self, Some(CrossAlign::End));
        assert_eq!(props.direction, Axis::Horizontal);
    }

    #[test]
    fn confirmation_children_can_consume_inherited_context() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let approval = ToolUiPartApproval::new("approval-1").approved(true);
        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                Confirmation::new(ToolUiPartState::ApprovalResponded)
                    .approval(approval)
                    .into_element_with_children(cx, |cx| {
                        vec![
                            ConfirmationTitle::new([
                                ConfirmationRequest::new([cx.text("Ask")])
                                    .test_id("request")
                                    .into_element(cx),
                                ConfirmationAccepted::new([cx.text("Approved")])
                                    .test_id("accepted")
                                    .into_element(cx),
                                ConfirmationRejected::new([cx.text("Rejected")])
                                    .test_id("rejected")
                                    .into_element(cx),
                            ])
                            .into_element(cx),
                            ConfirmationActions::new([cx.text("Actions")])
                                .test_id("actions")
                                .into_element(cx),
                        ]
                    })
            });

        assert!(has_test_id(&element, "accepted"));
        assert!(!has_test_id(&element, "request"));
        assert!(!has_test_id(&element, "rejected"));
        assert!(!has_test_id(&element, "actions"));
    }

    #[test]
    fn confirmation_direct_children_resolve_request_and_actions() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let approval = ToolUiPartApproval::new("approval-1");
        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                Confirmation::new(ToolUiPartState::ApprovalRequested)
                    .approval(approval)
                    .children([
                        ConfirmationTitle::new([
                            ConfirmationRequest::new([cx.text("Ask")])
                                .test_id("request")
                                .into_element(cx),
                            ConfirmationAccepted::new([cx.text("Approved")])
                                .test_id("accepted")
                                .into_element(cx),
                            ConfirmationRejected::new([cx.text("Rejected")])
                                .test_id("rejected")
                                .into_element(cx),
                        ])
                        .into_element(cx),
                        ConfirmationActions::new([cx.text("Actions")])
                            .test_id("actions")
                            .into_element(cx),
                    ])
                    .into_element(cx)
            });

        assert!(has_test_id(&element, "request"));
        assert!(has_test_id(&element, "actions"));
        assert!(!has_test_id(&element, "accepted"));
        assert!(!has_test_id(&element, "rejected"));
    }

    #[test]
    fn confirmation_direct_children_resolve_response_slots() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let approval = ToolUiPartApproval::new("approval-1").approved(true);
        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                Confirmation::new(ToolUiPartState::ApprovalResponded)
                    .approval(approval)
                    .children([
                        ConfirmationTitle::new([
                            ConfirmationRequest::new([cx.text("Ask")])
                                .test_id("request")
                                .into_element(cx),
                            ConfirmationAccepted::new([cx.text("Approved")])
                                .test_id("accepted")
                                .into_element(cx),
                            ConfirmationRejected::new([cx.text("Rejected")])
                                .test_id("rejected")
                                .into_element(cx),
                        ])
                        .into_element(cx),
                        ConfirmationActions::new([cx.text("Actions")])
                            .test_id("actions")
                            .into_element(cx),
                    ])
                    .into_element(cx)
            });

        assert!(has_test_id(&element, "accepted"));
        assert!(!has_test_id(&element, "request"));
        assert!(!has_test_id(&element, "rejected"));
        assert!(!has_test_id(&element, "actions"));
    }

    fn find_text<'a>(element: &'a AnyElement, text: &str) -> Option<&'a TextProps> {
        match &element.kind {
            ElementKind::Text(props) if props.text.as_ref() == text => return Some(props),
            _ => {}
        }

        element
            .children
            .iter()
            .find_map(|child| find_text(child, text))
    }

    #[test]
    fn confirmation_title_scopes_alert_description_typography_without_patching_nested_plain_text() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                ConfirmationTitle::new([ConfirmationRequest::new([cx.text("Ask")])
                    .state(ToolUiPartState::ApprovalRequested)
                    .into_element(cx)])
                .into_element(cx)
            });

        let text = find_text(&element, "Ask").expect("expected nested text node");
        assert!(
            text.style.is_none(),
            "expected nested passive text to inherit typography at runtime instead of being patched"
        );

        let style = element
            .inherited_text_style
            .as_ref()
            .expect("expected confirmation title root to carry inherited text style");

        let theme = Theme::global(&app).snapshot();
        let expected_px = theme
            .metric_by_key("component.alert.description_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or_else(|| theme.metric_token("font.size"));
        let expected_line_height = theme
            .metric_by_key("component.alert.description_line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or_else(|| theme.metric_token("font.line_height"));

        assert_eq!(style.size, Some(expected_px));
        assert_eq!(style.line_height, Some(expected_line_height));
        assert_eq!(
            style.line_height_policy,
            Some(TextLineHeightPolicy::FixedFromStyle)
        );
    }

    #[test]
    fn confirmation_action_accepts_custom_children_without_reintroducing_label_text() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                ConfirmationAction::new("Approve")
                    .children([cx.text("Run tool")])
                    .into_element(cx)
            });

        assert!(
            find_text(&element, "Run tool").is_some(),
            "expected custom confirmation action child text to render"
        );
        assert!(
            find_text(&element, "Approve").is_none(),
            "expected custom children to replace the default visible label text"
        );
    }
}
