//! AI Elements-aligned `MicSelector` surfaces (UI-only).
//!
//! Upstream reference: `repo-ref/ai-elements/packages/elements/src/mic-selector.tsx`.
//!
//! Notes:
//! - Upstream enumerates `MediaDeviceInfo` via `navigator.mediaDevices` and handles permissions.
//! - In Fret, device enumeration and permission prompts are **app-owned**. This module renders the
//!   selector chrome and emits intent seams (`on_value_change`).

use std::sync::Arc;

use fret_core::{Color, FontWeight, Px, TextAlign, TextWrap};
use fret_runtime::Model;
use fret_ui::action::ActionCx;
use fret_ui::element::{AnyElement, SemanticsDecoration, TextProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::controllable_state;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::visually_hidden::visually_hidden;
use fret_ui_kit::typography;
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, Space, ui};
use fret_ui_shadcn::facade::{
    Button, ButtonSize, ButtonVariant, Command, CommandEntry, CommandGroup, CommandInput,
    CommandItem, CommandList, CommandSeparator, Popover, PopoverContent,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MicSelectorChildSlot {
    Trigger,
    Content,
}

pub type OnMicSelectorValueChange =
    Arc<dyn Fn(&mut dyn fret_ui::action::UiActionHost, ActionCx, Option<Arc<str>>) + 'static>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MicSelectorDevice {
    pub id: Arc<str>,
    pub label: Arc<str>,
}

impl MicSelectorDevice {
    pub fn new(id: impl Into<Arc<str>>, label: impl Into<Arc<str>>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
        }
    }
}

#[derive(Clone)]
pub struct MicSelectorController {
    pub devices: Arc<[MicSelectorDevice]>,
    pub value: Model<Option<Arc<str>>>,
    pub open: Model<bool>,
    pub query: Model<String>,
    pub on_value_change: Option<OnMicSelectorValueChange>,
}

impl std::fmt::Debug for MicSelectorController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MicSelectorController")
            .field("devices_len", &self.devices.len())
            .field("value", &self.value.id())
            .field("open", &self.open.id())
            .field("query", &self.query.id())
            .field("has_on_value_change", &self.on_value_change.is_some())
            .finish()
    }
}

#[derive(Debug, Clone, Copy)]
struct MicSelectorAnchorWidth(Option<Px>);

pub fn use_mic_selector_controller<H: UiHost>(
    cx: &ElementContext<'_, H>,
) -> Option<MicSelectorController> {
    cx.provided::<MicSelectorController>().cloned()
}

fn use_mic_selector_anchor_width<H: UiHost>(cx: &ElementContext<'_, H>) -> Option<Px> {
    cx.provided::<MicSelectorAnchorWidth>().and_then(|st| st.0)
}

#[track_caller]
fn query_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    cx.local_model(String::new)
}

fn muted_fg(theme: &Theme) -> Color {
    theme
        .color_by_key("muted-foreground")
        .or_else(|| theme.color_by_key("muted_foreground"))
        .unwrap_or_else(|| theme.color_required("foreground"))
}

fn split_device_label(label: &str) -> Option<(&str, &str)> {
    // Matches `name (1234:abcd)` with the suffix at the end.
    let label = label.trim_end();
    let (prefix, suffix) = label.rsplit_once('(')?;
    let suffix = suffix.strip_suffix(')')?.trim();
    if suffix.len() != 9 {
        return None;
    }
    let (a, b) = suffix.split_once(':')?;
    if a.len() != 4 || b.len() != 4 {
        return None;
    }
    let is_hex4 = |s: &str| s.chars().all(|c| c.is_ascii_hexdigit());
    if !is_hex4(a) || !is_hex4(b) {
        return None;
    }
    Some((prefix.trim_end(), suffix))
}

/// AI Elements-aligned `MicSelector` root.
#[derive(Clone)]
pub struct MicSelector {
    devices: Arc<[MicSelectorDevice]>,
    value: Option<Model<Option<Arc<str>>>>,
    default_value: Option<Arc<str>>,
    open: Option<Model<bool>>,
    default_open: bool,
    on_value_change: Option<OnMicSelectorValueChange>,
}

impl std::fmt::Debug for MicSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MicSelector")
            .field("devices_len", &self.devices.len())
            .field("value", &self.value.as_ref().map(|m| m.id()))
            .field(
                "default_value_len",
                &self.default_value.as_ref().map(|s| s.len()),
            )
            .field("open", &self.open.as_ref().map(|m| m.id()))
            .field("default_open", &self.default_open)
            .field("has_on_value_change", &self.on_value_change.is_some())
            .finish()
    }
}

impl MicSelector {
    pub fn new(devices: impl IntoIterator<Item = MicSelectorDevice>) -> Self {
        Self::from_arc(devices.into_iter().collect::<Vec<_>>().into())
    }

    pub fn from_arc(devices: Arc<[MicSelectorDevice]>) -> Self {
        Self {
            devices,
            value: None,
            default_value: None,
            open: None,
            default_open: false,
            on_value_change: None,
        }
    }

    pub fn value_model(mut self, model: Model<Option<Arc<str>>>) -> Self {
        self.value = Some(model);
        self
    }

