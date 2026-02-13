//! AI Elements-aligned `Confirmation` surfaces.

use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_ui::element::{AnyElement, InteractivityGateProps, LayoutStyle, SemanticsDecoration};
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::declarative::stack;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::{Alert, AlertDescription, Button, ButtonSize};

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
#[derive(Debug, Clone)]
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

        let alert = Alert::new(self.children)
            .refine_layout(self.layout)
            .into_element(cx);

        let Some(test_id) = self.test_id else {
            return alert;
        };
        alert.attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Group)
                .test_id(test_id),
        )
    }
}

/// Inline title aligned with AI Elements `ConfirmationTitle`.
#[derive(Debug, Clone)]
pub struct ConfirmationTitle {
    text: Arc<str>,
}

impl ConfirmationTitle {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        AlertDescription::new(self.text).into_element(cx)
    }
}

/// Slot that only renders when approval is requested (`approval-requested`).
#[derive(Debug, Clone)]
pub struct ConfirmationRequest {
    state: ToolUiPartState,
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
}

impl ConfirmationRequest {
    pub fn new(state: ToolUiPartState, children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            state,
            children: children.into_iter().collect(),
            test_id: None,
        }
    }

    pub fn test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        if self.state != ToolUiPartState::ApprovalRequested {
            return hidden(cx);
        }
        let el = stack::vstack(cx, stack::VStackProps::default().gap(Space::N1), |_cx| {
            self.children
        });
        let Some(test_id) = self.test_id else {
            return el;
        };
        el.attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Group)
                .test_id(test_id),
        )
    }
}

/// Slot that only renders when approved and in response states.
#[derive(Debug, Clone)]
pub struct ConfirmationAccepted {
    approval: Option<ToolUiPartApproval>,
    state: ToolUiPartState,
    children: Vec<AnyElement>,
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
            test_id: None,
        }
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
        let el = stack::vstack(cx, stack::VStackProps::default().gap(Space::N1), |_cx| {
            self.children
        });
        let Some(test_id) = self.test_id else {
            return el;
        };
        el.attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Group)
                .test_id(test_id),
        )
    }
}

/// Slot that only renders when rejected and in response states.
#[derive(Debug, Clone)]
pub struct ConfirmationRejected {
    approval: Option<ToolUiPartApproval>,
    state: ToolUiPartState,
    children: Vec<AnyElement>,
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
            test_id: None,
        }
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
        let el = stack::vstack(cx, stack::VStackProps::default().gap(Space::N1), |_cx| {
            self.children
        });
        let Some(test_id) = self.test_id else {
            return el;
        };
        el.attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Group)
                .test_id(test_id),
        )
    }
}

/// Actions slot aligned with AI Elements `ConfirmationActions`.
#[derive(Debug, Clone)]
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

        let el = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(self.layout)
                .gap(Space::N2)
                .justify_end()
                .items_center(),
            |_cx| self.children,
        );
        let Some(test_id) = self.test_id else {
            return el;
        };
        el.attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Group)
                .test_id(test_id),
        )
    }
}

/// Action button aligned with AI Elements `ConfirmationAction`.
#[derive(Clone)]
pub struct ConfirmationAction {
    label: Arc<str>,
    on_activate: Option<fret_ui::action::OnActivate>,
    disabled: bool,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for ConfirmationAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConfirmationAction")
            .field("label", &self.label)
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
            on_activate: None,
            disabled: false,
            test_id: None,
        }
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
        let mut button = Button::new(self.label).size(ButtonSize::Sm);
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
