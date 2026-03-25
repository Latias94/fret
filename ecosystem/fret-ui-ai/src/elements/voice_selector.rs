//! AI Elements-aligned `VoiceSelector` surfaces (UI-only).
//!
//! Upstream reference: `repo-ref/ai-elements/packages/elements/src/voice-selector.tsx`.
//!
//! Notes:
//! - Upstream renders a dialog + cmdk list, with optional per-voice preview playback.
//! - In Fret, voice inventory and preview playback are **app-owned**; this module renders the
//!   selector chrome and emits intent seams (`on_value_change`).

use std::sync::Arc;

use fret_core::{
    Color, FontWeight, Px, SemanticsRole, TextAlign, TextOverflow, TextStyleRefinement, TextWrap,
};
use fret_runtime::Model;
use fret_ui::action::ActionCx;
use fret_ui::element::{AnyElement, SemanticsDecoration, TextProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::controllable_state;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::declarative::visually_hidden::visually_hidden;
use fret_ui_kit::typography;
use fret_ui_kit::ui;
use fret_ui_kit::{ChromeRefinement, ColorRef, Items, LayoutRefinement, Radius, Space};
use fret_ui_shadcn::facade::{
    Button, ButtonSize, ButtonVariant, Command, CommandDialog, CommandEmpty, CommandEntry,
    CommandGroup, CommandInput, CommandItem, CommandList, CommandSeparator, CommandShortcut,
    Dialog, DialogContent, DialogTitle, Spinner,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoiceSelectorChildSlot {
    Trigger,
    Content,
}

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

pub fn use_voice_selector_controller<H: UiHost>(
    cx: &ElementContext<'_, H>,
) -> Option<VoiceSelectorController> {
    cx.provided::<VoiceSelectorController>().cloned()
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

fn border_fg(theme: &Theme) -> Color {
    theme
        .color_by_key("border")
        .unwrap_or_else(|| theme.color_required("foreground"))
}

fn inline_children_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    layout: LayoutRefinement,
    children: Vec<AnyElement>,
    inherited_text_style: Option<TextStyleRefinement>,
    inherited_foreground: Option<Color>,
) -> AnyElement {
    let mut element = ui::h_row(move |_cx| children)
        .layout(layout)
        .gap(Space::N0)
        .items(Items::Center)
        .into_element(cx);

    if let Some(text_style) = inherited_text_style {
        element = element.inherit_text_style(text_style);
    }
    if let Some(foreground) = inherited_foreground {
        element = element.inherit_foreground(foreground);
    }

    element
}

fn voice_selector_gender_icon(value: Option<&str>) -> fret_icons::IconId {
    match value {
        Some("male") => fret_icons::IconId::new_static("lucide.mars"),
        Some("female") => fret_icons::IconId::new_static("lucide.venus"),
        Some("transgender") => fret_icons::IconId::new_static("lucide.transgender"),
        Some("androgyne") => fret_icons::IconId::new_static("lucide.mars-stroke"),
        Some("non-binary") => fret_icons::IconId::new_static("lucide.non-binary"),
        Some("intersex") => fret_icons::IconId::new_static("lucide.venus-and-mars"),
        _ => fret_icons::IconId::new_static("lucide.circle-small"),
    }
}

fn voice_selector_accent_emoji(value: Option<&str>) -> Option<&'static str> {
    match value {
        Some("american") => Some("🇺🇸"),
        Some("british") => Some("🇬🇧"),
        Some("australian") => Some("🇦🇺"),
        Some("canadian") => Some("🇨🇦"),
        Some("irish") => Some("🇮🇪"),
        Some("scottish") => Some("🏴"),
        Some("indian") => Some("🇮🇳"),
        Some("south-african") => Some("🇿🇦"),
        Some("new-zealand") => Some("🇳🇿"),
        Some("spanish") => Some("🇪🇸"),
        Some("french") => Some("🇫🇷"),
        Some("german") => Some("🇩🇪"),
        Some("italian") => Some("🇮🇹"),
        Some("portuguese") => Some("🇵🇹"),
        Some("brazilian") => Some("🇧🇷"),
        Some("mexican") => Some("🇲🇽"),
        Some("argentinian") => Some("🇦🇷"),
        Some("japanese") => Some("🇯🇵"),
        Some("chinese") => Some("🇨🇳"),
        Some("korean") => Some("🇰🇷"),
        Some("russian") => Some("🇷🇺"),
        Some("arabic") => Some("🇸🇦"),
        Some("dutch") => Some("🇳🇱"),
        Some("swedish") => Some("🇸🇪"),
        Some("norwegian") => Some("🇳🇴"),
        Some("danish") => Some("🇩🇰"),
        Some("finnish") => Some("🇫🇮"),
        Some("polish") => Some("🇵🇱"),
        Some("turkish") => Some("🇹🇷"),
        Some("greek") => Some("🇬🇷"),
        _ => None,
    }
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

    /// Docs-shaped compound children composition aligned with upstream `<VoiceSelector>...</VoiceSelector>`.
    pub fn children<I, C>(self, children: I) -> VoiceSelectorWithChildren
    where
        I: IntoIterator<Item = C>,
        C: Into<VoiceSelectorChild>,
    {
        VoiceSelectorWithChildren {
            root: self,
            children: children.into_iter().map(Into::into).collect(),
        }
    }

    pub fn trigger(self, trigger: VoiceSelectorTrigger) -> VoiceSelectorWithChildren {
        self.children([VoiceSelectorChild::Trigger(trigger)])
    }

    pub fn content(self, content: VoiceSelectorContent) -> VoiceSelectorWithChildren {
        self.children([VoiceSelectorChild::Content(content)])
    }

    /// Rust-friendly compound entrypoint for the upstream JSX children shape.
    pub fn into_element_with_children<H, F>(
        self,
        cx: &mut ElementContext<'_, H>,
        children: F,
    ) -> AnyElement
    where
        H: UiHost,
        F: Fn(&mut ElementContext<'_, H>, VoiceSelectorChildSlot) -> AnyElement + Clone + 'static,
    {
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
        let controller_for_content = controller;
        let children_for_trigger = children.clone();
        let children_for_content = children;

        Dialog::new(open.clone()).into_element(
            cx,
            move |cx| {
                cx.provide(controller_for_trigger.clone(), |cx| {
                    children_for_trigger(cx, VoiceSelectorChildSlot::Trigger)
                })
            },
            move |cx| {
                cx.provide(controller_for_content.clone(), |cx| {
                    children_for_content(cx, VoiceSelectorChildSlot::Content)
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
            move |cx| cx.provide(controller_for_trigger.clone(), trigger),
            move |cx| cx.provide(controller_for_content.clone(), content),
        )
    }
}

pub enum VoiceSelectorChild {
    Trigger(VoiceSelectorTrigger),
    Content(VoiceSelectorContent),
}

impl std::fmt::Debug for VoiceSelectorChild {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Trigger(_) => f.write_str("VoiceSelectorChild::Trigger(..)"),
            Self::Content(_) => f.write_str("VoiceSelectorChild::Content(..)"),
        }
    }
}

impl From<VoiceSelectorTrigger> for VoiceSelectorChild {
    fn from(value: VoiceSelectorTrigger) -> Self {
        Self::Trigger(value)
    }
}

impl From<VoiceSelectorContent> for VoiceSelectorChild {
    fn from(value: VoiceSelectorContent) -> Self {
        Self::Content(value)
    }
}

pub struct VoiceSelectorWithChildren {
    root: VoiceSelector,
    children: Vec<VoiceSelectorChild>,
}

impl std::fmt::Debug for VoiceSelectorWithChildren {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VoiceSelectorWithChildren")
            .field("root", &self.root)
            .field("children_len", &self.children.len())
            .finish()
    }
}

impl VoiceSelectorWithChildren {
    pub fn children<I, C>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = C>,
        C: Into<VoiceSelectorChild>,
    {
        self.children.extend(children.into_iter().map(Into::into));
        self
    }

    pub fn trigger(self, trigger: VoiceSelectorTrigger) -> Self {
        self.children([VoiceSelectorChild::Trigger(trigger)])
    }

    pub fn content(self, content: VoiceSelectorContent) -> Self {
        self.children([VoiceSelectorChild::Content(content)])
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let mut trigger = None;
        let mut content = None;

        for child in self.children {
            match child {
                VoiceSelectorChild::Trigger(value) => {
                    if trigger.is_some() {
                        debug_assert!(false, "VoiceSelector expects a single VoiceSelectorTrigger");
                    }
                    trigger = Some(value);
                }
                VoiceSelectorChild::Content(value) => {
                    if content.is_some() {
                        debug_assert!(false, "VoiceSelector expects a single VoiceSelectorContent");
                    }
                    content = Some(value);
                }
            }
        }

        let Some(trigger) = trigger else {
            debug_assert!(false, "VoiceSelector requires a VoiceSelectorTrigger");
            return visually_hidden(cx, |_| Vec::<AnyElement>::new());
        };

        let content =
            content.unwrap_or_else(|| VoiceSelectorContent::new(Vec::<AnyElement>::new()));

        self.root.into_element(
            cx,
            move |cx| trigger.into_element(cx),
            move |cx| content.into_element(cx),
        )
    }
}

/// AI Elements-aligned `VoiceSelectorTrigger` (button that opens the dialog).
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(controller) = use_voice_selector_controller(cx) else {
            return visually_hidden(cx, |_| Vec::<AnyElement>::new());
        };

        let open = controller.open.clone();
        let query = controller.query.clone();
        let child = self.child;

        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _| {
            let _ = host.models_mut().update(&query, |v| v.clear());
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
                vec![child]
            },
        );

        if let Some(test_id) = self.test_id {
            out = out.attach_semantics(SemanticsDecoration::default().test_id(test_id));
        }

        out
    }
}