    pub fn default_value(mut self, value: impl Into<Arc<str>>) -> Self {
        self.default_value = Some(value.into());
        self
    }

    pub fn open_model(mut self, open: Model<bool>) -> Self {
        self.open = Some(open);
        self
    }

    pub fn default_open(mut self, open: bool) -> Self {
        self.default_open = open;
        self
    }

    pub fn on_value_change(mut self, cb: OnMicSelectorValueChange) -> Self {
        self.on_value_change = Some(cb);
        self
    }

    /// Docs-shaped compound children composition aligned with upstream `<MicSelector>...</MicSelector>`.
    pub fn children<I, C>(self, children: I) -> MicSelectorWithChildren
    where
        I: IntoIterator<Item = C>,
        C: Into<MicSelectorChild>,
    {
        MicSelectorWithChildren {
            root: self,
            children: children.into_iter().map(Into::into).collect(),
        }
    }

    pub fn trigger(self, trigger: MicSelectorTrigger) -> MicSelectorWithChildren {
        self.children([MicSelectorChild::Trigger(trigger)])
    }

    pub fn content(self, content: MicSelectorContent) -> MicSelectorWithChildren {
        self.children([MicSelectorChild::Content(content)])
    }

    /// Rust-friendly compound entrypoint for the upstream JSX children shape.
    ///
    /// This mirrors the existing AI Elements pattern used elsewhere in the repo: descendants are
    /// built under the nearest provider context, while callers still write an explicit trigger /
    /// content pair.
    pub fn into_element_with_children<H, F>(
        self,
        cx: &mut ElementContext<'_, H>,
        children: F,
    ) -> AnyElement
    where
        H: UiHost,
        F: Fn(&mut ElementContext<'_, H>, MicSelectorChildSlot) -> AnyElement + Clone + 'static,
    {
        let open = fret_ui_kit::primitives::popover::PopoverRoot::new()
            .open(self.open.clone())
            .default_open(self.default_open)
            .open_model(cx);

        let value = controllable_state::use_controllable_model(cx, self.value.clone(), || {
            self.default_value.clone()
        })
        .model();

        let query = query_model(cx);
        let controller = MicSelectorController {
            devices: self.devices.clone(),
            value: value.clone(),
            open: open.clone(),
            query: query.clone(),
            on_value_change: self.on_value_change.clone(),
        };

        let controller_for_trigger = controller.clone();
        let controller_for_content = controller;
        let children_for_trigger = children.clone();
        let children_for_content = children;

        Popover::from_open(open.clone()).into_element_with_anchor(
            cx,
            move |cx| {
                cx.provide(controller_for_trigger.clone(), |cx| {
                    cx.provide(MicSelectorAnchorWidth(None), |cx| {
                        children_for_trigger(cx, MicSelectorChildSlot::Trigger)
                    })
                })
            },
            move |cx, anchor_rect| {
                cx.provide(controller_for_content.clone(), |cx| {
                    cx.provide(MicSelectorAnchorWidth(Some(anchor_rect.size.width)), |cx| {
                        children_for_content(cx, MicSelectorChildSlot::Content)
                    })
                })
            },
        )
    }

    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    ) -> AnyElement {
        let open = fret_ui_kit::primitives::popover::PopoverRoot::new()
            .open(self.open.clone())
            .default_open(self.default_open)
            .open_model(cx);

        let value = controllable_state::use_controllable_model(cx, self.value.clone(), || {
            self.default_value.clone()
        })
        .model();

        let query = query_model(cx);
        let controller = MicSelectorController {
            devices: self.devices.clone(),
            value: value.clone(),
            open: open.clone(),
            query: query.clone(),
            on_value_change: self.on_value_change.clone(),
        };

        let controller_for_trigger = controller.clone();
        let controller_for_content = controller.clone();

        Popover::from_open(open.clone()).into_element_with_anchor(
            cx,
            move |cx| {
                cx.provide(controller_for_trigger.clone(), |cx| {
                    cx.provide(MicSelectorAnchorWidth(None), trigger)
                })
            },
            move |cx, anchor_rect| {
                cx.provide(controller_for_content.clone(), |cx| {
                    cx.provide(
                        MicSelectorAnchorWidth(Some(anchor_rect.size.width)),
                        content,
                    )
                })
            },
        )
    }
}

pub enum MicSelectorChild {
    Trigger(MicSelectorTrigger),
    Content(MicSelectorContent),
}

impl std::fmt::Debug for MicSelectorChild {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Trigger(_) => f.write_str("MicSelectorChild::Trigger(..)"),
            Self::Content(_) => f.write_str("MicSelectorChild::Content(..)"),
        }
    }
}

impl From<MicSelectorTrigger> for MicSelectorChild {
    fn from(value: MicSelectorTrigger) -> Self {
        Self::Trigger(value)
    }
}

impl From<MicSelectorContent> for MicSelectorChild {
    fn from(value: MicSelectorContent) -> Self {
        Self::Content(value)
    }
}

pub struct MicSelectorWithChildren {
    root: MicSelector,
    children: Vec<MicSelectorChild>,
}

impl std::fmt::Debug for MicSelectorWithChildren {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MicSelectorWithChildren")
            .field("root", &self.root)
            .field("children_len", &self.children.len())
            .finish()
    }
}

