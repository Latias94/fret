//! AI Elements-aligned `ModelSelector` surfaces.
//!
//! Upstream reference: `repo-ref/ai-elements/packages/elements/src/model-selector.tsx`.
//!
//! Notes:
//! - The upstream implementation is a thin wrapper around shadcn/ui `Dialog` + `Command*`.
//! - Fret's cmdk-aligned surface is primarily `fret-ui-shadcn::CommandDialog` / `CommandPalette`,
//!   so this module focuses on matching outcomes while offering a Rust-friendly decomposition.
//! - Upstream provider logos are loaded from `https://models.dev/logos/<provider>.svg`.
//!   The Fret port renders a local placeholder badge by default (apps can swap in their own icon).

use std::sync::Arc;

use fret_core::{Color, Px, SemanticsRole};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, UiActionHost};
use fret_ui::element::{AnyElement, SemanticsDecoration};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::declarative::visually_hidden::visually_hidden;
use fret_ui_kit::ui;
use fret_ui_kit::{ChromeRefinement, ColorRef, Items, Justify, LayoutRefinement, Radius, Space};
use fret_ui_shadcn::facade::{
    Command, CommandDialog, CommandEmpty, CommandEntry, CommandGroup, CommandInput, CommandItem,
    CommandList, CommandSeparator, CommandShortcut, Dialog, DialogContent, DialogTitle,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelSelectorChildSlot {
    Trigger,
    Content,
}

#[derive(Clone)]
pub struct ModelSelectorController {
    pub open: Model<bool>,
}

impl std::fmt::Debug for ModelSelectorController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModelSelectorController")
            .field("open", &self.open.id())
            .finish()
    }
}

pub fn use_model_selector_controller<H: UiHost>(
    cx: &ElementContext<'_, H>,
) -> Option<ModelSelectorController> {
    cx.provided::<ModelSelectorController>().cloned()
}

/// AI Elements-aligned `ModelSelector` root.
#[derive(Clone)]
pub struct ModelSelector {
    open: Option<Model<bool>>,
    default_open: bool,
}

impl std::fmt::Debug for ModelSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModelSelector")
            .field("open", &self.open.as_ref().map(|model| model.id()))
            .field("default_open", &self.default_open)
            .finish()
    }
}

impl ModelSelector {
    pub fn new() -> Self {
        Self {
            open: None,
            default_open: false,
        }
    }

    /// Controlled open model (Radix `open`).
    pub fn open_model(mut self, open: Model<bool>) -> Self {
        self.open = Some(open);
        self
    }

    /// Uncontrolled initial open value (Radix `defaultOpen`).
    pub fn default_open(mut self, default_open: bool) -> Self {
        self.default_open = default_open;
        self
    }

    /// Docs-shaped compound children composition aligned with upstream `<ModelSelector>...</ModelSelector>`.
    pub fn children<I, C>(self, children: I) -> ModelSelectorWithChildren
    where
        I: IntoIterator<Item = C>,
        C: Into<ModelSelectorChild>,
    {
        ModelSelectorWithChildren {
            root: self,
            children: children.into_iter().map(Into::into).collect(),
        }
    }

    pub fn trigger(self, trigger: ModelSelectorTrigger) -> ModelSelectorWithChildren {
        self.children([ModelSelectorChild::Trigger(trigger)])
    }

    pub fn content(self, content: ModelSelectorContent) -> ModelSelectorWithChildren {
        self.children([ModelSelectorChild::Content(content)])
    }

    fn resolved_open_model<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> Model<bool> {
        fret_ui_kit::primitives::dialog::DialogRoot::new()
            .open(self.open)
            .default_open(self.default_open)
            .open_model(cx)
    }