/// AI Elements-aligned `VoiceSelectorContent` (DialogContent + Command container).
pub struct VoiceSelectorContent {
    title: Arc<str>,
    children: Vec<AnyElement>,
    input: Option<VoiceSelectorInput>,
    list: Option<VoiceSelectorList>,
    test_id_root: Option<Arc<str>>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    command_chrome: ChromeRefinement,
    command_layout: LayoutRefinement,
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
            input: None,
            list: None,
            test_id_root: None,
            chrome: ChromeRefinement::default().p(Space::N0),
            layout: LayoutRefinement::default().w_full().min_w_0(),
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

    pub fn input(mut self, input: VoiceSelectorInput) -> Self {
        self.input = Some(input);
        self
    }

    pub fn list(mut self, list: VoiceSelectorList) -> Self {
        self.list = Some(list);
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
            return visually_hidden(cx, |_| Vec::<AnyElement>::new());
        };

        let mut input = CommandInput::new(controller.query)
            .placeholder(self.placeholder)
            .wrapper_height_auto()
            .input_height_auto()
            .input_padding_y_px(Px(12.0))
            .refine_layout(LayoutRefinement::default().w_full().min_w_0())
            .into_element(cx);

        if let Some(test_id) = self.test_id {
            input = input.attach_semantics(SemanticsDecoration::default().test_id(test_id));
        }