impl MicSelectorWithChildren {
    pub fn children<I, C>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = C>,
        C: Into<MicSelectorChild>,
    {
        self.children.extend(children.into_iter().map(Into::into));
        self
    }

    pub fn trigger(self, trigger: MicSelectorTrigger) -> Self {
        self.children([MicSelectorChild::Trigger(trigger)])
    }

    pub fn content(self, content: MicSelectorContent) -> Self {
        self.children([MicSelectorChild::Content(content)])
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let mut trigger = None;
        let mut content = None;

        for child in self.children {
            match child {
                MicSelectorChild::Trigger(value) => {
                    if trigger.is_some() {
                        debug_assert!(false, "MicSelector expects a single MicSelectorTrigger");
                    }
                    trigger = Some(value);
                }
                MicSelectorChild::Content(value) => {
                    if content.is_some() {
                        debug_assert!(false, "MicSelector expects a single MicSelectorContent");
                    }
                    content = Some(value);
                }
            }
        }

        let Some(trigger) = trigger else {
            debug_assert!(false, "MicSelector requires a MicSelectorTrigger");
            return visually_hidden(cx, |_| Vec::<AnyElement>::new());
        };

        let content = content.unwrap_or_else(|| MicSelectorContent::new(Vec::<AnyElement>::new()));

        self.root.into_element(
            cx,
            move |cx| trigger.into_element(cx),
            move |cx| content.into_element(cx),
        )
    }
}

/// AI Elements-aligned `MicSelectorTrigger`.
pub struct MicSelectorTrigger {
    children: Vec<AnyElement>,
    value: Option<MicSelectorValue>,
    disabled: bool,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for MicSelectorTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MicSelectorTrigger")
            .field("children_len", &self.children.len())
            .field("has_value", &self.value.is_some())
            .field("disabled", &self.disabled)
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .finish()
    }
}

impl MicSelectorTrigger {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            value: None,
            disabled: false,
            test_id: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
        }
    }

    pub fn value(mut self, value: MicSelectorValue) -> Self {
        self.value = Some(value);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
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
        let Some(controller) = use_mic_selector_controller(cx) else {
            return visually_hidden(cx, |_| Vec::<AnyElement>::new());
        };
        let query = controller.query.clone();

        let icon = decl_icon::icon(
            cx,
            fret_icons::IconId::new_static("lucide.chevrons-up-down"),
        );
        let mut children = self.children;
        if let Some(value) = self.value {
            children.push(value.into_element(cx));
        }
        children.push(icon);

        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _| {
            let _ = host.models_mut().update(&query, |v| v.clear());
            host.notify(action_cx);
            host.request_redraw(action_cx.window);
        });

        let mut btn = Button::new("Microphone")
            .variant(ButtonVariant::Outline)
            .size(ButtonSize::Default)
            .on_activate(on_activate)
            .toggle_model(controller.open)
            .children(children)
            .disabled(self.disabled)
            .refine_layout(self.layout);

        if let Some(test_id) = self.test_id {
            btn = btn.test_id(test_id);
        }

        btn.into_element(cx)
    }
}

/// AI Elements-aligned `MicSelectorContent` (PopoverContent + Command container).
pub struct MicSelectorContent {
    children: Vec<AnyElement>,
    input: Option<MicSelectorInput>,
    list: Option<MicSelectorList>,
    popover_chrome: ChromeRefinement,
    popover_layout: LayoutRefinement,
    command_chrome: ChromeRefinement,
    command_layout: LayoutRefinement,
    test_id_root: Option<Arc<str>>,
}

impl std::fmt::Debug for MicSelectorContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MicSelectorContent")
            .field("children_len", &self.children.len())
            .field("has_input", &self.input.is_some())
            .field("has_list", &self.list.is_some())
            .field("popover_layout", &self.popover_layout)
            .field("test_id_root", &self.test_id_root.as_deref())
            .finish()
    }
}

impl MicSelectorContent {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            input: None,
            list: None,
            popover_chrome: ChromeRefinement::default().p(Space::N0),
            popover_layout: LayoutRefinement::default().min_w_0(),
            command_chrome: ChromeRefinement::default(),
            command_layout: LayoutRefinement::default().w_full().min_w_0(),
            test_id_root: None,
        }
    }

    pub fn input(mut self, input: MicSelectorInput) -> Self {
        self.input = Some(input);
        self
    }

    pub fn list<L>(mut self, list: L) -> Self
    where
        L: Into<MicSelectorList>,
    {
        self.list = Some(list.into());
        self
    }

    pub fn test_id_root(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_root = Some(id.into());
        self
    }

    pub fn refine_popover_style(mut self, chrome: ChromeRefinement) -> Self {
        self.popover_chrome = self.popover_chrome.merge(chrome);
        self
    }

    pub fn refine_popover_layout(mut self, layout: LayoutRefinement) -> Self {
        self.popover_layout = self.popover_layout.merge(layout);
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
        let mut children = Vec::with_capacity(
            self.children.len()
                + usize::from(self.input.is_some())
                + usize::from(self.list.is_some()),
        );
        if let Some(input) = self.input {
            children.push(input.into_element(cx));
        }
        if let Some(list) = self.list {
            children.push(list.into_element(cx));
        }
        children.extend(self.children);

        let command = Command::new(children)
            .refine_style(self.command_chrome)
            .refine_layout(self.command_layout)
            .into_element(cx);

        let mut popover_layout = self.popover_layout;
        if let Some(anchor_width) = use_mic_selector_anchor_width(cx) {
            popover_layout =
                popover_layout.merge(LayoutRefinement::default().w_px(anchor_width).min_w_0());
        }

        let mut content = PopoverContent::new([command])
            .refine_style(self.popover_chrome)
            .refine_layout(popover_layout)
            .into_element(cx);

        if let Some(test_id) = self.test_id_root {
            content = content.attach_semantics(SemanticsDecoration::default().test_id(test_id));
        }

        content
    }
}