    /// Rust-friendly compound entrypoint for composing trigger + content in one closure.
    pub fn into_element_with_children<H, F>(
        self,
        cx: &mut ElementContext<'_, H>,
        children: F,
    ) -> AnyElement
    where
        H: UiHost,
        F: Fn(&mut ElementContext<'_, H>, ModelSelectorChildSlot, Model<bool>) -> AnyElement
            + Clone,
    {
        let open = self.resolved_open_model(cx);
        let controller = ModelSelectorController { open: open.clone() };
        let open_for_trigger = open.clone();
        let open_for_content = open.clone();
        let children_for_trigger = children.clone();
        let children_for_content = children;

        cx.provide(controller, |cx| {
            Dialog::new(open.clone()).into_element(
                cx,
                move |cx| {
                    children_for_trigger(
                        cx,
                        ModelSelectorChildSlot::Trigger,
                        open_for_trigger.clone(),
                    )
                },
                move |cx| {
                    children_for_content(
                        cx,
                        ModelSelectorChildSlot::Content,
                        open_for_content.clone(),
                    )
                },
            )
        })
    }

    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>, Model<bool>) -> AnyElement,
        content: impl FnOnce(&mut ElementContext<'_, H>, Model<bool>) -> AnyElement,
    ) -> AnyElement {
        let open = self.resolved_open_model(cx);
        let controller = ModelSelectorController { open: open.clone() };
        let open_for_trigger = open.clone();

        cx.provide(controller, |cx| {
            Dialog::new(open.clone()).into_element(
                cx,
                move |cx| trigger(cx, open_for_trigger.clone()),
                move |cx| content(cx, open.clone()),
            )
        })
    }
}

pub struct ModelSelectorWithChildren {
    root: ModelSelector,
    children: Vec<ModelSelectorChild>,
}

impl std::fmt::Debug for ModelSelectorWithChildren {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModelSelectorWithChildren")
            .field("root", &self.root)
            .field("children_len", &self.children.len())
            .finish()
    }
}

impl ModelSelectorWithChildren {
    pub fn trigger(mut self, trigger: ModelSelectorTrigger) -> Self {
        self.children.push(ModelSelectorChild::Trigger(trigger));
        self
    }

    pub fn content(mut self, content: ModelSelectorContent) -> Self {
        self.children.push(ModelSelectorChild::Content(content));
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let open = self.root.resolved_open_model(cx);
        let controller = ModelSelectorController { open: open.clone() };

        let mut trigger: Option<ModelSelectorTrigger> = None;
        let mut content: Option<ModelSelectorContent> = None;

        for child in self.children {
            match child {
                ModelSelectorChild::Trigger(next) => {
                    assert!(
                        trigger.replace(next).is_none(),
                        "ModelSelector::children(...) accepts at most one ModelSelectorTrigger"
                    );
                }
                ModelSelectorChild::Content(next) => {
                    assert!(
                        content.replace(next).is_none(),
                        "ModelSelector::children(...) accepts at most one ModelSelectorContent"
                    );
                }
            }
        }

        let trigger =
            trigger.expect("ModelSelector::children(...) requires one ModelSelectorTrigger");
        let content =
            content.expect("ModelSelector::children(...) requires one ModelSelectorContent");

        cx.provide(controller, |cx| {
            Dialog::new(open.clone()).into_element(
                cx,
                move |cx| trigger.into_element_with_open(cx, open.clone()),
                move |cx| content.into_element(cx),
            )
        })
    }
}

pub enum ModelSelectorChild {
    Trigger(ModelSelectorTrigger),
    Content(ModelSelectorContent),
}

impl std::fmt::Debug for ModelSelectorChild {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Trigger(value) => f.debug_tuple("Trigger").field(value).finish(),
            Self::Content(value) => f.debug_tuple("Content").field(value).finish(),
        }
    }
}

impl From<ModelSelectorTrigger> for ModelSelectorChild {
    fn from(value: ModelSelectorTrigger) -> Self {
        Self::Trigger(value)
    }
}

impl From<ModelSelectorContent> for ModelSelectorChild {
    fn from(value: ModelSelectorContent) -> Self {
        Self::Content(value)
    }
}

/// A shadcn `CommandDialog` wrapper aligned with AI Elements `ModelSelectorDialog`.
pub type ModelSelectorDialog = CommandDialog;

/// AI Elements-aligned `ModelSelectorTrigger`.
///
/// In upstream React this is `DialogTrigger`. In Fret we model it as a pressable wrapper that sets
/// the shared `open` model to `true`.
pub struct ModelSelectorTrigger {
    child: AnyElement,
    disabled: bool,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for ModelSelectorTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModelSelectorTrigger")
            .field("disabled", &self.disabled)
            .field("test_id", &self.test_id.as_deref())
            .finish()
    }
}

