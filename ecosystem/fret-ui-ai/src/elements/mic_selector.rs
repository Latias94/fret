//! AI Elements-aligned `MicSelector` surfaces (UI-only).
//!
//! Upstream reference: `repo-ref/ai-elements/packages/elements/src/mic-selector.tsx`.
//!
//! Notes:
//! - Upstream enumerates `MediaDeviceInfo` via `navigator.mediaDevices` and handles permissions.
//! - In Fret, device enumeration and permission prompts are **app-owned**. This module renders the
//!   selector chrome and emits intent seams (`on_value_change`).

use std::sync::Arc;

use fret_core::{Color, FontId, FontWeight, TextStyle, TextWrap};
use fret_runtime::Model;
use fret_ui::action::ActionCx;
use fret_ui::element::{AnyElement, SemanticsDecoration, TextProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::controllable_state;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::visually_hidden::visually_hidden;
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, Space};
use fret_ui_shadcn::{
    Button, ButtonSize, ButtonVariant, Command, CommandInput, CommandItem, CommandList, Popover,
    PopoverContent,
};

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

#[derive(Debug, Default, Clone)]
struct MicSelectorProviderState {
    controller: Option<MicSelectorController>,
}

pub fn use_mic_selector_controller<H: UiHost>(
    cx: &ElementContext<'_, H>,
) -> Option<MicSelectorController> {
    cx.inherited_state::<MicSelectorProviderState>()
        .and_then(|st| st.controller.clone())
}

#[derive(Default)]
struct MicSelectorState {
    query: Option<Model<String>>,
}

fn query_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    cx.with_state(MicSelectorState::default, |st| st.query.clone())
        .unwrap_or_else(|| {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(MicSelectorState::default, |st| {
                st.query = Some(model.clone())
            });
            model
        })
}

fn text_sm(theme: &Theme, weight: FontWeight) -> TextStyle {
    TextStyle {
        font: FontId::default(),
        size: theme.metric_required("component.text.sm_px"),
        weight,
        slant: Default::default(),
        line_height: Some(theme.metric_required("component.text.sm_line_height")),
        letter_spacing_em: None,
    }
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

    pub fn into_element<H: UiHost + 'static>(
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

        Popover::new(open.clone()).into_element(
            cx,
            move |cx| {
                cx.with_state(MicSelectorProviderState::default, |st| {
                    st.controller = Some(controller_for_trigger.clone());
                });
                trigger(cx)
            },
            move |cx| {
                cx.with_state(MicSelectorProviderState::default, |st| {
                    st.controller = Some(controller_for_content.clone());
                });
                content(cx)
            },
        )
    }
}

/// AI Elements-aligned `MicSelectorTrigger`.
#[derive(Clone)]
pub struct MicSelectorTrigger {
    children: Vec<AnyElement>,
    disabled: bool,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for MicSelectorTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MicSelectorTrigger")
            .field("children_len", &self.children.len())
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
            disabled: false,
            test_id: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
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

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(controller) = use_mic_selector_controller(cx) else {
            return visually_hidden(cx, |_| Vec::new());
        };

        let icon = decl_icon::icon(
            cx,
            fret_icons::IconId::new_static("lucide.chevrons-up-down"),
        );
        let mut children = self.children;
        children.push(icon);

        let mut btn = Button::new("Microphone")
            .variant(ButtonVariant::Outline)
            .size(ButtonSize::Default)
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
#[derive(Clone)]
pub struct MicSelectorContent {
    children: Vec<AnyElement>,
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
            .field("popover_layout", &self.popover_layout)
            .field("test_id_root", &self.test_id_root.as_deref())
            .finish()
    }
}