        input
    }
}

enum VoiceSelectorListMode {
    Auto,
    Entries(Vec<CommandEntry>),
}

/// Renders a filtered voice list and commits selection on activate.
pub struct VoiceSelectorList {
    empty_text: Arc<str>,
    test_id_prefix: Option<Arc<str>>,
    mode: VoiceSelectorListMode,
    scroll_layout: LayoutRefinement,
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
            mode: VoiceSelectorListMode::Auto,
            scroll_layout: LayoutRefinement::default(),
        }
    }

    pub fn new_entries<I, E>(entries: I) -> Self
    where
        I: IntoIterator<Item = E>,
        E: Into<CommandEntry>,
    {
        Self {
            mode: VoiceSelectorListMode::Entries(entries.into_iter().map(Into::into).collect()),
            ..Self::new()
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

    pub fn entries<I, E>(mut self, entries: I) -> Self
    where
        I: IntoIterator<Item = E>,
        E: Into<CommandEntry>,
    {
        self.mode = VoiceSelectorListMode::Entries(entries.into_iter().map(Into::into).collect());
        self
    }

    pub fn refine_scroll_layout(mut self, layout: LayoutRefinement) -> Self {
        self.scroll_layout = self.scroll_layout.merge(layout);
        self
    }

    /// Rust-friendly render-prop equivalent of upstream `<VoiceSelectorList>{voices => ...}</VoiceSelectorList>`.
    pub fn children<F>(self, children: F) -> VoiceSelectorListWithChildren<F> {
        VoiceSelectorListWithChildren {
            list: self,
            children,
        }
    }

    fn into_element_with_entries<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        empty_text: Arc<str>,
        scroll_layout: LayoutRefinement,
        entries: Vec<CommandEntry>,
    ) -> AnyElement {
        let Some(controller) = use_voice_selector_controller(cx) else {
            return visually_hidden(cx, |_| Vec::<AnyElement>::new());
        };

        CommandList::new_entries(entries)
            .empty_text(empty_text)
            .query_model(controller.query.clone())
            .highlight_query_model(controller.query)
            .refine_scroll_layout(scroll_layout)
            .into_element(cx)
    }

    /// Rust-friendly equivalent of upstream `VoiceSelectorList(children(voices))`.
    pub fn into_element_with_children<H, F, I, E>(
        self,
        cx: &mut ElementContext<'_, H>,
        children: F,
    ) -> AnyElement
    where
        H: UiHost,
        F: FnOnce(Arc<[VoiceSelectorVoice]>) -> I,
        I: IntoIterator<Item = E>,
        E: Into<CommandEntry>,
    {
        let Some(controller) = use_voice_selector_controller(cx) else {
            return visually_hidden(cx, |_| Vec::<AnyElement>::new());
        };

        let Self {
            empty_text,
            scroll_layout,
            ..
        } = self;

        let entries = children(controller.voices.clone())
            .into_iter()
            .map(Into::into)
            .collect::<Vec<_>>();

        Self::into_element_with_entries(cx, empty_text, scroll_layout, entries)
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Self {
            empty_text,
            test_id_prefix,
            mode,
            scroll_layout,
        } = self;

        let Some(controller) = use_voice_selector_controller(cx) else {
            return visually_hidden(cx, |_| Vec::<AnyElement>::new());
        };

        let entries = match mode {
            VoiceSelectorListMode::Auto => {
                let theme = Theme::global(&*cx.app).clone();
                let mut entries: Vec<CommandEntry> = Vec::new();
                for voice in controller.voices.iter() {
                    let id = voice.id.clone();
                    let name = voice.name.clone();
                    let desc = voice.description.clone();

                    let value_model = controller.value.clone();
                    let open_model = controller.open.clone();
                    let query_model = controller.query.clone();
                    let on_value_change = controller.on_value_change.clone();

                    let on_select: fret_ui::action::OnActivate =
                        Arc::new(move |host, action_cx, _| {
                            let _ = host
                                .models_mut()
                                .update(&value_model, |v| *v = Some(id.clone()));
                            let _ = host.models_mut().update(&query_model, |v| v.clear());
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
                            style: Some(typography::preset_text_style_with_overrides(
                                &theme,
                                typography::TypographyPreset::control_ui(
                                    typography::UiTextSize::Sm,
                                ),
                                Some(FontWeight::MEDIUM),
                                None,
                            )),
                            color: None,
                            wrap: TextWrap::None,
                            overflow: TextOverflow::Ellipsis,
                            align: TextAlign::Start,
                            ink_overflow: Default::default(),
                        });
                        let desc_el = desc.clone().map(|d| {
                            cx.text_props(TextProps {
                                layout: Default::default(),
                                text: d,
                                style: Some(typography::preset_text_style_with_overrides(
                                    &theme,
                                    typography::TypographyPreset::control_ui(
                                        typography::UiTextSize::Xs,
                                    ),
                                    Some(FontWeight::NORMAL),
                                    None,
                                )),
                                color: Some(muted_fg(&theme)),
                                wrap: TextWrap::None,
                                overflow: TextOverflow::Ellipsis,
                                align: TextAlign::Start,
                                ink_overflow: Default::default(),
                            })
                        });

                        ui::v_stack(move |_cx| match desc_el {
                            Some(desc_el) => vec![name_el, desc_el],
                            None => vec![name_el],
                        })
                        .layout(LayoutRefinement::default().w_full().min_w_0())
                        .gap(Space::N1)
                        .items(Items::Start)
                        .into_element(cx)
                    };

                    let mut item = CommandItem::new(voice.name.clone())
                        .value(voice.id.clone())
                        .keywords(desc.iter().cloned())
                        .on_select_action(on_select)
                        .children([row]);

                    if let Some(prefix) = test_id_prefix.as_deref() {
                        item = item.test_id(format!("{prefix}-{}", voice.id.as_ref()));
                    }
                    entries.push(item.into());
                }
                entries
            }
            VoiceSelectorListMode::Entries(entries) => entries,
        };

        Self::into_element_with_entries(cx, empty_text, scroll_layout, entries)
    }
}