impl ModelSelectorTrigger {
    pub fn new(child: AnyElement) -> Self {
        Self {
            child,
            disabled: false,
            test_id: None,
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

    pub fn into_element_with_open<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        open: Model<bool>,
    ) -> AnyElement {
        let disabled = self.disabled;
        let test_id = self.test_id;
        let mut element = self.child;

        let mut semantics = SemanticsDecoration::default().role(SemanticsRole::Button);
        if let Some(test_id) = test_id {
            semantics = semantics.test_id(test_id);
        }
        if disabled {
            semantics = semantics.disabled(true);
        }
        element = element.attach_semantics(semantics);

        if !disabled {
            cx.pressable_add_on_activate_for(
                element.id,
                Arc::new(move |host, action_cx, _reason| {
                    let _ = host.models_mut().update(&open, |v| *v = true);
                    host.notify(action_cx);
                    host.request_redraw(action_cx.window);
                }),
            );
        }

        element
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(controller) = use_model_selector_controller(cx) else {
            return visually_hidden(cx, |_| Vec::<AnyElement>::new());
        };

        self.into_element_with_open(cx, controller.open)
    }
}

/// AI Elements-aligned `ModelSelectorContent`.
///
/// Upstream applies "no border + p-0" and renders the title as `sr-only`.
pub struct ModelSelectorContent {
    title: Arc<str>,
    children: Vec<AnyElement>,
    input: Option<ModelSelectorInput>,
    list: Option<ModelSelectorList>,
    test_id_root: Option<Arc<str>>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    command_chrome: ChromeRefinement,
    command_layout: LayoutRefinement,
}

impl std::fmt::Debug for ModelSelectorContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModelSelectorContent")
            .field("title", &self.title.as_ref())
            .field("children_len", &self.children.len())
            .field("has_input", &self.input.is_some())
            .field("has_list", &self.list.is_some())
            .field("test_id_root", &self.test_id_root.as_deref())
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .finish()
    }
}

impl ModelSelectorContent {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            title: Arc::from("Model Selector"),
            children: children.into_iter().collect(),
            input: None,
            list: None,
            test_id_root: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            command_chrome: ChromeRefinement::default()
                .border_width(Px(0.0))
                .rounded(Radius::Md),
            command_layout: LayoutRefinement::default().w_full().min_w_0(),
        }
    }

    pub fn title(mut self, title: impl Into<Arc<str>>) -> Self {
        self.title = title.into();
        self
    }

    pub fn input(mut self, input: ModelSelectorInput) -> Self {
        self.input = Some(input);
        self
    }

    pub fn list(mut self, list: ModelSelectorList) -> Self {
        self.list = Some(list);
        self
    }

    pub fn test_id_root(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_root = Some(id.into());
        self
    }

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn refine_command_style(mut self, chrome: ChromeRefinement) -> Self {
        self.command_chrome = self.command_chrome.merge(chrome);
        self
    }

    pub fn refine_command_layout(mut self, layout: LayoutRefinement) -> Self {
        self.command_layout = self.command_layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let title = self.title;
        let hidden_title =
            visually_hidden(cx, move |cx| vec![DialogTitle::new(title).into_element(cx)]);

        let mut command_children = self.children;
        if let Some(input) = self.input {
            command_children.push(input.into_element(cx));
        }
        if let Some(list) = self.list {
            command_children.push(list.into_element(cx));
        }

        let command = Command::new(command_children)
            .refine_style(self.command_chrome)
            .refine_layout(self.command_layout)
            .into_element(cx);

        let chrome = ChromeRefinement::default()
            .p(Space::N0)
            .border_1()
            .merge(self.chrome);

        let mut element = DialogContent::new([hidden_title, command])
            .refine_style(chrome)
            .refine_layout(self.layout)
            .into_element(cx);

        if let Some(test_id) = self.test_id_root {
            element = element.attach_semantics(SemanticsDecoration::default().test_id(test_id));
        }
        element
    }
}

