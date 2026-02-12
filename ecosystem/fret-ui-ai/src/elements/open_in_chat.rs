//! AI Elements-aligned "open in chat" menu surfaces.
//!
//! Upstream reference: `repo-ref/ai-elements/packages/elements/src/open-in-chat.tsx`.

use std::sync::Arc;

use fret_core::Px;
use fret_runtime::{Effect, Model};
use fret_ui::action::OnActivate;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::{ColorRef, LayoutRefinement, Space};
use fret_ui_shadcn::{
    Button, ButtonVariant, DropdownMenu, DropdownMenuAlign, DropdownMenuEntry, DropdownMenuItem,
    DropdownMenuSide,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenInProvider {
    ChatGpt,
    Claude,
    T3,
    Scira,
    V0,
    Cursor,
}

impl OpenInProvider {
    fn title(self) -> &'static str {
        match self {
            Self::ChatGpt => "Open in ChatGPT",
            Self::Claude => "Open in Claude",
            Self::T3 => "Open in T3 Chat",
            Self::Scira => "Open in Scira",
            Self::V0 => "Open in v0",
            Self::Cursor => "Open in Cursor",
        }
    }

    fn icon(self) -> fret_icons::IconId {
        match self {
            Self::ChatGpt => fret_icons::IconId::new_static("lucide.sparkles"),
            Self::Claude => fret_icons::IconId::new_static("lucide.feather"),
            Self::T3 => fret_icons::IconId::new_static("lucide.message-circle"),
            Self::Scira => fret_icons::IconId::new_static("lucide.sparkle"),
            Self::V0 => fret_icons::IconId::new_static("lucide.wand"),
            Self::Cursor => fret_icons::IconId::new_static("lucide.mouse-pointer-click"),
        }
    }

    fn create_url(self, query: &str) -> String {
        let q = url_encode_component(query);
        match self {
            Self::ChatGpt => format!("https://chatgpt.com/?hints=search&prompt={q}"),
            Self::Claude => format!("https://claude.ai/new?q={q}"),
            Self::T3 => format!("https://t3.chat/new?q={q}"),
            Self::Scira => format!("https://scira.ai/?q={q}"),
            Self::V0 => format!("https://v0.app?q={q}"),
            Self::Cursor => format!("https://cursor.com/link/prompt?text={q}"),
        }
    }
}

fn url_encode_component(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for b in text.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*b as char)
            }
            b' ' => out.push_str("%20"),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

#[derive(Clone)]
pub struct OpenInController {
    pub query: Arc<str>,
}

impl std::fmt::Debug for OpenInController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenInController")
            .field("query_len", &self.query.len())
            .finish()
    }
}

#[derive(Debug, Default, Clone)]
struct OpenInProviderState {
    controller: Option<OpenInController>,
}

pub fn use_open_in_controller<H: UiHost>(cx: &ElementContext<'_, H>) -> Option<OpenInController> {
    cx.inherited_state::<OpenInProviderState>()
        .and_then(|st| st.controller.clone())
}

#[derive(Default)]
struct OpenInMenuState {
    open: Option<Model<bool>>,
}

fn open_in_open_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<bool> {
    let open = cx.with_state(OpenInMenuState::default, |st| st.open.clone());
    match open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(OpenInMenuState::default, |st| st.open = Some(model.clone()));
            model
        }
    }
}

#[derive(Clone)]
/// Open-in menu root aligned with AI Elements `OpenIn` + `OpenInTrigger` outcomes.
pub struct OpenIn {
    query: Arc<str>,
    modal: bool,
    align: DropdownMenuAlign,
    side: DropdownMenuSide,
    side_offset: Px,
    min_width: Px,
    trigger: OpenInTrigger,
}

impl std::fmt::Debug for OpenIn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenIn")
            .field("query_len", &self.query.len())
            .field("modal", &self.modal)
            .field("align", &self.align)
            .field("side", &self.side)
            .field("side_offset", &self.side_offset)
            .field("min_width", &self.min_width)
            .finish()
    }
}