pub struct VoiceSelectorListWithChildren<F> {
    list: VoiceSelectorList,
    children: F,
}

impl<F> std::fmt::Debug for VoiceSelectorListWithChildren<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VoiceSelectorListWithChildren")
            .field("list", &self.list)
            .finish_non_exhaustive()
    }
}

impl<F> VoiceSelectorListWithChildren<F> {
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

    pub fn into_element<H, I, E>(self, cx: &mut ElementContext<'_, H>) -> AnyElement
    where
        H: UiHost,
        F: FnOnce(Arc<[VoiceSelectorVoice]>) -> I,
        I: IntoIterator<Item = E>,
        E: Into<CommandEntry>,
    {
        self.list.into_element_with_children(cx, self.children)
    }
}

pub type VoiceSelectorDialog = CommandDialog;
pub type VoiceSelectorEmpty = CommandEmpty;
pub type VoiceSelectorGroup = CommandGroup;
pub type VoiceSelectorItem = CommandItem;
pub type VoiceSelectorShortcut = CommandShortcut;
pub type VoiceSelectorSeparator = CommandSeparator;

/// AI Elements-aligned `VoiceSelectorName`.
pub struct VoiceSelectorName {
    text: Arc<str>,
    children: Option<Vec<AnyElement>>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for VoiceSelectorName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VoiceSelectorName")
            .field("text", &self.text.as_ref())
            .field(
                "children_len",
                &self.children.as_ref().map(|children| children.len()),
            )
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .finish()
    }
}