/// AI Elements-aligned logo placeholder (local-only).
#[derive(Clone)]
pub struct ModelSelectorLogo {
    provider: Arc<str>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl std::fmt::Debug for ModelSelectorLogo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModelSelectorLogo")
            .field("provider", &self.provider.as_ref())
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .field("chrome", &self.chrome)
            .finish()
    }
}

impl ModelSelectorLogo {
    pub fn new(provider: impl Into<Arc<str>>) -> Self {
        Self {
            provider: provider.into(),
            test_id: None,
            layout: LayoutRefinement::default(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let bg = theme
            .color_by_key("background")
            .unwrap_or_else(|| theme.color_token("background"));
        let fg = theme
            .color_by_key("foreground")
            .unwrap_or_else(|| theme.color_token("foreground"));
        let border = theme
            .color_by_key("border")
            .unwrap_or_else(|| theme.color_token("border"));

        let size = Px(12.0);
        let initial: Arc<str> = self
            .provider
            .chars()
            .next()
            .map(|c| c.to_ascii_uppercase().to_string())
            .unwrap_or_else(|| "?".to_string())
            .into();

        let chrome = ChromeRefinement::default()
            .radius(Radius::Full)
            .bg(ColorRef::Color(bg))
            .border_1()
            .border_color(ColorRef::Color(border))
            .merge(self.chrome);

        let layout = LayoutRefinement::default()
            .w_px(size)
            .h_px(size)
            .flex_shrink_0()
            .merge(self.layout);

        let props = decl_style::container_props(&theme, chrome, layout);
        let mut element = cx.container(props, move |cx| {
            let text = fret_ui_kit::ui::text(initial)
                .text_size_px(Px(8.0))
                .font_medium()
                .nowrap()
                .text_color(ColorRef::Color(Color {
                    r: fg.r,
                    g: fg.g,
                    b: fg.b,
                    a: 0.8,
                }))
                .into_element(cx);
            vec![
                ui::h_row(move |_cx| vec![text])
                    .layout(LayoutRefinement::default().w_full().h_full())
                    .justify(Justify::Center)
                    .items(Items::Center)
                    .into_element(cx),
            ]
        });

        element = element.attach_semantics(SemanticsDecoration {
            role: Some(SemanticsRole::Generic),
            label: Some(Arc::from(format!("{} logo", self.provider))),
            ..Default::default()
        });

        if let Some(test_id) = self.test_id {
            element = element.attach_semantics(SemanticsDecoration::default().test_id(test_id));
        }
        element
    }
}

pub enum ModelSelectorLogoGroupChild {
    Logo(ModelSelectorLogo),
    Custom(AnyElement),
}

impl std::fmt::Debug for ModelSelectorLogoGroupChild {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Logo(value) => f.debug_tuple("Logo").field(value).finish(),
            Self::Custom(_) => f.write_str("Custom(..)"),
        }
    }
}

impl ModelSelectorLogoGroupChild {
    fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        match self {
            Self::Logo(value) => value.into_element(cx),
            Self::Custom(value) => value,
        }
    }
}

impl From<ModelSelectorLogo> for ModelSelectorLogoGroupChild {
    fn from(value: ModelSelectorLogo) -> Self {
        Self::Logo(value)
    }
}

impl From<AnyElement> for ModelSelectorLogoGroupChild {
    fn from(value: AnyElement) -> Self {
        Self::Custom(value)
    }
}

/// AI Elements-aligned logo group.
///
/// Upstream uses negative spacing (`-space-x-1`) to overlap the logos.
pub struct ModelSelectorLogoGroup {
    children: Vec<ModelSelectorLogoGroupChild>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    gap: Space,
    overlap_x: Space,
}

impl std::fmt::Debug for ModelSelectorLogoGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModelSelectorLogoGroup")
            .field("children_len", &self.children.len())
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .field("gap", &self.gap)
            .field("overlap_x", &self.overlap_x)
            .finish()
    }
}