impl OpenIn {
    pub fn new(query: impl Into<Arc<str>>) -> Self {
        Self {
            query: query.into(),
            modal: false,
            align: DropdownMenuAlign::Start,
            side: DropdownMenuSide::Bottom,
            side_offset: Px(4.0),
            min_width: Px(240.0),
            trigger: OpenInTrigger::new(),
        }
    }

    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = modal;
        self
    }

    pub fn align(mut self, align: DropdownMenuAlign) -> Self {
        self.align = align;
        self
    }

    pub fn side(mut self, side: DropdownMenuSide) -> Self {
        self.side = side;
        self
    }

    pub fn side_offset(mut self, offset: Px) -> Self {
        self.side_offset = offset;
        self
    }

    pub fn min_width(mut self, width: Px) -> Self {
        self.min_width = width;
        self
    }

    pub fn trigger(mut self, trigger: OpenInTrigger) -> Self {
        self.trigger = trigger;
        self
    }

    pub fn into_element_with_entries<H: UiHost + 'static>(
        self,
        cx: &mut ElementContext<'_, H>,
        entries: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<DropdownMenuEntry> + 'static,
    ) -> AnyElement {
        let open = open_in_open_model(cx);

        let controller = OpenInController { query: self.query };
        cx.with_state(OpenInProviderState::default, |st| {
            st.controller = Some(controller.clone());
        });

        let modal = self.modal;
        let align = self.align;
        let side = self.side;
        let side_offset = self.side_offset;
        let min_width = self.min_width;
        let trigger = self.trigger;
        let controller_for_trigger = controller.clone();
        let controller_for_entries = controller.clone();

        DropdownMenu::new(open.clone())
            .modal(modal)
            .align(align)
            .side(side)
            .side_offset(side_offset)
            .min_width(min_width)
            .into_element(
                cx,
                move |cx| {
                    cx.with_state(OpenInProviderState::default, |st| {
                        st.controller = Some(controller_for_trigger.clone());
                    });
                    trigger.into_element_with_open(cx, open.clone())
                },
                move |cx| {
                    cx.with_state(OpenInProviderState::default, |st| {
                        st.controller = Some(controller_for_entries.clone());
                    });
                    let out = entries(cx);
                    if out.is_empty() {
                        return vec![OpenInChatGpt::new().into_entry(cx)];
                    }
                    out
                },
            )
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.into_element_with_entries(cx, move |cx| {
            vec![
                OpenInChatGpt::new().into_entry(cx),
                OpenInClaude::new().into_entry(cx),
                OpenInT3::new().into_entry(cx),
                OpenInScira::new().into_entry(cx),
                OpenInv0::new().into_entry(cx),
                OpenInCursor::new().into_entry(cx),
            ]
        })
    }
}

#[derive(Clone)]
pub struct OpenInTrigger {
    label: Arc<str>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl OpenInTrigger {
    pub fn new() -> Self {
        Self {
            label: Arc::<str>::from("Open in chat"),
            test_id: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.label = label.into();
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

    pub fn into_element_with_open<H: UiHost + 'static>(
        self,
        cx: &mut ElementContext<'_, H>,
        open: Model<bool>,
    ) -> AnyElement {
        let label = self.label.clone();
        let on_toggle: OnActivate = Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&open, |v| *v = !*v);
            host.notify(action_cx);
        });

        let theme = Theme::global(&*cx.app).clone();
        let chevron = decl_icon::icon_with(
            cx,
            fret_icons::ids::ui::CHEVRON_DOWN,
            Some(Px(16.0)),
            Some(ColorRef::Color(theme.color_required("foreground"))),
        );

        let row = stack::hstack(
            cx,
            stack::HStackProps::default().items_center().gap(Space::N2),
            move |cx| vec![cx.text(label.as_ref()), chevron],
        );

        let mut btn = Button::new("Open in chat")
            .children([row])
            .variant(ButtonVariant::Outline)
            .on_activate(on_toggle)
            .refine_layout(self.layout);
        if let Some(id) = self.test_id {
            btn = btn.test_id(id);
        }
        btn.into_element(cx)
    }
}