impl VoiceSelectorName {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
            children: None,
            test_id: None,
            layout: LayoutRefinement::default().flex_1().min_w_0(),
        }
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = Some(children.into_iter().collect());
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
        let theme = Theme::global(&*cx.app).clone();
        let name_style = typography::preset_text_style_with_overrides(
            &theme,
            typography::TypographyPreset::control_ui(typography::UiTextSize::Sm),
            Some(FontWeight::MEDIUM),
            None,
        );
        let mut name_refinement = typography::preset_text_refinement(
            &theme,
            typography::TypographyPreset::control_ui(typography::UiTextSize::Sm),
        );
        name_refinement.weight = Some(FontWeight::MEDIUM);
        let mut element = match self.children {
            Some(children) => {
                inline_children_element(cx, self.layout, children, Some(name_refinement), None)
            }
            None => cx.text_props(TextProps {
                layout: decl_style::layout_style(&theme, self.layout),
                text: self.text,
                style: Some(name_style),
                color: None,
                wrap: TextWrap::None,
                overflow: TextOverflow::Ellipsis,
                align: TextAlign::Start,
                ink_overflow: Default::default(),
            }),
        };

        if let Some(test_id) = self.test_id {
            element = element.attach_semantics(SemanticsDecoration::default().test_id(test_id));
        }
        element
    }
}

/// AI Elements-aligned `VoiceSelectorDescription`.
pub struct VoiceSelectorDescription {
    text: Arc<str>,
    children: Option<Vec<AnyElement>>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for VoiceSelectorDescription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VoiceSelectorDescription")
            .field("text", &self.text.as_ref())
            .field(
                "children_len",
                &self.children.as_ref().map(|children| children.len()),
            )
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .finish()
    }
}

impl VoiceSelectorDescription {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
            children: None,
            test_id: None,
            layout: LayoutRefinement::default().min_w_0(),
        }
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = Some(children.into_iter().collect());
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
        let theme = Theme::global(&*cx.app).snapshot();
        let mut element = match self.children {
            Some(children) => inline_children_element(
                cx,
                self.layout,
                children,
                Some(typography::description_text_refinement_with_fallbacks(
                    &theme,
                    "component.voice_selector.description",
                    Some("component.text.xs_px"),
                    Some("component.text.xs_line_height"),
                )),
                Some(typography::muted_foreground_color(&theme)),
            ),
            None => typography::scope_description_text_with_fallbacks(
                cx.text_props(TextProps {
                    layout: decl_style::layout_style(&theme, self.layout),
                    text: self.text,
                    style: None,
                    color: None,
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Ellipsis,
                    align: TextAlign::Start,
                    ink_overflow: Default::default(),
                }),
                &theme,
                "component.voice_selector.description",
                Some("component.text.xs_px"),
                Some("component.text.xs_line_height"),
            ),
        };

        if let Some(test_id) = self.test_id {
            element = element.attach_semantics(SemanticsDecoration::default().test_id(test_id));
        }
        element
    }
}

/// AI Elements-aligned `VoiceSelectorAge`.
pub struct VoiceSelectorAge {
    text: Arc<str>,
    children: Option<Vec<AnyElement>>,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for VoiceSelectorAge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VoiceSelectorAge")
            .field("text", &self.text.as_ref())
            .field(
                "children_len",
                &self.children.as_ref().map(|children| children.len()),
            )
            .field("test_id", &self.test_id.as_deref())
            .finish()
    }
}

impl VoiceSelectorAge {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
            children: None,
            test_id: None,
        }
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = Some(children.into_iter().collect());
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let age_style = typography::preset_text_style_with_overrides(
            &theme,
            typography::TypographyPreset::control_ui(typography::UiTextSize::Xs),
            Some(FontWeight::NORMAL),
            None,
        );
        let age_refinement = typography::preset_text_refinement(
            &theme,
            typography::TypographyPreset::control_ui(typography::UiTextSize::Xs),
        );
        let mut element = match self.children {
            Some(children) => inline_children_element(
                cx,
                LayoutRefinement::default(),
                children,
                Some(age_refinement),
                Some(muted_fg(&theme)),
            ),
            None => cx.text_props(TextProps {
                layout: Default::default(),
                text: self.text,
                style: Some(age_style),
                color: Some(muted_fg(&theme)),
                wrap: TextWrap::None,
                overflow: TextOverflow::Ellipsis,
                align: TextAlign::Start,
                ink_overflow: Default::default(),
            }),
        };

        if let Some(test_id) = self.test_id {
            element = element.attach_semantics(SemanticsDecoration::default().test_id(test_id));
        }
        element
    }
}

/// AI Elements-aligned `VoiceSelectorAttributes`.
pub struct VoiceSelectorAttributes {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    gap: Space,
}

impl std::fmt::Debug for VoiceSelectorAttributes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VoiceSelectorAttributes")
            .field("children_len", &self.children.len())
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .field("gap", &self.gap)
            .finish()
    }
}

impl VoiceSelectorAttributes {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            test_id: None,
            layout: LayoutRefinement::default().min_w_0(),
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
        let mut element = ui::h_row(move |_cx| self.children)
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

/// AI Elements-aligned `VoiceSelectorBullet`.
#[derive(Default)]
pub struct VoiceSelectorBullet {
    children: Option<Vec<AnyElement>>,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for VoiceSelectorBullet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VoiceSelectorBullet")
            .field(
                "children_len",
                &self.children.as_ref().map(|children| children.len()),
            )
            .field("test_id", &self.test_id.as_deref())
            .finish()
    }
}