impl ModelSelectorLogoGroup {
    pub fn new<I, C>(children: I) -> Self
    where
        I: IntoIterator<Item = C>,
        C: Into<ModelSelectorLogoGroupChild>,
    {
        Self {
            children: children.into_iter().map(Into::into).collect(),
            test_id: None,
            layout: LayoutRefinement::default().flex_shrink_0(),
            gap: Space::N0,
            overlap_x: Space::N1,
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

    pub fn gap(mut self, gap: Space) -> Self {
        self.gap = gap;
        self
    }

    pub fn overlap_x(mut self, overlap_x: Space) -> Self {
        self.overlap_x = overlap_x;
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let overlap_x = self.overlap_x;
        let mut children: Vec<AnyElement> = Vec::new();
        for (idx, child) in self.children.into_iter().enumerate() {
            let child = child.into_element(cx);
            if idx == 0 || overlap_x == Space::N0 {
                children.push(child);
            } else {
                children.push(
                    ui::container(move |_cx| vec![child])
                        .layout(LayoutRefinement::default().ml_neg(overlap_x))
                        .into_element(cx),
                );
            }
        }

        let mut element = ui::h_row(move |_cx| children)
            .layout(self.layout)
            .gap(self.gap)
            .items(Items::Center)
            .into_element(cx);

        if let Some(test_id) = self.test_id {
            element = element.attach_semantics(SemanticsDecoration::default().test_id(test_id));
        }
        element
    }
}

/// AI Elements-aligned name label.
#[derive(Clone)]
pub struct ModelSelectorName {
    text: Arc<str>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for ModelSelectorName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModelSelectorName")
            .field("text", &self.text.as_ref())
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .finish()
    }
}

impl ModelSelectorName {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
            test_id: None,
            layout: LayoutRefinement::default().flex_1().min_w_0(),
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let mut element = fret_ui_kit::ui::text(self.text)
            .layout(self.layout)
            .truncate()
            .into_element(cx);

        if let Some(test_id) = self.test_id {
            element = element.attach_semantics(SemanticsDecoration::default().test_id(test_id));
        }
        element
    }
}

/// Convenience action: close a `ModelSelector` dialog.
pub fn close_model_selector_dialog(
    host: &mut dyn UiActionHost,
    action_cx: ActionCx,
    open: &Model<bool>,
) {
    let _ = host.models_mut().update(open, |v| *v = false);
    host.request_redraw(action_cx.window);
}

/// AI Elements-aligned `ModelSelectorInput`.
///
/// Upstream applies `h-auto py-3.5` to the underlying shadcn `CommandInput`. This wrapper bakes
/// those defaults in while still exposing the same customization hooks (`placeholder`, `disabled`,
/// `refine_*`).
#[derive(Clone)]
pub struct ModelSelectorInput {
    inner: CommandInput,
}

impl std::fmt::Debug for ModelSelectorInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModelSelectorInput").finish_non_exhaustive()
    }
}

impl ModelSelectorInput {
    pub fn new(model: Model<String>) -> Self {
        Self {
            inner: CommandInput::new(model)
                .wrapper_height_auto()
                .input_height_auto()
                .input_padding_y_px(Px(14.0)),
        }
    }

    pub fn input_test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.inner = self.inner.input_test_id(test_id);
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.inner = self.inner.a11y_label(label);
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.inner = self.inner.placeholder(placeholder);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.inner = self.inner.disabled(disabled);
        self
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.inner = self.inner.refine_style(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.inner = self.inner.refine_layout(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.inner.into_element(cx)
    }
}

pub enum ModelSelectorItemChild {
    Logo(ModelSelectorLogo),
    LogoGroup(ModelSelectorLogoGroup),
    Name(ModelSelectorName),
    Shortcut(ModelSelectorShortcut),
    Custom(AnyElement),
}

impl std::fmt::Debug for ModelSelectorItemChild {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Logo(value) => f.debug_tuple("Logo").field(value).finish(),
            Self::LogoGroup(value) => f.debug_tuple("LogoGroup").field(value).finish(),
            Self::Name(value) => f.debug_tuple("Name").field(value).finish(),
            Self::Shortcut(_) => f.write_str("Shortcut(..)"),
            Self::Custom(_) => f.write_str("Custom(..)"),
        }
    }
}

impl ModelSelectorItemChild {
    fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        match self {
            Self::Logo(value) => value.into_element(cx),
            Self::LogoGroup(value) => value.into_element(cx),
            Self::Name(value) => value.into_element(cx),
            Self::Shortcut(value) => value.into_element(cx),
            Self::Custom(value) => value,
        }
    }
}