/// AI Elements-aligned `MicSelectorInput`.
#[derive(Clone)]
pub struct MicSelectorInput {
    placeholder: Arc<str>,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for MicSelectorInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MicSelectorInput")
            .field("placeholder", &self.placeholder.as_ref())
            .field("test_id", &self.test_id.as_deref())
            .finish()
    }
}

impl Default for MicSelectorInput {
    fn default() -> Self {
        Self::new()
    }
}

impl MicSelectorInput {
    pub fn new() -> Self {
        Self {
            placeholder: Arc::from("Search microphones..."),
            test_id: None,
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(controller) = use_mic_selector_controller(cx) else {
            return visually_hidden(cx, |_| Vec::<AnyElement>::new());
        };

        let mut input = CommandInput::new(controller.query)
            .placeholder(self.placeholder)
            .into_element(cx);

        if let Some(test_id) = self.test_id {
            input = input.attach_semantics(SemanticsDecoration::default().test_id(test_id));
        }

        input
    }
}

/// Renders the current selected microphone label (or placeholder text).
#[derive(Clone)]
pub struct MicSelectorValue {
    placeholder: Arc<str>,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for MicSelectorValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MicSelectorValue")
            .field("placeholder", &self.placeholder.as_ref())
            .field("test_id", &self.test_id.as_deref())
            .finish()
    }
}

impl Default for MicSelectorValue {
    fn default() -> Self {
        Self::new()
    }
}

impl MicSelectorValue {
    pub fn new() -> Self {
        Self {
            placeholder: Arc::from("Select microphone..."),
            test_id: None,
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(controller) = use_mic_selector_controller(cx) else {
            return visually_hidden(cx, |_| Vec::<AnyElement>::new());
        };

        let theme = Theme::global(&*cx.app).clone();

        let selected = cx
            .app
            .models()
            .read(&controller.value, |v| v.clone())
            .ok()
            .flatten();

        let mut out: AnyElement = if let Some(id) = selected {
            if let Some(device) = controller
                .devices
                .iter()
                .find(|d| d.id.as_ref() == id.as_ref())
            {
                MicSelectorLabel::new(device.clone()).into_element(cx)
            } else {
                cx.text_props(TextProps {
                    layout: Default::default(),
                    text: id,
                    style: Some(typography::preset_text_style_with_overrides(
                        &theme,
                        typography::TypographyPreset::control_ui(typography::UiTextSize::Sm),
                        Some(FontWeight::NORMAL),
                        None,
                    )),
                    color: None,
                    wrap: TextWrap::None,
                    overflow: fret_core::TextOverflow::Ellipsis,
                    align: TextAlign::Start,
                    ink_overflow: Default::default(),
                })
            }
        } else {
            cx.text_props(TextProps {
                layout: Default::default(),
                text: self.placeholder.clone(),
                style: Some(typography::preset_text_style_with_overrides(
                    &theme,
                    typography::TypographyPreset::control_ui(typography::UiTextSize::Sm),
                    Some(FontWeight::NORMAL),
                    None,
                )),
                color: Some(muted_fg(&theme)),
                wrap: TextWrap::None,
                overflow: fret_core::TextOverflow::Ellipsis,
                align: TextAlign::Start,
                ink_overflow: Default::default(),
            })
        };

        if let Some(test_id) = self.test_id {
            out = out.attach_semantics(SemanticsDecoration::default().test_id(test_id));
        }
        out
    }
}

/// Label renderer aligned with upstream `MicSelectorLabel`.
#[derive(Clone)]
pub struct MicSelectorLabel {
    device: MicSelectorDevice,
}

impl std::fmt::Debug for MicSelectorLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MicSelectorLabel")
            .field("id", &self.device.id.as_ref())
            .field("label_len", &self.device.label.len())
            .finish()
    }
}