fn open_in_item_entry<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    provider: OpenInProvider,
    test_id: Option<Arc<str>>,
) -> DropdownMenuEntry {
    let Some(controller) = use_open_in_controller(cx) else {
        return DropdownMenuEntry::Item(DropdownMenuItem::new(provider.title()));
    };

    let url = provider.create_url(&controller.query);
    let label = provider.title();

    let icon = decl_icon::icon_with(
        cx,
        provider.icon(),
        Some(Px(16.0)),
        Some(ColorRef::Color(
            Theme::global(&*cx.app).color_required("muted-foreground"),
        )),
    );
    let external = decl_icon::icon_with(
        cx,
        fret_icons::IconId::new_static("lucide.external-link"),
        Some(Px(16.0)),
        Some(ColorRef::Color(
            Theme::global(&*cx.app).color_required("muted-foreground"),
        )),
    );

    let on_activate: OnActivate = Arc::new(move |host, action_cx, _reason| {
        host.push_effect(Effect::OpenUrl {
            url: url.clone(),
            target: None,
            rel: None,
        });
        host.notify(action_cx);
    });

    let mut item = DropdownMenuItem::new(label);
    item.leading = Some(icon);
    item.trailing = Some(external);
    item.on_activate = Some(on_activate);
    if let Some(id) = test_id {
        item.test_id = Some(id);
    }
    DropdownMenuEntry::Item(item)
}

#[derive(Clone, Default)]
pub struct OpenInChatGpt {
    test_id: Option<Arc<str>>,
}

impl OpenInChatGpt {
    pub fn new() -> Self {
        Self { test_id: None }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_entry<H: UiHost + 'static>(
        self,
        cx: &mut ElementContext<'_, H>,
    ) -> DropdownMenuEntry {
        open_in_item_entry(cx, OpenInProvider::ChatGpt, self.test_id)
    }
}

#[derive(Clone, Default)]
pub struct OpenInClaude {
    test_id: Option<Arc<str>>,
}

impl OpenInClaude {
    pub fn new() -> Self {
        Self { test_id: None }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_entry<H: UiHost + 'static>(
        self,
        cx: &mut ElementContext<'_, H>,
    ) -> DropdownMenuEntry {
        open_in_item_entry(cx, OpenInProvider::Claude, self.test_id)
    }
}

#[derive(Clone, Default)]
pub struct OpenInT3 {
    test_id: Option<Arc<str>>,
}

impl OpenInT3 {
    pub fn new() -> Self {
        Self { test_id: None }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_entry<H: UiHost + 'static>(
        self,
        cx: &mut ElementContext<'_, H>,
    ) -> DropdownMenuEntry {
        open_in_item_entry(cx, OpenInProvider::T3, self.test_id)
    }
}

#[derive(Clone, Default)]
pub struct OpenInScira {
    test_id: Option<Arc<str>>,
}

impl OpenInScira {
    pub fn new() -> Self {
        Self { test_id: None }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_entry<H: UiHost + 'static>(
        self,
        cx: &mut ElementContext<'_, H>,
    ) -> DropdownMenuEntry {
        open_in_item_entry(cx, OpenInProvider::Scira, self.test_id)
    }
}

#[derive(Clone, Default)]
pub struct OpenInv0 {
    test_id: Option<Arc<str>>,
}

impl OpenInv0 {
    pub fn new() -> Self {
        Self { test_id: None }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_entry<H: UiHost + 'static>(
        self,
        cx: &mut ElementContext<'_, H>,
    ) -> DropdownMenuEntry {
        open_in_item_entry(cx, OpenInProvider::V0, self.test_id)
    }
}

#[derive(Clone, Default)]
pub struct OpenInCursor {
    test_id: Option<Arc<str>>,
}

impl OpenInCursor {
    pub fn new() -> Self {
        Self { test_id: None }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_entry<H: UiHost + 'static>(
        self,
        cx: &mut ElementContext<'_, H>,
    ) -> DropdownMenuEntry {
        open_in_item_entry(cx, OpenInProvider::Cursor, self.test_id)
    }
}