impl From<ModelSelectorLogo> for ModelSelectorItemChild {
    fn from(value: ModelSelectorLogo) -> Self {
        Self::Logo(value)
    }
}

impl From<ModelSelectorLogoGroup> for ModelSelectorItemChild {
    fn from(value: ModelSelectorLogoGroup) -> Self {
        Self::LogoGroup(value)
    }
}

impl From<ModelSelectorName> for ModelSelectorItemChild {
    fn from(value: ModelSelectorName) -> Self {
        Self::Name(value)
    }
}

impl From<ModelSelectorShortcut> for ModelSelectorItemChild {
    fn from(value: ModelSelectorShortcut) -> Self {
        Self::Shortcut(value)
    }
}

impl From<AnyElement> for ModelSelectorItemChild {
    fn from(value: AnyElement) -> Self {
        Self::Custom(value)
    }
}

pub struct ModelSelectorItem {
    label: Arc<str>,
    value: Arc<str>,
    disabled: bool,
    force_mount: bool,
    keywords: Vec<Arc<str>>,
    test_id: Option<Arc<str>>,
    on_select: Option<fret_ui::action::OnActivate>,
    children: Vec<ModelSelectorItemChild>,
}

impl std::fmt::Debug for ModelSelectorItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModelSelectorItem")
            .field("label", &self.label.as_ref())
            .field("value", &self.value.as_ref())
            .field("disabled", &self.disabled)
            .field("force_mount", &self.force_mount)
            .field("keywords_len", &self.keywords.len())
            .field("test_id", &self.test_id.as_deref())
            .field("has_on_select", &self.on_select.is_some())
            .field("children_len", &self.children.len())
            .finish()
    }
}

impl ModelSelectorItem {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        let label = label.into();
        Self {
            label: label.clone(),
            value: label,
            disabled: false,
            force_mount: false,
            keywords: Vec::new(),
            test_id: None,
            on_select: None,
            children: Vec::new(),
        }
    }

    pub fn value(mut self, value: impl Into<Arc<str>>) -> Self {
        self.value = value.into();
        self
    }

    pub fn keywords<I, S>(mut self, keywords: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<Arc<str>>,
    {
        self.keywords = keywords.into_iter().map(Into::into).collect();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn force_mount(mut self, force_mount: bool) -> Self {
        self.force_mount = force_mount;
        self
    }

    pub fn test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
        self
    }

    pub fn on_select_action(mut self, on_select: fret_ui::action::OnActivate) -> Self {
        self.on_select = Some(on_select);
        self
    }

    pub fn child<C>(mut self, child: C) -> Self
    where
        C: Into<ModelSelectorItemChild>,
    {
        self.children.push(child.into());
        self
    }

    pub fn children<I, C>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = C>,
        C: Into<ModelSelectorItemChild>,
    {
        self.children = children.into_iter().map(Into::into).collect();
        self
    }

    fn into_command_item<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> CommandItem {
        let mut item = CommandItem::new(self.label)
            .value(self.value)
            .keywords(self.keywords)
            .disabled(self.disabled)
            .force_mount(self.force_mount);

        if let Some(on_select) = self.on_select {
            item = item.on_select_action(on_select);
        }
        if !self.children.is_empty() {
            item = item.children(
                self.children
                    .into_iter()
                    .map(|child| child.into_element(cx))
                    .collect::<Vec<_>>(),
            );
        }
        if let Some(test_id) = self.test_id {
            item = item.test_id(test_id);
        }

        item
    }
}

pub struct ModelSelectorGroup {
    heading: Option<Arc<str>>,
    items: Vec<ModelSelectorItem>,
    force_mount: bool,
}

impl std::fmt::Debug for ModelSelectorGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModelSelectorGroup")
            .field("heading", &self.heading.as_deref())
            .field("items_len", &self.items.len())
            .field("force_mount", &self.force_mount)
            .finish()
    }
}

impl ModelSelectorGroup {
    pub fn new(items: impl IntoIterator<Item = ModelSelectorItem>) -> Self {
        Self {
            heading: None,
            items: items.into_iter().collect(),
            force_mount: false,
        }
    }

    pub fn heading(mut self, heading: impl Into<Arc<str>>) -> Self {
        self.heading = Some(heading.into());
        self
    }