impl MicSelectorLabel {
    pub fn new(device: MicSelectorDevice) -> Self {
        Self { device }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let label = self.device.label.as_ref();

        if let Some((name, device_id)) = split_device_label(label) {
            let name_el = cx.text_props(TextProps {
                layout: Default::default(),
                text: Arc::from(name),
                style: Some(typography::preset_text_style_with_overrides(
                    &theme,
                    typography::TypographyPreset::control_ui(typography::UiTextSize::Sm),
                    Some(FontWeight::NORMAL),
                    None,
                )),
                color: None,
                wrap: TextWrap::None,
                overflow: fret_core::TextOverflow::Ellipsis,
                align: TextAlign::Start,

                ink_overflow: fret_ui::element::TextInkOverflow::None,
            });
            let id_el = cx.text_props(TextProps {
                layout: Default::default(),
                text: Arc::from(format!(" ({device_id})")),
                style: Some(typography::preset_text_style_with_overrides(
                    &theme,
                    typography::TypographyPreset::control_ui(typography::UiTextSize::Sm),
                    Some(FontWeight::NORMAL),
                    None,
                )),
                color: Some(muted_fg(&theme)),
                wrap: TextWrap::None,
                overflow: fret_core::TextOverflow::Clip,
                align: TextAlign::Start,

                ink_overflow: fret_ui::element::TextInkOverflow::None,
            });

            return ui::h_row(|_cx| vec![name_el, id_el])
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .items_center()
                .gap(Space::N1)
                .into_element(cx);
        }

        cx.text_props(TextProps {
            layout: Default::default(),
            text: self.device.label,
            style: Some(typography::preset_text_style_with_overrides(
                &theme,
                typography::TypographyPreset::control_ui(typography::UiTextSize::Sm),
                Some(FontWeight::NORMAL),
                None,
            )),
            color: None,
            wrap: TextWrap::None,
            overflow: fret_core::TextOverflow::Ellipsis,
            align: TextAlign::Start,

            ink_overflow: fret_ui::element::TextInkOverflow::None,
        })
    }
}

fn mic_selector_select_action(
    controller: &MicSelectorController,
    value: Arc<str>,
) -> fret_ui::action::OnActivate {
    let value_model = controller.value.clone();
    let open_model = controller.open.clone();
    let query_model = controller.query.clone();
    let on_value_change = controller.on_value_change.clone();

    Arc::new(move |host, action_cx, _| {
        let _ = host
            .models_mut()
            .update(&value_model, |v| *v = Some(value.clone()));
        let _ = host.models_mut().update(&query_model, |v| v.clear());
        let _ = host.models_mut().update(&open_model, |v| *v = false);
        if let Some(cb) = on_value_change.clone() {
            cb(host, action_cx, Some(value.clone()));
        }
        host.notify(action_cx);
        host.request_redraw(action_cx.window);
    })
}

fn default_mic_selector_item_test_id(
    prefix: Option<&Arc<str>>,
    value: &Arc<str>,
) -> Option<Arc<str>> {
    prefix.map(|prefix| Arc::from(format!("{prefix}-{}", value.as_ref().replace(' ', "-"))))
}

#[derive(Clone)]
pub struct MicSelectorEmpty {
    text: Arc<str>,
}

impl std::fmt::Debug for MicSelectorEmpty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MicSelectorEmpty")
            .field("text", &self.text.as_ref())
            .finish()
    }
}

impl Default for MicSelectorEmpty {
    fn default() -> Self {
        Self::new()
    }
}

impl MicSelectorEmpty {
    pub fn new() -> Self {
        Self {
            text: Arc::from("No microphone found."),
        }
    }

    pub fn text(mut self, text: impl Into<Arc<str>>) -> Self {
        self.text = text.into();
        self
    }
}

pub enum MicSelectorItemChild {
    Label(MicSelectorLabel),
    Custom(AnyElement),
}

impl std::fmt::Debug for MicSelectorItemChild {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Label(value) => f.debug_tuple("Label").field(value).finish(),
            Self::Custom(_) => f.write_str("Custom(..)"),
        }
    }
}

impl MicSelectorItemChild {
    fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        match self {
            Self::Label(value) => value.into_element(cx),
            Self::Custom(value) => value,
        }
    }
}

impl From<MicSelectorLabel> for MicSelectorItemChild {
    fn from(value: MicSelectorLabel) -> Self {
        Self::Label(value)
    }
}

impl From<AnyElement> for MicSelectorItemChild {
    fn from(value: AnyElement) -> Self {
        Self::Custom(value)
    }
}

pub struct MicSelectorItem {
    label: Arc<str>,
    value: Arc<str>,
    disabled: bool,
    force_mount: bool,
    keywords: Vec<Arc<str>>,
    test_id: Option<Arc<str>>,
    on_select: Option<fret_ui::action::OnActivate>,
    children: Vec<MicSelectorItemChild>,
}

impl std::fmt::Debug for MicSelectorItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MicSelectorItem")
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

impl MicSelectorItem {
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
        C: Into<MicSelectorItemChild>,
    {
        self.children.push(child.into());
        self
    }

    pub fn children<I, C>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = C>,
        C: Into<MicSelectorItemChild>,
    {
        self.children = children.into_iter().map(Into::into).collect();
        self
    }

    fn into_command_item<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        controller: &MicSelectorController,
        test_id_prefix: Option<&Arc<str>>,
    ) -> CommandItem {
        let value = self.value.clone();
        let on_select = self
            .on_select
            .unwrap_or_else(|| mic_selector_select_action(controller, value.clone()));

        let mut item = CommandItem::new(self.label)
            .value(value.clone())
            .keywords(self.keywords)
            .disabled(self.disabled)
            .force_mount(self.force_mount)
            .on_select_action(on_select);

        if !self.children.is_empty() {
            item = item.children(
                self.children
                    .into_iter()
                    .map(|child| child.into_element(cx))
                    .collect::<Vec<_>>(),
            );
        }

        if let Some(test_id) = self
            .test_id
            .or_else(|| default_mic_selector_item_test_id(test_id_prefix, &value))
        {
            item = item.test_id(test_id);
        }

        item
    }
}

