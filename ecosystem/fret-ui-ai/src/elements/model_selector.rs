//! AI Elements-aligned `ModelSelector` surfaces.
//!
//! Upstream reference: `repo-ref/ai-elements/packages/elements/src/model-selector.tsx`.
//!
//! Notes:
//! - The upstream implementation is a thin wrapper around shadcn/ui `Dialog` + `Command*`.
//! - Fret's cmdk-aligned surface is primarily `fret-ui-shadcn::CommandDialog` / `CommandPalette`,
//!   so this module focuses on matching *outcomes* and offering a familiar decomposition.
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
    Command, CommandDialog, CommandEmpty, CommandGroup, CommandInput, CommandItem, CommandList,
    CommandSeparator, CommandShortcut, Dialog, DialogContent, DialogTitle,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelSelectorChildSlot {
    Trigger,
    Content,
}

/// AI Elements-aligned `ModelSelector` root.
///
/// This is a light wrapper over shadcn `Dialog` that exposes a stable open model to both
/// `trigger` and `content` builders (so the trigger can set `open=true` and the content can close).
#[derive(Clone)]
pub struct ModelSelector {
    open: Option<Model<bool>>,
    default_open: bool,
}

impl std::fmt::Debug for ModelSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModelSelector")
            .field("open", &"<model>")
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
        let open = fret_ui_kit::primitives::dialog::DialogRoot::new()
            .open(self.open)
            .default_open(self.default_open)
            .open_model(cx);

        let open_for_trigger = open.clone();
        let open_for_content = open.clone();
        let children_for_trigger = children.clone();
        let children_for_content = children;

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
    }

    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>, Model<bool>) -> AnyElement,
        content: impl FnOnce(&mut ElementContext<'_, H>, Model<bool>) -> AnyElement,
    ) -> AnyElement {
        let open = fret_ui_kit::primitives::dialog::DialogRoot::new()
            .open(self.open)
            .default_open(self.default_open)
            .open_model(cx);

        let open_for_trigger = open.clone();
        Dialog::new(open.clone()).into_element(
            cx,
            move |cx| trigger(cx, open_for_trigger.clone()),
            move |cx| content(cx, open.clone()),
        )
    }
}

/// A shadcn `CommandDialog` wrapper aligned with AI Elements `ModelSelectorDialog`.
pub type ModelSelectorDialog = CommandDialog;

/// AI Elements-aligned `ModelSelectorTrigger`.
///
/// In upstream React this is `DialogTrigger`. In Fret we model it as a pressable wrapper that sets
/// the provided `open` model to `true`.
pub struct ModelSelectorTrigger {
    open: Model<bool>,
    child: AnyElement,
    disabled: bool,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for ModelSelectorTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModelSelectorTrigger")
            .field("open", &"<model>")
            .field("disabled", &self.disabled)
            .field("test_id", &self.test_id.as_deref())
            .finish()
    }
}

impl ModelSelectorTrigger {
    pub fn new(open: Model<bool>, child: AnyElement) -> Self {
        Self {
            open,
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let open = self.open;
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
                Arc::new(
                    move |host: &mut dyn fret_ui::action::UiActionHost,
                          acx: fret_ui::action::ActionCx,
                          _reason: fret_ui::action::ActivateReason| {
                        let _ = host.models_mut().update(&open, |v| *v = true);
                        host.request_redraw(acx.window);
                    },
                ),
            );
        }

        element
    }
}

/// AI Elements-aligned `ModelSelectorContent`.
///
/// Upstream applies "no border + p-0" and renders the title as `sr-only`.
pub struct ModelSelectorContent {
    title: Arc<str>,
    children: Vec<AnyElement>,
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
            test_id_root: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            // Upstream `Command` surface is borderless; the surrounding DialogContent applies an
            // outline-like stroke. Keep Fret's shadcn `Command` chrome but drop its border here.
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

        let command = Command::new(self.children)
            .refine_style(self.command_chrome)
            .refine_layout(self.command_layout)
            .into_element(cx);

        // AI Elements uses `outline` + `border-none` on the DialogContent. Fret's chrome surface
        // does not have a separate outline primitive, so we approximate with a 1px border and zero
        // padding.
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

/// AI Elements-aligned logo group.
///
/// Upstream uses negative spacing (`-space-x-1`) to overlap the logos.
pub struct ModelSelectorLogoGroup {
    children: Vec<AnyElement>,
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
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
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
        let children: Vec<AnyElement> = self
            .children
            .into_iter()
            .enumerate()
            .map(|(idx, child)| {
                if idx == 0 || overlap_x == Space::N0 {
                    return child;
                }

                ui::container(move |_cx| vec![child])
                    .layout(LayoutRefinement::default().ml_neg(overlap_x))
                    .into_element(cx)
            })
            .collect();

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

// ---- shadcn command taxonomy (names match AI Elements exports) ----

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
    pub fn new(model: fret_runtime::Model<String>) -> Self {
        Self {
            inner: CommandInput::new(model)
                .wrapper_height_auto()
                .input_height_auto()
                // Tailwind `py-3.5` ≈ 14px.
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

pub type ModelSelectorList = CommandList;
pub type ModelSelectorEmpty = CommandEmpty;
pub type ModelSelectorGroup = CommandGroup;
pub type ModelSelectorItem = CommandItem;
pub type ModelSelectorShortcut = CommandShortcut;
pub type ModelSelectorSeparator = CommandSeparator;