    pub fn force_mount(mut self, force_mount: bool) -> Self {
        self.force_mount = force_mount;
        self
    }

    pub fn item(mut self, item: ModelSelectorItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = ModelSelectorItem>) -> Self {
        self.items.extend(items);
        self
    }

    fn into_command_group<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> CommandGroup {
        let mut group = CommandGroup::new(
            self.items
                .into_iter()
                .map(|item| item.into_command_item(cx)),
        );

        if let Some(heading) = self.heading {
            group = group.heading(heading);
        }
        if self.force_mount {
            group = group.force_mount(true);
        }

        group
    }
}

pub enum ModelSelectorListEntry {
    Item(ModelSelectorItem),
    Group(ModelSelectorGroup),
    Shared(CommandEntry),
}

impl std::fmt::Debug for ModelSelectorListEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Item(item) => f.debug_tuple("Item").field(item).finish(),
            Self::Group(group) => f.debug_tuple("Group").field(group).finish(),
            Self::Shared(_) => f.write_str("Shared(..)"),
        }
    }
}

impl ModelSelectorListEntry {
    fn into_command_entry<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> CommandEntry {
        match self {
            Self::Item(value) => value.into_command_item(cx).into(),
            Self::Group(value) => value.into_command_group(cx).into(),
            Self::Shared(value) => value,
        }
    }
}

impl From<ModelSelectorItem> for ModelSelectorListEntry {
    fn from(value: ModelSelectorItem) -> Self {
        Self::Item(value)
    }
}

impl From<ModelSelectorGroup> for ModelSelectorListEntry {
    fn from(value: ModelSelectorGroup) -> Self {
        Self::Group(value)
    }
}

impl From<CommandItem> for ModelSelectorListEntry {
    fn from(value: CommandItem) -> Self {
        Self::Shared(value.into())
    }
}

impl From<CommandGroup> for ModelSelectorListEntry {
    fn from(value: CommandGroup) -> Self {
        Self::Shared(value.into())
    }
}

impl From<CommandSeparator> for ModelSelectorListEntry {
    fn from(value: CommandSeparator) -> Self {
        Self::Shared(value.into())
    }
}

impl From<CommandEntry> for ModelSelectorListEntry {
    fn from(value: CommandEntry) -> Self {
        Self::Shared(value)
    }
}

pub struct ModelSelectorList {
    entries: Vec<ModelSelectorListEntry>,
    disabled: bool,
    empty_text: Arc<str>,
    query: Option<Model<String>>,
    highlight_query: Option<Model<String>>,
    scroll: LayoutRefinement,
}

impl std::fmt::Debug for ModelSelectorList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModelSelectorList")
            .field("entries_len", &self.entries.len())
            .field("disabled", &self.disabled)
            .field("empty_text", &self.empty_text.as_ref())
            .field("query", &self.query.as_ref().map(|model| model.id()))
            .field(
                "highlight_query",
                &self.highlight_query.as_ref().map(|model| model.id()),
            )
            .field("scroll", &self.scroll)
            .finish()
    }
}

impl ModelSelectorList {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            disabled: false,
            empty_text: Arc::from("No results."),
            query: None,
            highlight_query: None,
            scroll: LayoutRefinement::default()
                .max_h(Px(300.0))
                .w_full()
                .min_w_0(),
        }
    }

    pub fn new_entries<I, E>(entries: I) -> Self
    where
        I: IntoIterator<Item = E>,
        E: Into<ModelSelectorListEntry>,
    {
        Self {
            entries: entries.into_iter().map(Into::into).collect(),
            ..Self::new()
        }
    }

    pub fn entries<I, E>(mut self, entries: I) -> Self
    where
        I: IntoIterator<Item = E>,
        E: Into<ModelSelectorListEntry>,
    {
        self.entries = entries.into_iter().map(Into::into).collect();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn empty_text(mut self, text: impl Into<Arc<str>>) -> Self {
        self.empty_text = text.into();
        self
    }

    pub fn query_model(mut self, model: Model<String>) -> Self {
        self.query = Some(model);
        self
    }

    pub fn highlight_query_model(mut self, model: Model<String>) -> Self {
        self.highlight_query = Some(model);
        self
    }

    pub fn refine_scroll_layout(mut self, layout: LayoutRefinement) -> Self {
        self.scroll = self.scroll.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let entries = self
            .entries
            .into_iter()
            .map(|entry| entry.into_command_entry(cx))
            .collect::<Vec<_>>();

        let mut list = CommandList::new_entries(entries)
            .disabled(self.disabled)
            .empty_text(self.empty_text)
            .refine_scroll_layout(self.scroll);

        if let Some(query) = self.query {
            list = list.query_model(query);
        }
        if let Some(highlight_query) = self.highlight_query {
            list = list.highlight_query_model(highlight_query);
        }

        list.into_element(cx)
    }
}