enum MicSelectorListMode {
    Auto,
    Entries(Vec<MicSelectorListEntry>),
    Builder(Box<dyn FnOnce(Arc<[MicSelectorDevice]>) -> Vec<MicSelectorListEntry> + 'static>),
}

impl std::fmt::Debug for MicSelectorListMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auto => f.write_str("Auto"),
            Self::Entries(entries) => f.debug_tuple("Entries").field(entries).finish(),
            Self::Builder(_) => f.write_str("Builder(..)"),
        }
    }
}

pub enum MicSelectorListEntry {
    Item(MicSelectorItem),
    Shared(CommandEntry),
}

impl std::fmt::Debug for MicSelectorListEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Item(item) => f.debug_tuple("Item").field(item).finish(),
            Self::Shared(_) => f.write_str("Shared(..)"),
        }
    }
}

impl From<MicSelectorItem> for MicSelectorListEntry {
    fn from(value: MicSelectorItem) -> Self {
        Self::Item(value)
    }
}

impl From<CommandItem> for MicSelectorListEntry {
    fn from(value: CommandItem) -> Self {
        Self::Shared(value.into())
    }
}

impl From<CommandGroup> for MicSelectorListEntry {
    fn from(value: CommandGroup) -> Self {
        Self::Shared(value.into())
    }
}

impl From<CommandSeparator> for MicSelectorListEntry {
    fn from(value: CommandSeparator) -> Self {
        Self::Shared(value.into())
    }
}

impl From<CommandEntry> for MicSelectorListEntry {
    fn from(value: CommandEntry) -> Self {
        Self::Shared(value)
    }
}

/// List renderer aligned with upstream `MicSelectorList` outcomes.
pub struct MicSelectorList {
    empty_text: Arc<str>,
    test_id_prefix: Option<Arc<str>>,
    mode: MicSelectorListMode,
    scroll_layout: LayoutRefinement,
}

impl std::fmt::Debug for MicSelectorList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MicSelectorList")
            .field("empty_text", &self.empty_text.as_ref())
            .field("test_id_prefix", &self.test_id_prefix.as_deref())
            .field("scroll_layout", &self.scroll_layout)
            .finish()
    }
}

impl Default for MicSelectorList {
    fn default() -> Self {
        Self::new()
    }
}

impl MicSelectorList {
    pub fn new() -> Self {
        Self {
            empty_text: Arc::from("No microphone found."),
            test_id_prefix: None,
            mode: MicSelectorListMode::Auto,
            scroll_layout: LayoutRefinement::default(),
        }
    }

    pub fn new_entries<I, E>(entries: I) -> Self
    where
        I: IntoIterator<Item = E>,
        E: Into<MicSelectorListEntry>,
    {
        Self {
            mode: MicSelectorListMode::Entries(entries.into_iter().map(Into::into).collect()),
            ..Self::new()
        }
    }

    pub fn empty(mut self, empty: MicSelectorEmpty) -> Self {
        self.empty_text = empty.text;
        self
    }

    pub fn empty_text(mut self, text: impl Into<Arc<str>>) -> Self {
        self.empty_text = text.into();
        self
    }

    pub fn test_id_prefix(mut self, prefix: impl Into<Arc<str>>) -> Self {
        self.test_id_prefix = Some(prefix.into());
        self
    }

    pub fn entries<I, E>(mut self, entries: I) -> Self
    where
        I: IntoIterator<Item = E>,
        E: Into<MicSelectorListEntry>,
    {
        self.mode = MicSelectorListMode::Entries(entries.into_iter().map(Into::into).collect());
        self
    }

    pub fn refine_scroll_layout(mut self, layout: LayoutRefinement) -> Self {
        self.scroll_layout = self.scroll_layout.merge(layout);
        self
    }

    /// Rust-friendly render-prop equivalent of upstream `<MicSelectorList>{(devices) => ...}</MicSelectorList>`.
    pub fn children<F>(self, children: F) -> MicSelectorListWithChildren<F> {
        MicSelectorListWithChildren {
            list: self,
            children,
        }
    }

    fn auto_entries(devices: &[MicSelectorDevice]) -> Vec<MicSelectorListEntry> {
        devices
            .iter()
            .cloned()
            .map(|device| {
                MicSelectorItem::new(device.label.clone())
                    .value(device.id.clone())
                    .child(MicSelectorLabel::new(device))
                    .into()
            })
            .collect()
    }

