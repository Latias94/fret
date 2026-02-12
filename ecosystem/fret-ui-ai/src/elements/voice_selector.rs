//! AI Elements-aligned `VoiceSelector` surfaces (UI-only).
//!
//! Upstream reference: `repo-ref/ai-elements/packages/elements/src/voice-selector.tsx`.
//!
//! Notes:
//! - Upstream renders a dialog + cmdk list, with optional per-voice preview playback.
//! - In Fret, voice inventory and preview playback are **app-owned**; this module renders the
//!   selector chrome and emits intent seams (`on_value_change`).

use std::sync::Arc;

use fret_core::{Color, FontId, FontWeight, SemanticsRole, TextOverflow, TextStyle, TextWrap};
use fret_runtime::Model;
use fret_ui::action::ActionCx;
use fret_ui::element::{AnyElement, SemanticsDecoration, TextProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::controllable_state;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::visually_hidden::visually_hidden;
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, Space};
use fret_ui_shadcn::{
    Button, ButtonVariant, Command, CommandInput, CommandItem, CommandList, Dialog, DialogContent,
    DialogTitle,
};

pub type OnVoiceSelectorValueChange =
    Arc<dyn Fn(&mut dyn fret_ui::action::UiActionHost, ActionCx, Option<Arc<str>>) + 'static>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoiceSelectorVoice {
    pub id: Arc<str>,
    pub name: Arc<str>,
    pub description: Option<Arc<str>>,
}

impl VoiceSelectorVoice {
    pub fn new(id: impl Into<Arc<str>>, name: impl Into<Arc<str>>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
        }
    }

    pub fn description(mut self, description: impl Into<Arc<str>>) -> Self {
        self.description = Some(description.into());
        self
    }
}

#[derive(Clone)]
pub struct VoiceSelectorController {
    pub voices: Arc<[VoiceSelectorVoice]>,
    pub value: Model<Option<Arc<str>>>,
    pub open: Model<bool>,
    pub query: Model<String>,
    pub on_value_change: Option<OnVoiceSelectorValueChange>,
}

impl std::fmt::Debug for VoiceSelectorController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VoiceSelectorController")
            .field("voices_len", &self.voices.len())
            .field("value", &self.value.id())
            .field("open", &self.open.id())
            .field("query", &self.query.id())
            .field("has_on_value_change", &self.on_value_change.is_some())
            .finish()
    }
}

#[derive(Debug, Default, Clone)]
struct VoiceSelectorProviderState {
    controller: Option<VoiceSelectorController>,
}

pub fn use_voice_selector_controller<H: UiHost>(
    cx: &ElementContext<'_, H>,
) -> Option<VoiceSelectorController> {
    cx.inherited_state::<VoiceSelectorProviderState>()
        .and_then(|st| st.controller.clone())
}

#[derive(Default)]
struct VoiceSelectorState {
    query: Option<Model<String>>,
}