pub type ModelSelectorEmpty = CommandEmpty;
pub type ModelSelectorShortcut = CommandShortcut;
pub type ModelSelectorSeparator = CommandSeparator;

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Rect, Size};
    use fret_ui::element::{ElementKind, TextProps};

    fn find_text_by_content<'a>(element: &'a AnyElement, needle: &str) -> Option<&'a TextProps> {
        match &element.kind {
            ElementKind::Text(props) if props.text.as_ref() == needle => Some(props),
            _ => element
                .children
                .iter()
                .find_map(|child| find_text_by_content(child, needle)),
        }
    }

    fn has_semantics_label(element: &AnyElement, needle: &str) -> bool {
        element
            .semantics_decoration
            .as_ref()
            .and_then(|decoration| decoration.label.as_ref())
            .is_some_and(|label| label.as_ref() == needle)
            || element
                .children
                .iter()
                .any(|child| has_semantics_label(child, needle))
    }

    #[test]
    fn model_selector_compound_children_surface_builds() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let query = app.models_mut().insert(String::new());

        let built = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(400.0), Px(240.0)),
            ),
            "test",
            |cx| {
                ModelSelector::new()
                    .children([
                        ModelSelectorChild::Trigger(ModelSelectorTrigger::new(cx.text("Select"))),
                        ModelSelectorChild::Content(
                            ModelSelectorContent::new(Vec::<AnyElement>::new())
                                .input(ModelSelectorInput::new(query.clone()))
                                .list(ModelSelectorList::new()),
                        ),
                    ])
                    .into_element(cx)
            },
        );

        assert!(
            find_text_by_content(&built, "Select").is_some(),
            "expected compound ModelSelector children surface to render the trigger child"
        );
    }

    #[test]
    fn model_selector_list_renders_typed_group_and_item_rows() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let query = app.models_mut().insert(String::new());

        let built = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(400.0), Px(240.0)),
            ),
            "test",
            |cx| {
                ModelSelectorList::new_entries([ModelSelectorGroup::new([{
                    ModelSelectorItem::new("GPT-4o")
                        .value("gpt-4o")
                        .keywords(["openai", "azure"])
                        .child(ModelSelectorLogo::new("openai"))
                        .child(ModelSelectorName::new("GPT-4o"))
                        .child(ModelSelectorLogoGroup::new([
                            ModelSelectorLogo::new("openai"),
                            ModelSelectorLogo::new("azure"),
                        ]))
                }])
                .heading("OpenAI")])
                .query_model(query)
                .into_element(cx)
            },
        );

        assert!(
            find_text_by_content(&built, "GPT-4o").is_some(),
            "expected typed ModelSelectorItem rows to render inside the list wrapper"
        );
        assert!(
            find_text_by_content(&built, "OpenAI").is_some(),
            "expected typed ModelSelectorGroup headings to render inside the list wrapper"
        );
    }

    #[test]
    fn model_selector_logo_group_accepts_typed_logo_children() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let built = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(160.0), Px(48.0))),
            "test",
            |cx| {
                ModelSelectorLogoGroup::new([
                    ModelSelectorLogo::new("openai"),
                    ModelSelectorLogo::new("azure"),
                ])
                .into_element(cx)
            },
        );

        assert!(
            has_semantics_label(&built, "openai logo"),
            "expected typed logo children to keep provider semantics"
        );
        assert!(
            has_semantics_label(&built, "azure logo"),
            "expected typed logo children to keep provider semantics"
        );
    }
}