impl VoiceSelectorBullet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = Some(children.into_iter().collect());
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let bullet_style = typography::preset_text_style_with_overrides(
            &theme,
            typography::TypographyPreset::control_ui(typography::UiTextSize::Xs),
            Some(FontWeight::NORMAL),
            None,
        );
        let bullet_refinement = typography::preset_text_refinement(
            &theme,
            typography::TypographyPreset::control_ui(typography::UiTextSize::Xs),
        );
        let mut element = match self.children {
            Some(children) => inline_children_element(
                cx,
                LayoutRefinement::default(),
                children,
                Some(bullet_refinement),
                Some(border_fg(&theme)),
            ),
            None => cx.text_props(TextProps {
                layout: Default::default(),
                text: Arc::<str>::from("•"),
                style: Some(bullet_style),
                color: Some(border_fg(&theme)),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                align: TextAlign::Start,
                ink_overflow: Default::default(),
            }),
        };

        if let Some(test_id) = self.test_id {
            element = element.attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Generic)
                    .test_id(test_id),
            );
        }
        element
    }
}

/// AI Elements-aligned `VoiceSelectorGender`.
#[derive(Default)]
pub struct VoiceSelectorGender {
    value: Option<Arc<str>>,
    children: Option<Vec<AnyElement>>,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for VoiceSelectorGender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VoiceSelectorGender")
            .field("value", &self.value.as_deref())
            .field(
                "children_len",
                &self.children.as_ref().map(|children| children.len()),
            )
            .field("test_id", &self.test_id.as_deref())
            .finish()
    }
}

impl VoiceSelectorGender {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn value(mut self, value: impl Into<Arc<str>>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = Some(children.into_iter().collect());
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let mut element = match self.children {
            Some(children) => inline_children_element(
                cx,
                LayoutRefinement::default(),
                children,
                Some(typography::preset_text_refinement(
                    &theme,
                    typography::TypographyPreset::control_ui(typography::UiTextSize::Xs),
                )),
                Some(muted_fg(&theme)),
            ),
            None => decl_icon::icon_with(
                cx,
                voice_selector_gender_icon(self.value.as_deref()),
                Some(Px(16.0)),
                Some(ColorRef::Color(muted_fg(&theme))),
            ),
        };

        if let Some(test_id) = self.test_id {
            element = element.attach_semantics(SemanticsDecoration::default().test_id(test_id));
        }
        element
    }
}

/// AI Elements-aligned `VoiceSelectorAccent`.
#[derive(Default)]
pub struct VoiceSelectorAccent {
    value: Option<Arc<str>>,
    children: Option<Vec<AnyElement>>,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for VoiceSelectorAccent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VoiceSelectorAccent")
            .field("value", &self.value.as_deref())
            .field(
                "children_len",
                &self.children.as_ref().map(|children| children.len()),
            )
            .field("test_id", &self.test_id.as_deref())
            .finish()
    }
}

impl VoiceSelectorAccent {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn value(mut self, value: impl Into<Arc<str>>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = Some(children.into_iter().collect());
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let accent_style = typography::preset_text_style_with_overrides(
            &theme,
            typography::TypographyPreset::control_ui(typography::UiTextSize::Xs),
            Some(FontWeight::NORMAL),
            None,
        );
        let accent_refinement = typography::preset_text_refinement(
            &theme,
            typography::TypographyPreset::control_ui(typography::UiTextSize::Xs),
        );
        let mut element = match self.children {
            Some(children) => inline_children_element(
                cx,
                LayoutRefinement::default(),
                children,
                Some(accent_refinement),
                Some(muted_fg(&theme)),
            ),
            None => cx.text_props(TextProps {
                layout: Default::default(),
                text: Arc::<str>::from(
                    voice_selector_accent_emoji(self.value.as_deref()).unwrap_or(""),
                ),
                style: Some(accent_style),
                color: Some(muted_fg(&theme)),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                align: TextAlign::Start,
                ink_overflow: Default::default(),
            }),
        };

        if let Some(test_id) = self.test_id {
            element = element.attach_semantics(SemanticsDecoration::default().test_id(test_id));
        }
        element
    }
}

/// AI Elements-aligned `VoiceSelectorPreview`.
#[derive(Default)]
pub struct VoiceSelectorPreview {
    playing: bool,
    loading: bool,
    on_play: Option<fret_ui::action::OnActivate>,
    children: Option<Vec<AnyElement>>,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for VoiceSelectorPreview {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VoiceSelectorPreview")
            .field("playing", &self.playing)
            .field("loading", &self.loading)
            .field("has_on_play", &self.on_play.is_some())
            .field(
                "children_len",
                &self.children.as_ref().map(|children| children.len()),
            )
            .field("test_id", &self.test_id.as_deref())
            .finish()
    }
}

