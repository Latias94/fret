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
use fret_ui::element::{AnyElement, PressableA11y, PressableProps, SemanticsDecoration};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::declarative::visually_hidden::visually_hidden;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, Space};
use fret_ui_shadcn::{Command, CommandDialog, Dialog, DialogContent, DialogTitle};

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

    pub fn into_element<H: UiHost + 'static>(
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
#[derive(Clone)]
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

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let open = self.open;
        let child = self.child;
        let disabled = self.disabled;
        let test_id = self.test_id;

        let trigger = cx.pressable(
            PressableProps {
                enabled: !disabled,
                focusable: true,
                a11y: PressableA11y {
                    role: Some(SemanticsRole::Button),
                    test_id,
                    ..Default::default()
                },
                ..Default::default()
            },
            move |cx, _st| {
                if !disabled {
                    cx.pressable_set_bool(&open, true);
                }
                vec![child]
            },
        );
        trigger
    }
}

/// AI Elements-aligned `ModelSelectorContent`.
///
/// Upstream applies "no border + p-0" and renders the title as `sr-only`.
#[derive(Clone)]
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
            command_chrome: ChromeRefinement::default(),
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
            .unwrap_or_else(|| theme.color_required("background"));
        let fg = theme
            .color_by_key("foreground")
            .unwrap_or_else(|| theme.color_required("foreground"));
        let border = theme
            .color_by_key("border")
            .unwrap_or_else(|| theme.color_required("border"));

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
            let text = fret_ui_kit::ui::text(cx, initial)
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
            vec![stack::hstack(
                cx,
                stack::HStackProps::default()
                    .layout(LayoutRefinement::default().w_full().h_full())
                    .justify_center()
                    .items_center(),
                move |_cx| vec![text],
            )]
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
/// Upstream uses negative spacing (`-space-x-1`) to overlap the logos. Fret currently does not
/// expose negative gap at the stack level, so we render a tight row (apps can customize with
/// transforms if needed).
#[derive(Clone)]
pub struct ModelSelectorLogoGroup {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    gap: Space,
}

impl std::fmt::Debug for ModelSelectorLogoGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModelSelectorLogoGroup")
            .field("children_len", &self.children.len())
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .field("gap", &self.gap)
            .finish()
    }
}

impl ModelSelectorLogoGroup {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            test_id: None,
            layout: LayoutRefinement::default(),
            gap: Space::N1,
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let mut element = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(self.layout)
                .gap(self.gap)
                .items_center(),
            move |_cx| self.children,
        );

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
        let mut element = fret_ui_kit::ui::text(cx, self.text)
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

pub type ModelSelectorInput = fret_ui_shadcn::CommandInput;
pub type ModelSelectorList = fret_ui_shadcn::CommandList;
pub type ModelSelectorEmpty = fret_ui_shadcn::CommandEmpty;
pub type ModelSelectorGroup = fret_ui_shadcn::CommandGroup;
pub type ModelSelectorItem = fret_ui_shadcn::CommandItem;
pub type ModelSelectorShortcut = fret_ui_shadcn::CommandShortcut;
pub type ModelSelectorSeparator = fret_ui_shadcn::CommandSeparator;