fn query_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    cx.with_state(VoiceSelectorState::default, |st| st.query.clone())
        .unwrap_or_else(|| {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(VoiceSelectorState::default, |st| {
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

fn text_xs(theme: &Theme) -> TextStyle {
    TextStyle {
        font: FontId::default(),
        size: theme.metric_required("component.text.xs_px"),
        weight: FontWeight::NORMAL,
        slant: Default::default(),
        line_height: Some(theme.metric_required("component.text.xs_line_height")),
        letter_spacing_em: None,
    }
}

fn muted_fg(theme: &Theme) -> Color {
    theme
        .color_by_key("muted-foreground")
        .or_else(|| theme.color_by_key("muted_foreground"))
        .unwrap_or_else(|| theme.color_required("foreground"))
}

/// AI Elements-aligned `VoiceSelector` root.
#[derive(Clone)]
pub struct VoiceSelector {
    voices: Arc<[VoiceSelectorVoice]>,
    value: Option<Model<Option<Arc<str>>>>,
    default_value: Option<Arc<str>>,
    open: Option<Model<bool>>,
    default_open: bool,
    on_value_change: Option<OnVoiceSelectorValueChange>,
}

impl std::fmt::Debug for VoiceSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VoiceSelector")
            .field("voices_len", &self.voices.len())
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

impl VoiceSelector {
    pub fn new(voices: impl IntoIterator<Item = VoiceSelectorVoice>) -> Self {
        Self::from_arc(voices.into_iter().collect::<Vec<_>>().into())
    }

    pub fn from_arc(voices: Arc<[VoiceSelectorVoice]>) -> Self {
        Self {
            voices,
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

    pub fn on_value_change(mut self, cb: OnVoiceSelectorValueChange) -> Self {
        self.on_value_change = Some(cb);
        self
    }

    pub fn into_element<H: UiHost + 'static>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    ) -> AnyElement {
        let open = fret_ui_kit::primitives::dialog::DialogRoot::new()
            .open(self.open.clone())
            .default_open(self.default_open)
            .open_model(cx);

        let value = controllable_state::use_controllable_model(cx, self.value.clone(), || {
            self.default_value.clone()
        })
        .model();

        let query = query_model(cx);
        let controller = VoiceSelectorController {
            voices: self.voices.clone(),
            value: value.clone(),
            open: open.clone(),
            query: query.clone(),
            on_value_change: self.on_value_change.clone(),
        };

        let controller_for_trigger = controller.clone();
        let controller_for_content = controller.clone();

        Dialog::new(open.clone()).into_element(
            cx,
            move |cx| {
                cx.with_state(VoiceSelectorProviderState::default, |st| {
                    st.controller = Some(controller_for_trigger.clone());
                });
                trigger(cx)
            },
            move |cx| {
                cx.with_state(VoiceSelectorProviderState::default, |st| {
                    st.controller = Some(controller_for_content.clone());
                });
                content(cx)
            },
        )
    }
}

/// AI Elements-aligned `VoiceSelectorTrigger` (button that opens the dialog).
#[derive(Clone)]
pub struct VoiceSelectorTrigger {
    child: AnyElement,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for VoiceSelectorTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VoiceSelectorTrigger")
            .field("test_id", &self.test_id.as_deref())
            .finish()
    }
}

impl VoiceSelectorTrigger {
    pub fn new(child: AnyElement) -> Self {
        Self {
            child,
            test_id: None,
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(controller) = use_voice_selector_controller(cx) else {
            return visually_hidden(cx, |_| Vec::new());
        };

        let open = controller.open.clone();
        let child = self.child;

        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _| {
            let _ = host.models_mut().update(&open, |v| *v = true);
            host.notify(action_cx);
            host.request_redraw(action_cx.window);
        });

        let mut out = cx.pressable(
            fret_ui::element::PressableProps {
                enabled: true,
                focusable: true,
                a11y: fret_ui::element::PressableA11y {
                    role: Some(SemanticsRole::Button),
                    test_id: self.test_id.clone(),
                    ..Default::default()
                },
                ..Default::default()
            },
            move |cx, _st| {
                cx.pressable_on_activate(on_activate.clone());
                vec![child.clone()]
            },
        );

        if let Some(test_id) = self.test_id {
            out = out.attach_semantics(SemanticsDecoration::default().test_id(test_id));
        }

        out
    }
}

/// AI Elements-aligned `VoiceSelectorContent` (DialogContent + Command container).
#[derive(Clone)]
pub struct VoiceSelectorContent {
    title: Arc<str>,
    children: Vec<AnyElement>,
    test_id_root: Option<Arc<str>>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for VoiceSelectorContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VoiceSelectorContent")
            .field("title", &self.title.as_ref())
            .field("children_len", &self.children.len())
            .field("test_id_root", &self.test_id_root.as_deref())
            .finish()
    }
}

impl VoiceSelectorContent {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            title: Arc::from("Voice Selector"),
            children: children.into_iter().collect(),
            test_id_root: None,
            chrome: ChromeRefinement::default().p(Space::N0),
            layout: LayoutRefinement::default().w_full().min_w_0(),
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let title = self.title;
        let hidden_title =
            visually_hidden(cx, move |cx| vec![DialogTitle::new(title).into_element(cx)]);

        let command = Command::new(self.children)
            .refine_layout(LayoutRefinement::default().w_full().min_w_0())
            .into_element(cx);

        let mut out = DialogContent::new([hidden_title, command])
            .refine_style(self.chrome)
            .refine_layout(self.layout)
            .into_element(cx);

        if let Some(test_id) = self.test_id_root {
            out = out.attach_semantics(SemanticsDecoration::default().test_id(test_id));
        }

        out
    }
}

/// AI Elements-aligned input (search).
#[derive(Clone)]
pub struct VoiceSelectorInput {
    placeholder: Arc<str>,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for VoiceSelectorInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VoiceSelectorInput")
            .field("placeholder", &self.placeholder.as_ref())
            .field("test_id", &self.test_id.as_deref())
            .finish()
    }
}

impl VoiceSelectorInput {
    pub fn new() -> Self {
        Self {
            placeholder: Arc::from("Search voices..."),
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
        let Some(controller) = use_voice_selector_controller(cx) else {
            return visually_hidden(cx, |_| Vec::new());
        };

        let mut input = CommandInput::new(controller.query)
            .placeholder(self.placeholder)
            .refine_layout(LayoutRefinement::default().w_full().min_w_0())
            .into_element(cx);

        if let Some(test_id) = self.test_id {
            input = input.attach_semantics(SemanticsDecoration::default().test_id(test_id));
        }

        input
    }
}

/// Renders a filtered voice list and commits selection on activate.
#[derive(Clone)]
pub struct VoiceSelectorList {
    empty_text: Arc<str>,
    test_id_prefix: Option<Arc<str>>,
}

impl std::fmt::Debug for VoiceSelectorList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VoiceSelectorList")
            .field("empty_text", &self.empty_text.as_ref())
            .field("test_id_prefix", &self.test_id_prefix.as_deref())
            .finish()
    }
}