impl VoiceSelectorPreview {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn playing(mut self, playing: bool) -> Self {
        self.playing = playing;
        self
    }

    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    pub fn on_play_action(mut self, on_play: fret_ui::action::OnActivate) -> Self {
        self.on_play = Some(on_play);
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = Some(children.into_iter().collect());
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let label = if self.playing {
            "Pause preview"
        } else {
            "Play preview"
        };

        let default_icon = if self.loading {
            Spinner::new()
                .refine_layout(LayoutRefinement::default().w_px(Px(12.0)).h_px(Px(12.0)))
                .into_element(cx)
        } else {
            let icon_id = if self.playing {
                fret_icons::IconId::new_static("lucide.pause")
            } else {
                fret_icons::ids::ui::PLAY
            };
            decl_icon::icon_with(cx, icon_id, Some(Px(12.0)), None)
        };

        let button_children = self.children.unwrap_or_else(|| vec![default_icon]);
        let mut button = Button::new(label)
            .variant(ButtonVariant::Outline)
            .size(ButtonSize::IconSm)
            .disabled(self.loading)
            .children(button_children);

        if let Some(on_play) = self.on_play {
            button = button.on_activate(on_play);
        }
        if let Some(test_id) = self.test_id {
            button = button.test_id(test_id);
        }
        button.into_element(cx)
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
            return visually_hidden(cx, |_| Vec::<AnyElement>::new());
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
            style: Some(typography::preset_text_style_with_overrides(
                &theme,
                typography::TypographyPreset::control_ui(typography::UiTextSize::Sm),
                Some(FontWeight::NORMAL),
                None,
            )),
            color: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
            align: TextAlign::Start,
            ink_overflow: Default::default(),
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(controller) = use_voice_selector_controller(cx) else {
            return visually_hidden(cx, |_| Vec::<AnyElement>::new());
        };

        let open = controller.open.clone();
        let query = controller.query.clone();
        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _| {
            let _ = host.models_mut().update(&query, |v| v.clear());
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

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size};
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

    #[test]
    fn voice_selector_list_entries_mode_captures_entries() {
        let list = VoiceSelectorList::new_entries([CommandItem::new("Alloy")]);
        match list.mode {
            VoiceSelectorListMode::Entries(entries) => assert_eq!(entries.len(), 1),
            VoiceSelectorListMode::Auto => panic!("expected entries mode"),
        }
    }

    #[test]
    fn voice_selector_description_scopes_inherited_text_style() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(120.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            VoiceSelectorDescription::new("Description").into_element(cx)
        });