impl MicSelectorContent {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            popover_chrome: ChromeRefinement::default().p(Space::N0),
            popover_layout: LayoutRefinement::default().min_w_0(),
            command_chrome: ChromeRefinement::default(),
            command_layout: LayoutRefinement::default().w_full().min_w_0(),
            test_id_root: None,
        }
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

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let command = Command::new(self.children)
            .refine_style(self.command_chrome)
            .refine_layout(self.command_layout)
            .into_element(cx);

        let mut content = PopoverContent::new([command])
            .refine_style(self.popover_chrome)
            .refine_layout(self.popover_layout)
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
            return visually_hidden(cx, |_| Vec::new());
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
            return visually_hidden(cx, |_| Vec::new());
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
                    style: Some(text_sm(&theme, FontWeight::NORMAL)),
                    color: None,
                    wrap: TextWrap::None,
                    overflow: fret_core::TextOverflow::Ellipsis,
                })
            }
        } else {
            cx.text_props(TextProps {
                layout: Default::default(),
                text: self.placeholder.clone(),
                style: Some(text_sm(&theme, FontWeight::NORMAL)),
                color: Some(muted_fg(&theme)),
                wrap: TextWrap::None,
                overflow: fret_core::TextOverflow::Ellipsis,
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
                style: Some(text_sm(&theme, FontWeight::NORMAL)),
                color: None,
                wrap: TextWrap::None,
                overflow: fret_core::TextOverflow::Ellipsis,
            });
            let id_el = cx.text_props(TextProps {
                layout: Default::default(),
                text: Arc::from(format!(" ({device_id})")),
                style: Some(text_sm(&theme, FontWeight::NORMAL)),
                color: Some(muted_fg(&theme)),
                wrap: TextWrap::None,
                overflow: fret_core::TextOverflow::Clip,
            });

            return fret_ui_kit::declarative::stack::hstack(
                cx,
                fret_ui_kit::declarative::stack::HStackProps::default()
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .items_center()
                    .gap_x(Space::N1),
                |_cx| vec![name_el, id_el],
            );
        }

        cx.text_props(TextProps {
            layout: Default::default(),
            text: self.device.label,
            style: Some(text_sm(&theme, FontWeight::NORMAL)),
            color: None,
            wrap: TextWrap::None,
            overflow: fret_core::TextOverflow::Ellipsis,
        })
    }
}

/// Convenience list renderer: filters devices by the current query and commits selection on activate.
#[derive(Clone)]
pub struct MicSelectorList {
    empty_text: Arc<str>,
    test_id_prefix: Option<Arc<str>>,
}

impl std::fmt::Debug for MicSelectorList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MicSelectorList")
            .field("empty_text", &self.empty_text.as_ref())
            .field("test_id_prefix", &self.test_id_prefix.as_deref())
            .finish()
    }
}

impl MicSelectorList {
    pub fn new() -> Self {
        Self {
            empty_text: Arc::from("No microphone found."),
            test_id_prefix: None,
        }
    }

    pub fn empty_text(mut self, text: impl Into<Arc<str>>) -> Self {
        self.empty_text = text.into();
        self
    }

    pub fn test_id_prefix(mut self, prefix: impl Into<Arc<str>>) -> Self {
        self.test_id_prefix = Some(prefix.into());
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(controller) = use_mic_selector_controller(cx) else {
            return visually_hidden(cx, |_| Vec::new());
        };

        let query = cx
            .app
            .models()
            .read(&controller.query, |s| s.trim().to_ascii_lowercase())
            .ok()
            .unwrap_or_default();

        let mut items: Vec<CommandItem> = Vec::new();
        for device in controller.devices.iter() {
            if !query.is_empty() && !device.label.to_ascii_lowercase().contains(&query) {
                continue;
            }

            let id = device.id.clone();
            let value_model = controller.value.clone();
            let open_model = controller.open.clone();
            let on_value_change = controller.on_value_change.clone();
            let device_clone = device.clone();

            let on_select: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _| {
                let _ = host
                    .models_mut()
                    .update(&value_model, |v| *v = Some(id.clone()));
                let _ = host.models_mut().update(&open_model, |v| *v = false);
                if let Some(cb) = on_value_change.clone() {
                    cb(host, action_cx, Some(id.clone()));
                }
                host.notify(action_cx);
                host.request_redraw(action_cx.window);
            });

            let mut item = CommandItem::new(device.label.clone())
                .value(device.id.clone())
                .on_select_action(on_select)
                .children([MicSelectorLabel::new(device_clone).into_element(cx)]);

            if let Some(prefix) = self.test_id_prefix.as_deref() {
                item = item.test_id(format!("{prefix}-{}", device.id.as_ref().replace(' ', "-")));
            }

            items.push(item);
        }

        CommandList::new(items)
            .empty_text(self.empty_text)
            .highlight_query_model(controller.query)
            .into_element(cx)
    }
}
