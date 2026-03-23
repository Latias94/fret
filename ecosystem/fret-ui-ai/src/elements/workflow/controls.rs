use std::any::Any;
use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_icons::IconId;
use fret_runtime::CommandId;
use fret_ui::action::OnActivate;
use fret_ui::element::{AnyElement, SemanticsProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{
    ChromeRefinement, ColorFallback, ColorRef, Items, LayoutRefinement, Radius, Space, ui,
};
use fret_ui_shadcn::facade::{Button, ButtonSize, ButtonVariant};

type ActionPayloadFactory = Arc<dyn Fn() -> Box<dyn Any + Send + Sync> + 'static>;

/// AI Elements-aligned workflow `Controls` chrome (UI-only).
///
/// Upstream reference: `repo-ref/ai-elements/packages/elements/src/controls.tsx`.
///
/// Notes:
/// - Upstream is `@xyflow/react`-backed (`ControlsPrimitive`).
/// - In Fret this is a styling/composition wrapper only; apps own the actual zoom/pan behavior.
pub struct WorkflowControls {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl std::fmt::Debug for WorkflowControls {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkflowControls")
            .field("children_len", &self.children.len())
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .field("chrome", &self.chrome)
            .finish()
    }
}

impl WorkflowControls {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            test_id: None,
            layout: LayoutRefinement::default().min_w_0().overflow_hidden(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let base_chrome = ChromeRefinement::default()
            .rounded(Radius::Md)
            .border_1()
            .bg(ColorRef::Token {
                key: "card",
                fallback: ColorFallback::ThemePanelBackground,
            })
            .border_color(ColorRef::Token {
                key: "border",
                fallback: ColorFallback::ThemePanelBorder,
            })
            .p(Space::N1);

        let children = self.children;
        let content = ui::v_stack(move |_cx| children)
            .gap(Space::N0)
            .items(Items::Stretch)
            .layout(LayoutRefinement::default().min_w_0())
            .into_element(cx);

        let props =
            decl_style::container_props(&theme, base_chrome.merge(self.chrome), self.layout);
        let body = cx.container(props, move |_cx| [content]);

        let Some(test_id) = self.test_id else {
            return body;
        };
        cx.semantics(
            SemanticsProps {
                role: SemanticsRole::Group,
                test_id: Some(test_id),
                ..Default::default()
            },
            move |_cx| [body],
        )
    }
}

/// A shadcn-skinned icon button intended for [`WorkflowControls`] children.
#[derive(Clone)]
pub struct WorkflowControlsButton {
    label: Arc<str>,
    icon: IconId,
    disabled: bool,
    test_id: Option<Arc<str>>,
    command: Option<CommandId>,
    action_payload: Option<ActionPayloadFactory>,
    on_activate: Option<OnActivate>,
}

impl std::fmt::Debug for WorkflowControlsButton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkflowControlsButton")
            .field("label", &self.label.as_ref())
            .field("icon", &self.icon)
            .field("disabled", &self.disabled)
            .field("test_id", &self.test_id.as_deref())
            .field("command", &self.command)
            .field("action_payload", &self.action_payload.is_some())
            .field("on_activate", &self.on_activate.is_some())
            .finish()
    }
}

impl WorkflowControlsButton {
    pub fn new(label: impl Into<Arc<str>>, icon: IconId) -> Self {
        Self {
            label: label.into(),
            icon,
            disabled: false,
            test_id: None,
            command: None,
            action_payload: None,
            on_activate: None,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    /// Bind a stable action ID to this workflow controls button (action-first authoring).
    pub fn action(mut self, action: impl Into<fret_runtime::ActionId>) -> Self {
        self.command = Some(action.into());
        self
    }

    /// Attach a payload for parameterized workflow-controls actions (ADR 0312).
    pub fn action_payload<T>(mut self, payload: T) -> Self
    where
        T: Any + Send + Sync + Clone + 'static,
    {
        let payload = Arc::new(payload);
        self.action_payload = Some(Arc::new(move || Box::new(payload.as_ref().clone())));
        self
    }

    /// Like [`WorkflowControlsButton::action_payload`], but computes the payload lazily.
    pub fn action_payload_factory(mut self, payload: ActionPayloadFactory) -> Self {
        self.action_payload = Some(payload);
        self
    }

    pub fn on_click(mut self, command: impl Into<CommandId>) -> Self {
        self.command = Some(command.into());
        self
    }

    pub fn on_activate(mut self, on_activate: OnActivate) -> Self {
        self.on_activate = Some(on_activate);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let icon = decl_icon::icon(cx, self.icon);

        let mut btn = Button::new(self.label)
            .size(ButtonSize::IconSm)
            .variant(ButtonVariant::Ghost)
            .children([icon])
            .disabled(self.disabled);

        if let Some(command) = self.command {
            btn = btn.on_click(command);
        }
        if let Some(payload) = self.action_payload {
            btn = btn.action_payload_factory(payload);
        }

        if let Some(on_activate) = self.on_activate {
            btn = btn.on_activate(on_activate);
        }

        if let Some(test_id) = self.test_id {
            btn = btn.test_id(test_id);
        }

        btn.into_element(cx)
    }
}