        let ElementKind::Text(props) = &element.kind else {
            panic!("expected VoiceSelectorDescription to be a text element");
        };
        assert!(props.style.is_none());
        assert!(props.color.is_none());
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);

        let theme = fret_ui::Theme::global(&app).snapshot();
        assert_eq!(
            element.inherited_text_style.as_ref(),
            Some(&typography::description_text_refinement_with_fallbacks(
                &theme,
                "component.voice_selector.description",
                Some("component.text.xs_px"),
                Some("component.text.xs_line_height"),
            ))
        );
        assert_eq!(
            element.inherited_foreground,
            Some(typography::muted_foreground_color(&theme))
        );
    }

    #[test]
    fn voice_selector_gender_icon_maps_known_values() {
        assert_eq!(
            voice_selector_gender_icon(Some("female")),
            fret_icons::IconId::new_static("lucide.venus")
        );
        assert_eq!(
            voice_selector_gender_icon(Some("non-binary")),
            fret_icons::IconId::new_static("lucide.non-binary")
        );
        assert_eq!(
            voice_selector_gender_icon(None),
            fret_icons::IconId::new_static("lucide.circle-small")
        );
    }

    #[test]
    fn voice_selector_accent_emoji_maps_common_values() {
        assert_eq!(voice_selector_accent_emoji(Some("american")), Some("🇺🇸"));
        assert_eq!(voice_selector_accent_emoji(Some("british")), Some("🇬🇧"));
        assert_eq!(voice_selector_accent_emoji(Some("unknown")), None);
    }

    #[test]
    fn voice_selector_compound_children_surface_builds() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let voices = Arc::from([VoiceSelectorVoice::new("alloy", "Alloy")]);

        let built = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(400.0), Px(240.0)),
            ),
            "test",
            |cx| {
                VoiceSelector::from_arc(voices)
                    .children([
                        VoiceSelectorChild::Trigger(VoiceSelectorTrigger::new(
                            cx.text("Select Voice"),
                        )),
                        VoiceSelectorChild::Content(
                            VoiceSelectorContent::new(Vec::<AnyElement>::new())
                                .input(VoiceSelectorInput::new())
                                .list(VoiceSelectorList::new_entries(Vec::<CommandEntry>::new())),
                        ),
                    ])
                    .into_element(cx)
            },
        );

        assert!(
            !built.children.is_empty(),
            "expected compound VoiceSelector children surface to build a non-empty tree"
        );
    }

    #[test]
    fn voice_selector_list_children_builder_builds() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let controller = VoiceSelectorController {
            voices: Arc::from([VoiceSelectorVoice::new("alloy", "Alloy")]),
            value: app.models_mut().insert(None),
            open: app.models_mut().insert(false),
            query: app.models_mut().insert(String::new()),
            on_value_change: None,
        };

        let built = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(400.0), Px(240.0)),
            ),
            "test",
            |cx| {
                cx.provide(controller.clone(), |cx| {
                    VoiceSelectorList::new()
                        .children(|voices: Arc<[VoiceSelectorVoice]>| {
                            voices
                                .iter()
                                .map(|voice| VoiceSelectorItem::new(voice.name.clone()))
                                .collect::<Vec<_>>()
                        })
                        .into_element(cx)
                })
            },
        );

        assert!(
            find_text_by_content(&built, "Alloy").is_some(),
            "expected render-prop list builder to render the explicit row"
        );
    }

    #[test]
    fn voice_selector_leaf_children_overrides_render_custom_content() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let built = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(400.0), Px(240.0)),
            ),
            "test",
            |cx| {
                ui::v_stack(|cx| {
                    vec![
                        VoiceSelectorGender::new()
                            .children([cx.text("Custom Gender")])
                            .into_element(cx),
                        VoiceSelectorAccent::new()
                            .value("american")
                            .children([cx.text("US English")])
                            .into_element(cx),
                        VoiceSelectorPreview::new()
                            .children([cx.text("Preview")])
                            .into_element(cx),
                    ]
                })
                .into_element(cx)
            },
        );

        assert!(find_text_by_content(&built, "Custom Gender").is_some());
        assert!(find_text_by_content(&built, "US English").is_some());
        assert!(find_text_by_content(&built, "Preview").is_some());
    }

    #[test]
    fn voice_selector_accent_unknown_renders_empty_text_node() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let built = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(320.0), Px(120.0)),
            ),
            "test",
            |cx| VoiceSelectorAccent::new().value("unknown").into_element(cx),
        );

        let ElementKind::Text(props) = &built.kind else {
            panic!("expected unknown accent to render an empty text node");
        };
        assert_eq!(props.text.as_ref(), "");
    }

    #[test]
    fn voice_selector_surfaces_use_shared_typography_presets() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let controller = VoiceSelectorController {
            voices: std::sync::Arc::from([VoiceSelectorVoice::new("alloy", "Alloy")]),
            value: app.models_mut().insert(None),
            open: app.models_mut().insert(false),
            query: app.models_mut().insert(String::new()),
            on_value_change: None,
        };

        let element = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(320.0), Px(160.0)),
            ),
            "test",
            |cx| {
                cx.provide(controller.clone(), |cx| {
                    ui::v_stack(|cx| {
                        vec![
                            VoiceSelectorValue::new()
                                .placeholder("Pick a voice")
                                .into_element(cx),
                            VoiceSelectorName::new("Alloy").into_element(cx),
                            VoiceSelectorAge::new("Adult").into_element(cx),
                            VoiceSelectorAccent::new()
                                .value("american")
                                .into_element(cx),
                        ]
                    })
                    .into_element(cx)
                })
            },
        );

        let theme = Theme::global(&app).clone();
        let expected_sm_normal = Some(typography::preset_text_style_with_overrides(
            &theme,
            typography::TypographyPreset::control_ui(typography::UiTextSize::Sm),
            Some(FontWeight::NORMAL),
            None,
        ));
        let expected_sm_medium = Some(typography::preset_text_style_with_overrides(
            &theme,
            typography::TypographyPreset::control_ui(typography::UiTextSize::Sm),
            Some(FontWeight::MEDIUM),
            None,
        ));
        let expected_xs_normal = Some(typography::preset_text_style_with_overrides(
            &theme,
            typography::TypographyPreset::control_ui(typography::UiTextSize::Xs),
            Some(FontWeight::NORMAL),
            None,
        ));

        let value = find_text_by_content(&element, "Pick a voice").expect("voice value text");
        assert_eq!(value.style, expected_sm_normal);

        let name = find_text_by_content(&element, "Alloy").expect("voice name text");
        assert_eq!(name.style, expected_sm_medium);

        let age = find_text_by_content(&element, "Adult").expect("voice age text");
        assert_eq!(age.style, expected_xs_normal.clone());

        let accent = find_text_by_content(&element, "🇺🇸").expect("voice accent text");
        assert_eq!(accent.style, expected_xs_normal);
    }
}