impl VoiceSelectorList {
    pub fn new() -> Self {
        Self {
            empty_text: Arc::from("No voice found."),
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
        let Some(controller) = use_voice_selector_controller(cx) else {
            return visually_hidden(cx, |_| Vec::new());
        };

        let theme = Theme::global(&*cx.app).clone();

        let query = cx
            .app
            .models()
            .read(&controller.query, |s| s.trim().to_ascii_lowercase())
            .ok()
            .unwrap_or_default();

        let mut items: Vec<CommandItem> = Vec::new();
        for voice in controller.voices.iter() {
            let haystack = format!(
                "{} {}",
                voice.name.as_ref(),
                voice.description.as_deref().unwrap_or("")
            )
            .to_ascii_lowercase();
            if !query.is_empty() && !haystack.contains(&query) {
                continue;
            }

            let id = voice.id.clone();
            let name = voice.name.clone();
            let desc = voice.description.clone();

            let value_model = controller.value.clone();
            let open_model = controller.open.clone();
            let on_value_change = controller.on_value_change.clone();

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

            let row = {
                let name_el = cx.text_props(TextProps {
                    layout: Default::default(),
                    text: name,
                    style: Some(text_sm(&theme, FontWeight::MEDIUM)),
                    color: None,
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Ellipsis,
                });
                let desc_el = desc.map(|d| {
                    cx.text_props(TextProps {
                        layout: Default::default(),
                        text: d,
                        style: Some(text_xs(&theme)),
                        color: Some(muted_fg(&theme)),
                        wrap: TextWrap::None,
                        overflow: TextOverflow::Ellipsis,
                    })
                });

                stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .layout(LayoutRefinement::default().w_full().min_w_0())
                        .gap(Space::N1)
                        .items_start(),
                    move |_cx| match desc_el {
                        Some(desc_el) => vec![name_el, desc_el],
                        None => vec![name_el],
                    },
                )
            };

            let mut item = CommandItem::new(voice.name.clone())
                .value(voice.id.clone())
                .on_select_action(on_select)
                .children([row]);

            if let Some(prefix) = self.test_id_prefix.as_deref() {
                item = item.test_id(format!("{prefix}-{}", voice.id.as_ref()));
            }
            items.push(item);
        }

        CommandList::new(items)
            .empty_text(self.empty_text)
            .highlight_query_model(controller.query)
            .into_element(cx)
    }
}

/// Renders the selected voice name (or a placeholder).
#[derive(Clone)]
pub struct VoiceSelectorValue {
    placeholder: Arc<str>,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for VoiceSelectorValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VoiceSelectorValue")
            .field("placeholder", &self.placeholder.as_ref())
            .field("test_id", &self.test_id.as_deref())
            .finish()
    }
}

impl VoiceSelectorValue {
    pub fn new() -> Self {
        Self {
            placeholder: Arc::from("Select voice..."),
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
        let Some(controller) = use_voice_selector_controller(cx) else {
            return visually_hidden(cx, |_| Vec::new());
        };

        let theme = Theme::global(&*cx.app).clone();
        let selected = cx
            .app
            .models()
            .read(&controller.value, |v| v.clone())
            .ok()
            .flatten();

        let text: Arc<str> = if let Some(id) = selected {
            if let Some(voice) = controller
                .voices
                .iter()
                .find(|v| v.id.as_ref() == id.as_ref())
            {
                voice.name.clone()
            } else {
                id
            }
        } else {
            self.placeholder.clone()
        };

        let mut out = cx.text_props(TextProps {
            layout: Default::default(),
            text,
            style: Some(text_sm(&theme, FontWeight::NORMAL)),
            color: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
        });

        if let Some(test_id) = self.test_id {
            out = out.attach_semantics(SemanticsDecoration::default().test_id(test_id));
        }
        out
    }
}

/// Default trigger button used in demos/recipes.
#[derive(Clone)]
pub struct VoiceSelectorButton {
    test_id: Option<Arc<str>>,
}

impl Default for VoiceSelectorButton {
    fn default() -> Self {
        Self { test_id: None }
    }
}

impl VoiceSelectorButton {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(controller) = use_voice_selector_controller(cx) else {
            return visually_hidden(cx, |_| Vec::new());
        };

        let open = controller.open.clone();
        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _| {
            let _ = host.models_mut().update(&open, |v| *v = true);
            host.notify(action_cx);
            host.request_redraw(action_cx.window);
        });

        let mut btn = Button::new("Voice")
            .variant(ButtonVariant::Outline)
            .children([VoiceSelectorValue::new().into_element(cx)])
            .on_activate(on_activate);

        if let Some(test_id) = self.test_id {
            btn = btn.test_id(test_id);
        }

        btn.into_element(cx)
    }
}