    fn resolve_entries<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        controller: &MicSelectorController,
        test_id_prefix: Option<&Arc<str>>,
        entries: Vec<MicSelectorListEntry>,
    ) -> Vec<CommandEntry> {
        entries
            .into_iter()
            .map(|entry| match entry {
                MicSelectorListEntry::Item(item) => item
                    .into_command_item(cx, controller, test_id_prefix)
                    .into(),
                MicSelectorListEntry::Shared(entry) => entry,
            })
            .collect()
    }

    fn into_element_with_entries<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        empty_text: Arc<str>,
        scroll_layout: LayoutRefinement,
        entries: Vec<CommandEntry>,
    ) -> AnyElement {
        let Some(controller) = use_mic_selector_controller(cx) else {
            return visually_hidden(cx, |_| Vec::<AnyElement>::new());
        };
        CommandList::new_entries(entries)
            .empty_text(empty_text)
            .query_model(controller.query.clone())
            .highlight_query_model(controller.query)
            .refine_scroll_layout(scroll_layout)
            .into_element(cx)
    }

    /// Rust-friendly equivalent of upstream `MicSelectorList(children(data))`.
    ///
    /// The closure receives the current device slice and returns explicit selector entries,
    /// preserving the docs-style compound composition while keeping device enumeration app-owned.
    pub fn into_element_with_children<H, F, I>(
        self,
        cx: &mut ElementContext<'_, H>,
        children: F,
    ) -> AnyElement
    where
        H: UiHost,
        F: FnOnce(Arc<[MicSelectorDevice]>) -> I,
        I: IntoIterator,
        I::Item: Into<MicSelectorListEntry>,
    {
        let Some(controller) = use_mic_selector_controller(cx) else {
            return visually_hidden(cx, |_| Vec::<AnyElement>::new());
        };

        let Self {
            empty_text,
            test_id_prefix,
            scroll_layout,
            ..
        } = self;

        let entries = Self::resolve_entries(
            cx,
            &controller,
            test_id_prefix.as_ref(),
            children(controller.devices.clone())
                .into_iter()
                .map(Into::into)
                .collect(),
        );

        Self::into_element_with_entries(cx, empty_text, scroll_layout, entries)
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Self {
            empty_text,
            test_id_prefix,
            mode,
            scroll_layout,
        } = self;

        let Some(controller) = use_mic_selector_controller(cx) else {
            return visually_hidden(cx, |_| Vec::<AnyElement>::new());
        };

        let entries = match mode {
            MicSelectorListMode::Auto => Self::resolve_entries(
                cx,
                &controller,
                test_id_prefix.as_ref(),
                Self::auto_entries(controller.devices.as_ref()),
            ),
            MicSelectorListMode::Entries(entries) => {
                Self::resolve_entries(cx, &controller, test_id_prefix.as_ref(), entries)
            }
            MicSelectorListMode::Builder(children) => Self::resolve_entries(
                cx,
                &controller,
                test_id_prefix.as_ref(),
                children(controller.devices.clone()),
            ),
        };

        Self::into_element_with_entries(cx, empty_text, scroll_layout, entries)
    }
}

pub struct MicSelectorListWithChildren<F> {
    list: MicSelectorList,
    children: F,
}

impl<F> std::fmt::Debug for MicSelectorListWithChildren<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MicSelectorListWithChildren")
            .field("list", &self.list)
            .finish_non_exhaustive()
    }
}

impl<F> MicSelectorListWithChildren<F> {
    pub fn empty(mut self, empty: MicSelectorEmpty) -> Self {
        self.list = self.list.empty(empty);
        self
    }

    pub fn empty_text(mut self, text: impl Into<Arc<str>>) -> Self {
        self.list = self.list.empty_text(text);
        self
    }

    pub fn test_id_prefix(mut self, prefix: impl Into<Arc<str>>) -> Self {
        self.list = self.list.test_id_prefix(prefix);
        self
    }

    pub fn refine_scroll_layout(mut self, layout: LayoutRefinement) -> Self {
        self.list = self.list.refine_scroll_layout(layout);
        self
    }

