//! AI Elements-aligned `Confirmation` surfaces.

use std::sync::Arc;

use fret_ui::element::{AnyElement, InteractivityGateProps, LayoutStyle, SemanticsDecoration};
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::ui;
use fret_ui_kit::{Items, Justify, LayoutRefinement, Space};
use fret_ui_shadcn::{Alert, AlertDescription, Button, ButtonSize, ButtonVariant};

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
        if self.approval.is_none() || self.state.is_input_state() {
            return hidden(cx);
        }

        // AI Elements overrides shadcn/ui Alert's default grid layout with `flex flex-col gap-2`.
        // Our Alert implementation has its own internal gap policy, so we model the AI Elements
        // outcome by wrapping the confirmation children in a single vstack with the desired gap.
        //
        // Note: this intentionally disables Alert's "icon as first child" heuristics for this
        // surface, matching the AI Elements Confirmation composition.
        let body = ui::v_stack(|_cx| self.children)
            .gap(Space::N2)
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .into_element(cx);

        let alert = Alert::new([body])
            .refine_layout(self.layout)
            .into_element(cx);

        let Some(test_id) = self.test_id else {
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
        let el = AlertDescription::new_children(self.children).into_element(cx);
        let Some(test_id) = self.test_id else {
            return el;
        };
        el.attach_semantics(SemanticsDecoration::default().test_id(test_id))
    }
}

/// Slot that only renders when approval is requested (`approval-requested`).
#[derive(Debug)]
pub struct ConfirmationRequest {
    state: ToolUiPartState,
    children: Vec<AnyElement>,
    layout: LayoutRefinement,
    test_id: Option<Arc<str>>,
}

impl ConfirmationRequest {
    pub fn new(state: ToolUiPartState, children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            state,
            children: children.into_iter().collect(),
            layout: LayoutRefinement::default().w_full().min_w_0(),
            test_id: None,
        }
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
        if self.state != ToolUiPartState::ApprovalRequested {
            return hidden(cx);
        }
        let mut children = self.children;
        let el = match children.len() {
            0 => ui::v_stack(|_cx| Vec::<AnyElement>::new())
                .layout(self.layout)
                .gap(Space::N1)
                .items(Items::Start)
                .into_element(cx),
            1 => {
                children
                    .pop()
                    .expect("expected exactly one child after len() check")
            }
            _ => ui::v_stack(move |_cx| children)
                .layout(self.layout)
                .gap(Space::N1)
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
    state: ToolUiPartState,
    children: Vec<AnyElement>,
    layout: LayoutRefinement,
    test_id: Option<Arc<str>>,
}

impl ConfirmationAccepted {
    pub fn new(
        approval: Option<ToolUiPartApproval>,
        state: ToolUiPartState,
        children: impl IntoIterator<Item = AnyElement>,
    ) -> Self {
        Self {
            approval,
            state,
            children: children.into_iter().collect(),
            layout: LayoutRefinement::default().w_full().min_w_0(),
            test_id: None,
        }
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
        let approved = self.approval.as_ref().and_then(|a| a.approved) == Some(true);
        if !approved || !self.state.is_response_state() {
            return hidden(cx);
        }
        let mut children = self.children;
        let el = match children.len() {
            0 => ui::h_row(|_cx| Vec::<AnyElement>::new())
                .layout(self.layout)
                .gap(Space::N1)
                .items(Items::Center)
                .into_element(cx),
            1 => {
                children
                    .pop()
                    .expect("expected exactly one child after len() check")
            }
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
    state: ToolUiPartState,
    children: Vec<AnyElement>,
    layout: LayoutRefinement,
    test_id: Option<Arc<str>>,
}

impl ConfirmationRejected {
    pub fn new(
        approval: Option<ToolUiPartApproval>,
        state: ToolUiPartState,
        children: impl IntoIterator<Item = AnyElement>,
    ) -> Self {
        Self {
            approval,
            state,
            children: children.into_iter().collect(),
            layout: LayoutRefinement::default().w_full().min_w_0(),
            test_id: None,
        }
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
        let rejected = self.approval.as_ref().and_then(|a| a.approved) == Some(false);
        if !rejected || !self.state.is_response_state() {
            return hidden(cx);
        }
        let mut children = self.children;
        let el = match children.len() {
            0 => ui::h_row(|_cx| Vec::<AnyElement>::new())
                .layout(self.layout)
                .gap(Space::N1)
                .items(Items::Center)
                .into_element(cx),
            1 => {
                children
                    .pop()
                    .expect("expected exactly one child after len() check")
            }
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
    state: ToolUiPartState,
    children: Vec<AnyElement>,
    layout: LayoutRefinement,
    test_id: Option<Arc<str>>,
}

impl ConfirmationActions {
    pub fn new(state: ToolUiPartState, children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            state,
            children: children.into_iter().collect(),
            layout: LayoutRefinement::default().w_full().min_w_0(),
            test_id: None,
        }
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
        if self.state != ToolUiPartState::ApprovalRequested {
            return hidden(cx);
        }

        let el = ui::h_row(|_cx| self.children)
            .layout(self.layout)
            .gap(Space::N2)
            .justify(Justify::End)
            .items(Items::Center)
            .into_element(cx);
        let Some(test_id) = self.test_id else {
            return el;
        };
        el.attach_semantics(SemanticsDecoration::default().test_id(test_id))
    }
}

/// Action button aligned with AI Elements `ConfirmationAction`.
#[derive(Clone)]
pub struct ConfirmationAction {
    label: Arc<str>,
    variant: ButtonVariant,
    on_activate: Option<fret_ui::action::OnActivate>,
    disabled: bool,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for ConfirmationAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConfirmationAction")
            .field("label", &self.label)
            .field("variant", &self.variant)
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
            variant: ButtonVariant::Default,
            on_activate: None,
            disabled: false,
            test_id: None,
        }
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
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
        let mut button = Button::new(self.label)
            .variant(self.variant)
            .size(ButtonSize::Sm);
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
    use fret_core::{AppWindowId, Point, Px, Rect, SemanticsRole, Size};
    use fret_ui::element::ElementKind;

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(200.0)),
        )
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
                ConfirmationRequest::new(
                    ToolUiPartState::ApprovalRequested,
                    [cx.text("A"), cx.text("B")],
                )
                .into_element(cx)
            });

        assert!(matches!(element.kind, ElementKind::Column(_)));
    }

    #[test]
    fn confirmation_accepted_defaults_to_hstack_for_multiple_children() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let approval = ToolUiPartApproval::new("approval-1").approved(true);
        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                ConfirmationAccepted::new(
                    Some(approval),
                    ToolUiPartState::ApprovalResponded,
                    [cx.text("Ok"), cx.text("Done")],
                )
                .into_element(cx)
            });

        assert!(matches!(element.kind, ElementKind::Row(_)));
    }
}