    pub fn into_element<H, I>(self, cx: &mut ElementContext<'_, H>) -> AnyElement
    where
        H: UiHost,
        F: FnOnce(Arc<[MicSelectorDevice]>) -> I,
        I: IntoIterator,
        I::Item: Into<MicSelectorListEntry>,
    {
        self.list.into_element_with_children(cx, self.children)
    }
}

impl<F, I> From<MicSelectorListWithChildren<F>> for MicSelectorList
where
    F: FnOnce(Arc<[MicSelectorDevice]>) -> I + 'static,
    I: IntoIterator,
    I::Item: Into<MicSelectorListEntry>,
{
    fn from(value: MicSelectorListWithChildren<F>) -> Self {
        let MicSelectorListWithChildren { mut list, children } = value;
        list.mode = MicSelectorListMode::Builder(Box::new(move |devices| {
            children(devices).into_iter().map(Into::into).collect()
        }));
        list
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size};
    use fret_ui::element::{ElementKind, TextProps};

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        )
    }

    fn find_text_by_content<'a>(element: &'a AnyElement, needle: &str) -> Option<&'a TextProps> {
        match &element.kind {
            ElementKind::Text(props) if props.text.as_ref() == needle => Some(props),
            _ => element
                .children
                .iter()
                .find_map(|child| find_text_by_content(child, needle)),
        }
    }

    fn explicit_mic_selector_entries(devices: Arc<[MicSelectorDevice]>) -> Vec<MicSelectorItem> {
        devices
            .iter()
            .cloned()
            .map(|device| {
                MicSelectorItem::new(device.label.clone())
                    .value(device.id.clone())
                    .child(MicSelectorLabel::new(device))
            })
            .collect()
    }

    #[test]
    fn mic_selector_list_entries_mode_captures_entries() {
        let list = MicSelectorList::new_entries([MicSelectorItem::new("Default Microphone")]);
        match list.mode {
            MicSelectorListMode::Entries(entries) => assert_eq!(entries.len(), 1),
            MicSelectorListMode::Auto => panic!("expected entries mode"),
            MicSelectorListMode::Builder(_) => panic!("expected entries mode"),
        }
    }

    #[test]
    fn split_device_label_parses_trailing_hex_id() {
        assert_eq!(
            split_device_label("Default Microphone (1234:abcd)"),
            Some(("Default Microphone", "1234:abcd"))
        );
    }

    #[test]
    fn split_device_label_rejects_non_matching_suffix() {
        assert_eq!(split_device_label("Loopback"), None);
        assert_eq!(split_device_label("Mic (123:abcd)"), None);
        assert_eq!(split_device_label("Mic (1234:abcg)"), None);
    }

    #[test]
    fn mic_selector_surfaces_use_shared_sm_typography_preset() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let device = MicSelectorDevice::new("usb-mic", "Podcast Mic (1234:abcd)");
        let controller = MicSelectorController {
            devices: std::sync::Arc::from([device.clone()]),
            value: app.models_mut().insert(None),
            open: app.models_mut().insert(false),
            query: app.models_mut().insert(String::new()),
            on_value_change: None,
        };

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                cx.provide(controller.clone(), |cx| {
                    cx.provide(MicSelectorAnchorWidth(Some(Px(240.0))), |cx| {
                        ui::v_stack(|cx| {
                            vec![
                                MicSelectorValue::new()
                                    .placeholder("Pick a mic")
                                    .into_element(cx),
                                MicSelectorLabel::new(device.clone()).into_element(cx),
                            ]
                        })
                        .into_element(cx)
                    })
                })
            });

        let theme = Theme::global(&app).clone();
        let expected = Some(typography::preset_text_style_with_overrides(
            &theme,
            typography::TypographyPreset::control_ui(typography::UiTextSize::Sm),
            Some(FontWeight::NORMAL),
            None,
        ));

        let placeholder = find_text_by_content(&element, "Pick a mic").expect("placeholder text");
        assert_eq!(placeholder.style, expected.clone());

        let name = find_text_by_content(&element, "Podcast Mic").expect("device name text");
        assert_eq!(name.style, expected.clone());

        let device_id = find_text_by_content(&element, " (1234:abcd)").expect("device id text");
        assert_eq!(device_id.style, expected);
    }

    #[test]
    fn mic_selector_compound_children_surface_builds() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let devices = Arc::from([MicSelectorDevice::new(
            "default",
            "Default Microphone (1234:abcd)",
        )]);

        let _element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                MicSelector::from_arc(devices)
                    .default_open(true)
                    .children([
                        MicSelectorChild::Trigger(
                            MicSelectorTrigger::new([])
                                .value(MicSelectorValue::new())
                                .test_id("mic-selector-trigger"),
                        ),
                        MicSelectorChild::Content(
                            MicSelectorContent::new([])
                                .input(MicSelectorInput::new())
                                .list(
                                    MicSelectorList::new_entries(Vec::<MicSelectorItem>::new())
                                        .empty(MicSelectorEmpty::new()),
                                )
                                .test_id_root("mic-selector-content"),
                        ),
                    ])
                    .into_element(cx)
            });
    }

    #[test]
    fn mic_selector_list_children_builder_builds_typed_item_rows() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let device = MicSelectorDevice::new("usb-mic", "Podcast Mic (1234:abcd)");
        let controller = MicSelectorController {
            devices: Arc::from([device.clone()]),
            value: app.models_mut().insert(None),
            open: app.models_mut().insert(false),
            query: app.models_mut().insert(String::new()),
            on_value_change: None,
        };

        let built = fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
            cx.provide(controller.clone(), |cx| {
                MicSelectorList::new()
                    .children(explicit_mic_selector_entries)
                    .into_element(cx)
            })
        });

        assert!(
            find_text_by_content(&built, "Podcast Mic").is_some(),
            "expected render-prop list builder to render the explicit row"
        );
        assert!(
            find_text_by_content(&built, " (1234:abcd)").is_some(),
            "expected typed MicSelectorItem children to render MicSelectorLabel without prebuilt AnyElement rows"
        );
    }

    #[test]
    fn mic_selector_content_list_accepts_list_children_builder() {
        let content = MicSelectorContent::new(Vec::<AnyElement>::new()).list(
            MicSelectorList::new()
                .children(explicit_mic_selector_entries)
                .empty_text("No microphones found."),
        );

        let list = content
            .list
            .as_ref()
            .expect("expected MicSelectorContent::list(...) to store a list");
        assert!(
            matches!(list.mode, MicSelectorListMode::Builder(_)),
            "expected MicSelectorContent::list(...) to accept the render-prop list builder"
        );
    }
}
