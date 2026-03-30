use std::sync::Arc;
use std::time::{Duration, SystemTime};

use fret_core::{
    ClipboardAccessError, Color, Corners, Edges, FontId, FontWeight, Px, SemanticsRole,
    TextOverflow, TextStyle, TextWrap, window::ColorScheme,
};
use fret_icons::ids;
use fret_runtime::Effect;
use fret_ui::element::{
    AnyElement, ContainerProps, LayoutStyle, Length, PressableProps, SemanticsDecoration,
    SizeStyle, TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::chrome::centered_fixed_chrome_pressable_with_id_props;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::typography;
use fret_ui_kit::ui;
use fret_ui_kit::{
    ChromeRefinement, ColorFallback, ColorRef, Items, Justify, LayoutRefinement, MetricRef, Radius,
    Space,
};
use fret_ui_shadcn::facade::{
    Avatar, AvatarFallback, Collapsible, CollapsibleContent, CollapsibleTrigger,
};

use super::clipboard_copy::{
    ClipboardCopyFeedbackRef, begin_request, finish_request, handle_reset_timer,
};

pub type OnCommitFilePathClick = Arc<
    dyn Fn(&mut dyn fret_ui::action::UiActionHost, fret_ui::action::ActionCx, Arc<str>) + 'static,
>;

fn alpha(color: Color, a: f32) -> Color {
    Color {
        r: color.r,
        g: color.g,
        b: color.b,
        a: (color.a * a).clamp(0.0, 1.0),
    }
}

fn muted_fg(theme: &Theme) -> Color {
    theme
        .color_by_key("muted-foreground")
        .or_else(|| theme.color_by_key("muted_foreground"))
        .unwrap_or_else(|| theme.color_token("foreground"))
}

fn inline_children<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    children: Vec<AnyElement>,
) -> AnyElement {
    let mut iter = children.into_iter();
    match (iter.next(), iter.next()) {
        (None, _) => cx.text(""),
        (Some(first), None) => first,
        (Some(first), Some(second)) => {
            let mut row_children = Vec::with_capacity(2 + iter.len());
            row_children.push(first);
            row_children.push(second);
            row_children.extend(iter);

            ui::h_row(move |_cx| row_children)
                .gap(Space::N0)
                .items(Items::Center)
                .layout(LayoutRefinement::default())
                .into_element(cx)
        }
    }
}

fn status_color(theme: &Theme, status: CommitFileStatusKind) -> Color {
    let key = match status {
        CommitFileStatusKind::Added => "color.commit.file.added",
        CommitFileStatusKind::Deleted => "color.commit.file.deleted",
        CommitFileStatusKind::Modified => "color.commit.file.modified",
        CommitFileStatusKind::Renamed => "color.commit.file.renamed",
    };

    if let Some(color) = theme.color_by_key(key) {
        return color;
    }

    let scheme_is_dark = theme.color_scheme == Some(ColorScheme::Dark);
    let rgb = match (status, scheme_is_dark) {
        // Tailwind: green-600 (#16a34a), dark: green-400 (#4ade80).
        (CommitFileStatusKind::Added, false) => 0x16_a3_4a,
        (CommitFileStatusKind::Added, true) => 0x4a_de_80,
        // Tailwind: red-600 (#dc2626), dark: red-400 (#f87171).
        (CommitFileStatusKind::Deleted, false) => 0xdc_26_26,
        (CommitFileStatusKind::Deleted, true) => 0xf8_71_71,
        // Tailwind: yellow-600 (#ca8a04), dark: yellow-400 (#facc15).
        (CommitFileStatusKind::Modified, false) => 0xca_8a_04,
        (CommitFileStatusKind::Modified, true) => 0xfa_cc_15,
        // Tailwind: blue-600 (#2563eb), dark: blue-400 (#60a5fa).
        (CommitFileStatusKind::Renamed, false) => 0x25_63_eb,
        (CommitFileStatusKind::Renamed, true) => 0x60_a5_fa,
    };
    fret_ui_kit::colors::linear_from_hex_rgb(rgb)
}

fn monospace_text_style(theme: &Theme, size: Px, weight: FontWeight) -> TextStyle {
    typography::as_control_text(TextStyle {
        font: FontId::monospace(),
        size,
        weight,
        slant: Default::default(),
        line_height: Some(theme.metric_token("metric.font.mono_line_height")),
        letter_spacing_em: None,
        ..Default::default()
    })
}

/// Commit disclosure root aligned with AI Elements `commit.tsx`.
#[derive(Debug)]
pub struct Commit {
    default_open: bool,
    header: Option<CommitHeader>,
    content: Option<CommitContent>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl Commit {
    /// Docs-shaped compound root aligned with upstream `<Commit>...</Commit>`.
    pub fn root() -> Self {
        Self {
            default_open: false,
            header: None,
            content: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn new(header: CommitHeader, content: CommitContent) -> Self {
        Self::root().header(header).content(content)
    }

    pub fn children(mut self, children: impl IntoIterator<Item = CommitChild>) -> Self {
        for child in children {
            match child {
                CommitChild::Header(header) => {
                    if self.header.is_some() {
                        debug_assert!(false, "Commit expects a single CommitHeader");
                    }
                    self.header = Some(header);
                }
                CommitChild::Content(content) => {
                    if self.content.is_some() {
                        debug_assert!(false, "Commit expects a single CommitContent");
                    }
                    self.content = Some(content);
                }
            }
        }
        self
    }

    pub fn header(mut self, header: CommitHeader) -> Self {
        self.header = Some(header);
        self
    }

    pub fn content(mut self, content: CommitContent) -> Self {
        self.content = Some(content);
        self
    }

    pub fn default_open(mut self, default_open: bool) -> Self {
        self.default_open = default_open;
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
        let Some(header) = self.header else {
            debug_assert!(false, "Commit requires a CommitHeader");
            return cx.container(Default::default(), |_| Vec::new());
        };
        let base_chrome = ChromeRefinement::default()
            .rounded(Radius::Lg)
            .border_1()
            .bg(ColorRef::Token {
                key: "background",
                fallback: ColorFallback::ThemePanelBackground,
            })
            .border_color(ColorRef::Token {
                key: "border",
                fallback: ColorFallback::ThemePanelBorder,
            });

        let content = self
            .content
            .unwrap_or_else(|| CommitContent::new(Vec::<AnyElement>::new()));

        Collapsible::uncontrolled(self.default_open)
            .refine_layout(self.layout)
            .refine_style(base_chrome.merge(self.chrome))
            .into_element_with_open_model(
                cx,
                move |cx, open_model, is_open| header.into_trigger(cx, open_model, is_open),
                move |cx| content.into_element(cx),
            )
    }
}

#[derive(Debug)]
pub enum CommitChild {
    Header(CommitHeader),
    Content(CommitContent),
}

impl From<CommitHeader> for CommitChild {
    fn from(value: CommitHeader) -> Self {
        Self::Header(value)
    }
}

impl From<CommitContent> for CommitChild {
    fn from(value: CommitContent) -> Self {
        Self::Content(value)
    }
}

/// Commit disclosure header row (Collapsible trigger).
pub struct CommitHeader {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl std::fmt::Debug for CommitHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommitHeader")
            .field("children_len", &self.children.len())
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .field("chrome", &self.chrome)
            .finish()
    }
}

impl CommitHeader {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            test_id: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default().p(Space::N3),
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

    fn into_trigger<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        open_model: fret_runtime::Model<bool>,
        is_open: bool,
    ) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let children = self.children;
        let test_id = self.test_id;
        let layout = self.layout;
        let chrome = self.chrome;

        let hover_layout = decl_style::layout_style(&theme, layout.clone());
        let hover = fret_ui::element::HoverRegionProps {
            layout: hover_layout,
        };

        let row = cx.hover_region(hover, move |cx, hovered| {
            let opacity = if hovered { 0.8 } else { 1.0 };

            let row = ui::h_row(move |_cx| children)
                .layout(layout)
                .gap(Space::N4)
                .items(Items::Center)
                .justify(Justify::Between)
                .into_element(cx);

            let row = cx.container(
                decl_style::container_props(&theme, chrome, LayoutRefinement::default()),
                move |_cx| vec![row],
            );

            vec![cx.opacity(opacity, move |_cx| vec![row])]
        });

        let trigger = CollapsibleTrigger::new(open_model, vec![row])
            .a11y_label("Toggle commit details")
            .into_element(cx, is_open);

        let Some(test_id) = test_id else {
            return trigger;
        };
        trigger.attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Button)
                .test_id(test_id),
        )
    }
}

/// Commit disclosure content wrapper (`CollapsibleContent`).
#[derive(Debug)]
pub struct CommitContent {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl CommitContent {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            test_id: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
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
        let border = theme.color_token("border");

        let mut wrapper = ContainerProps::default();
        wrapper.layout = decl_style::layout_style(&theme, self.layout);
        wrapper.padding = Edges::all(MetricRef::space(Space::N3).resolve(&theme)).into();
        wrapper.border = Edges {
            top: Px(1.0),
            right: Px(0.0),
            bottom: Px(0.0),
            left: Px(0.0),
        };
        wrapper.border_color = Some(border);

        let content = CollapsibleContent::new([cx.container(wrapper, move |_cx| self.children)])
            .refine_style(self.chrome)
            .into_element(cx);

        let Some(test_id) = self.test_id else {
            return content;
        };
        content.attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Group)
                .test_id(test_id),
        )
    }
}

/// Monospace hash label aligned with AI Elements `CommitHash`.
#[derive(Debug)]
pub struct CommitHash {
    hash: Arc<str>,
    children: Vec<AnyElement>,
}

impl CommitHash {
    pub fn new(hash: impl Into<Arc<str>>) -> Self {
        Self {
            hash: hash.into(),
            children: Vec::new(),
        }
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let color = muted_fg(&theme);
        let icon = decl_icon::icon_with(
            cx,
            ids::ui::GIT_COMMIT,
            Some(Px(12.0)),
            Some(ColorRef::Color(color)),
        );

        let text = if self.children.is_empty() {
            cx.text_props(TextProps {
                layout: LayoutStyle::default(),
                text: self.hash,
                style: Some(monospace_text_style(
                    &theme,
                    theme
                        .metric_by_key("component.commit.hash_text_px")
                        .unwrap_or(Px(12.0)),
                    FontWeight::NORMAL,
                )),
                color: Some(color),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                align: fret_core::TextAlign::Start,
                ink_overflow: Default::default(),
            })
        } else {
            inline_children(cx, self.children)
        };

        ui::h_row(move |_cx| vec![icon, text])
            .gap(Space::N1)
            .items(Items::Center)
            .layout(LayoutRefinement::default())
            .into_element(cx)
    }
}

/// Commit message label aligned with AI Elements `CommitMessage`.
#[derive(Debug)]
pub struct CommitMessage {
    text: Arc<str>,
    children: Vec<AnyElement>,
}

impl CommitMessage {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
            children: Vec::new(),
        }
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        if !self.children.is_empty() {
            return inline_children(cx, self.children);
        }

        let theme = Theme::global(&*cx.app).clone();
        cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text: self.text,
            style: Some(typography::as_control_text(TextStyle {
                font: FontId::default(),
                size: theme
                    .metric_by_key("component.commit.message_text_px")
                    .unwrap_or_else(|| theme.metric_token("font.size")),
                weight: FontWeight::MEDIUM,
                slant: Default::default(),
                line_height: Some(theme.metric_token("font.line_height")),
                letter_spacing_em: None,
                ..Default::default()
            })),
            color: Some(theme.color_token("foreground")),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            ink_overflow: Default::default(),
        })
    }
}

/// Metadata row aligned with AI Elements `CommitMetadata`.
#[derive(Debug)]
pub struct CommitMetadata {
    children: Vec<AnyElement>,
    layout: LayoutRefinement,
}

impl CommitMetadata {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        ui::h_row(move |_cx| self.children)
            .layout(self.layout)
            .gap(Space::N2)
            .items(Items::Center)
            .into_element(cx)
    }
}

/// Separator token aligned with AI Elements `CommitSeparator`.
#[derive(Debug)]
pub struct CommitSeparator {
    text: Arc<str>,
    children: Vec<AnyElement>,
}

impl Default for CommitSeparator {
    fn default() -> Self {
        Self {
            text: Arc::<str>::from("•"),
            children: Vec::new(),
        }
    }
}

impl CommitSeparator {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
            children: Vec::new(),
        }
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        if !self.children.is_empty() {
            return inline_children(cx, self.children);
        }

        let theme = Theme::global(&*cx.app).clone();
        cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text: self.text,
            style: Some(typography::fixed_line_box_style(
                FontId::default(),
                Px(12.0),
                Px(16.0),
            )),
            color: Some(muted_fg(&theme)),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            ink_overflow: Default::default(),
        })
    }
}

/// Vertical info column aligned with AI Elements `CommitInfo`.
#[derive(Debug)]
pub struct CommitInfo {
    children: Vec<AnyElement>,
}

impl CommitInfo {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
        }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        ui::v_flex(move |_cx| self.children)
            .layout(LayoutRefinement::default().flex_1().min_w_0())
            .gap(Space::N1)
            .into_element(cx)
    }
}

/// Author row aligned with AI Elements `CommitAuthor`.
#[derive(Debug)]
pub struct CommitAuthor {
    children: Vec<AnyElement>,
}

impl CommitAuthor {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
        }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        ui::h_row(move |_cx| self.children)
            .gap(Space::N2)
            .items(Items::Center)
            .layout(LayoutRefinement::default())
            .into_element(cx)
    }
}

/// Author avatar aligned with AI Elements `CommitAuthorAvatar`.
#[derive(Debug, Clone)]
pub struct CommitAuthorAvatar {
    initials: Arc<str>,
}

impl CommitAuthorAvatar {
    pub fn new(initials: impl Into<Arc<str>>) -> Self {
        Self {
            initials: initials.into(),
        }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        Avatar::new([AvatarFallback::new(self.initials)
            .delay_ms(0)
            .into_element(cx)])
        .refine_layout(
            LayoutRefinement::default()
                .w_px(MetricRef::space(Space::N8))
                .h_px(MetricRef::space(Space::N8)),
        )
        .into_element(cx)
    }
}

/// Relative timestamp aligned with AI Elements `CommitTimestamp`.
#[derive(Debug)]
pub struct CommitTimestamp {
    date: SystemTime,
    children: Vec<AnyElement>,
}

impl CommitTimestamp {
    pub fn new(date: SystemTime) -> Self {
        Self {
            date,
            children: Vec::new(),
        }
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        if !self.children.is_empty() {
            return inline_children(cx, self.children);
        }

        let theme = Theme::global(&*cx.app).clone();
        let text = relative_days_label(self.date);
        cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text: Arc::<str>::from(text),
            style: Some(typography::fixed_line_box_style(
                FontId::default(),
                Px(12.0),
                Px(16.0),
            )),
            color: Some(muted_fg(&theme)),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            ink_overflow: Default::default(),
        })
    }
}

fn relative_days_label(date: SystemTime) -> String {
    let now = SystemTime::now();
    let delta = match date.duration_since(now) {
        Ok(future) => future.as_secs_f64(),
        Err(past) => -(past.duration().as_secs_f64()),
    };
    let days = (delta / 86_400.0).round() as i64;
    match days {
        0 => "today".to_string(),
        1 => "tomorrow".to_string(),
        -1 => "yesterday".to_string(),
        n if n > 1 => format!("in {n} days"),
        n => format!("{} days ago", -n),
    }
}

/// Actions row aligned with AI Elements `CommitActions`.
#[derive(Debug)]
pub struct CommitActions {
    children: Vec<AnyElement>,
}

impl CommitActions {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
        }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        ui::h_row(move |_cx| self.children)
            .gap(Space::N1)
            .items(Items::Center)
            .layout(LayoutRefinement::default().flex_shrink_0())
            .into_element(cx)
    }
}

/// Copy button aligned with AI Elements `CommitCopyButton`.
pub struct CommitCopyButton {
    hash: Arc<str>,
    on_copy: Option<
        Arc<dyn Fn(&mut dyn fret_ui::action::UiActionHost, fret_ui::action::ActionCx) + 'static>,
    >,
    on_error: Option<
        Arc<
            dyn Fn(
                    &mut dyn fret_ui::action::UiActionHost,
                    fret_ui::action::ActionCx,
                    ClipboardAccessError,
                ) + 'static,
        >,
    >,
    children: Vec<AnyElement>,
    timeout: Duration,
    test_id: Option<Arc<str>>,
    copied_marker_test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for CommitCopyButton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommitCopyButton")
            .field("hash_len", &self.hash.len())
            .field("timeout_ms", &self.timeout.as_millis())
            .field("test_id", &self.test_id.as_deref())
            .field(
                "copied_marker_test_id",
                &self.copied_marker_test_id.as_deref(),
            )
            .finish()
    }
}

impl CommitCopyButton {
    pub fn new(hash: impl Into<Arc<str>>) -> Self {
        Self {
            hash: hash.into(),
            on_copy: None,
            on_error: None,
            children: Vec::new(),
            timeout: Duration::from_millis(2000),
            test_id: None,
            copied_marker_test_id: None,
        }
    }

    /// Called after clipboard write completion succeeds.
    pub fn on_copy(
        mut self,
        on_copy: Arc<
            dyn Fn(&mut dyn fret_ui::action::UiActionHost, fret_ui::action::ActionCx) + 'static,
        >,
    ) -> Self {
        self.on_copy = Some(on_copy);
        self
    }

    /// Called after clipboard write completion fails.
    pub fn on_error(
        mut self,
        on_error: Arc<
            dyn Fn(
                    &mut dyn fret_ui::action::UiActionHost,
                    fret_ui::action::ActionCx,
                    ClipboardAccessError,
                ) + 'static,
        >,
    ) -> Self {
        self.on_error = Some(on_error);
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    /// Optional marker that only exists while the button is in the "copied" state.
    ///
    /// This is intended for `fretboard diag` scripts.
    pub fn copied_marker_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.copied_marker_test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let feedback = cx.slot_state(ClipboardCopyFeedbackRef::default, |st| st.clone());

        let hash = self.hash;
        let on_copy = self.on_copy;
        let on_error = self.on_error;
        let children = self.children;
        let timeout = self.timeout;
        let test_id = self.test_id;
        let copied_marker_test_id = self.copied_marker_test_id;

        centered_fixed_chrome_pressable_with_id_props(cx, move |cx, st, id| {
            let copied = feedback.is_copied();
            let label: Arc<str> = if copied {
                Arc::<str>::from("Copied")
            } else {
                Arc::<str>::from("Copy commit hash")
            };

            cx.timer_on_timer_for(
                id,
                Arc::new({
                    let feedback = feedback.clone();
                    move |host, action_cx, token| {
                        if !handle_reset_timer(&feedback, token) {
                            return false;
                        }
                        host.notify(action_cx);
                        host.request_redraw(action_cx.window);
                        true
                    }
                }),
            );

            cx.pressable_on_clipboard_write_completed({
                let feedback = feedback.clone();
                let on_copy = on_copy.clone();
                let on_error = on_error.clone();
                Arc::new(move |host, action_cx, token, outcome| {
                    let Some(result) =
                        finish_request(&feedback, token, outcome, || host.next_timer_token())
                    else {
                        return false;
                    };

                    if let Some(prev_reset) = result.prev_reset {
                        host.push_effect(Effect::CancelTimer { token: prev_reset });
                    }
                    if let Some(reset_token) = result.next_reset {
                        host.push_effect(Effect::SetTimer {
                            window: Some(action_cx.window),
                            token: reset_token,
                            after: timeout,
                            repeat: None,
                        });
                    }
                    if let Some(error) = result.error {
                        if let Some(on_error) = on_error.as_ref() {
                            on_error(host, action_cx, error);
                        }
                    } else if let Some(on_copy) = on_copy.as_ref() {
                        on_copy(host, action_cx);
                    }
                    host.notify(action_cx);
                    host.request_redraw(action_cx.window);
                    true
                })
            });

            cx.pressable_on_activate({
                let hash = hash.clone();
                let feedback = feedback.clone();
                Arc::new(move |host, action_cx, _reason| {
                    let Some(request) = begin_request(&feedback, || host.next_clipboard_token())
                    else {
                        return;
                    };

                    if let Some(prev_reset) = request.prev_reset {
                        host.push_effect(Effect::CancelTimer { token: prev_reset });
                    }
                    host.push_effect(Effect::ClipboardWriteText {
                        window: action_cx.window,
                        token: request.clipboard_token,
                        text: hash.to_string(),
                    });
                })
            });

            let mut pressable = PressableProps::default();
            pressable.enabled = true;
            pressable.focusable = true;
            pressable.a11y.role = Some(SemanticsRole::Button);
            pressable.a11y.label = Some(label);
            pressable.a11y.test_id = test_id.clone();

            let fg = theme.color_token("muted-foreground");
            let bg_hover = theme.color_token("color.menu.item.hover");
            let bg_pressed = theme.color_token("accent");

            let bg = if st.pressed {
                alpha(bg_pressed, 0.9)
            } else if st.hovered {
                alpha(bg_hover, 0.9)
            } else {
                Color::TRANSPARENT
            };

            let size = Px(28.0);
            let radius = theme.metric_token("metric.radius.sm");

            let icon_id = if copied {
                ids::ui::CHECK
            } else {
                ids::ui::COPY
            };
            let icon = decl_icon::icon_with(cx, icon_id, Some(Px(14.0)), Some(ColorRef::Color(fg)));

            let mut chrome_props = ContainerProps::default();
            chrome_props.layout.size.width = Length::Px(size);
            chrome_props.layout.size.height = Length::Px(size);
            chrome_props.layout.flex.shrink = 0.0;
            chrome_props.background = Some(bg);
            chrome_props.corner_radii = Corners::all(radius);
            chrome_props.border = Edges::all(Px(0.0));
            chrome_props.padding = Edges::all(Px(0.0)).into();

            (pressable, chrome_props, move |cx| {
                let row_children = if children.is_empty() {
                    vec![icon]
                } else {
                    children
                };
                let row = ui::h_row(move |_cx| row_children)
                    .items(Items::Center)
                    .justify(Justify::Center)
                    .layout(LayoutRefinement::default().w_full().h_full())
                    .into_element(cx);

                let marker = copied_marker_test_id.clone().and_then(|marker_id| {
                    copied.then(|| {
                        cx.text_props(TextProps {
                            layout: LayoutStyle {
                                size: SizeStyle {
                                    width: Length::Px(Px(0.0)),
                                    height: Length::Px(Px(0.0)),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            text: Arc::<str>::from(""),
                            style: None,
                            color: None,
                            wrap: TextWrap::None,
                            overflow: TextOverflow::Clip,
                            align: fret_core::TextAlign::Start,
                            ink_overflow: Default::default(),
                        })
                        .attach_semantics(
                            SemanticsDecoration::default()
                                .role(SemanticsRole::Generic)
                                .test_id(marker_id),
                        )
                    })
                });

                let mut children = vec![row];
                if let Some(marker) = marker {
                    children.push(marker);
                }
                children
            })
        })
    }
}

/// Wrapper aligned with AI Elements `CommitFiles`.
#[derive(Debug)]
pub struct CommitFiles {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
}

impl CommitFiles {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            test_id: None,
        }
    }

    pub fn test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let el = ui::v_flex(move |_cx| self.children)
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N1)
            .into_element(cx);

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

/// Wrapper aligned with AI Elements `CommitFile`.
#[derive(Debug)]
pub struct CommitFile {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
}

impl CommitFile {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            test_id: None,
        }
    }

    pub fn test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let hover_bg = theme
            .color_by_key("muted")
            .map(|c| alpha(c, 0.5))
            .unwrap_or_else(|| alpha(theme.color_token("accent"), 0.2));

        let children = self.children;
        let test_id = self.test_id;
        let el = cx.hover_region(
            fret_ui::element::HoverRegionProps::default(),
            move |cx, hovered| {
                let bg = hovered.then_some(hover_bg);
                let mut props = ContainerProps::default();
                props.layout = decl_style::layout_style(
                    &theme,
                    LayoutRefinement::default().w_full().min_w_0(),
                );
                props.padding = Edges::symmetric(
                    MetricRef::space(Space::N2).resolve(&theme),
                    MetricRef::space(Space::N1).resolve(&theme),
                )
                .into();
                props.background = bg;
                props.corner_radii = Corners::all(MetricRef::radius(Radius::Sm).resolve(&theme));

                let row = ui::h_row(move |_cx| children)
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .gap(Space::N2)
                    .items(Items::Center)
                    .justify(Justify::Between)
                    .into_element(cx);
                vec![cx.container(props, move |_cx| vec![row])]
            },
        );

        let Some(test_id) = test_id else {
            return el;
        };
        el.attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Group)
                .test_id(test_id),
        )
    }
}

/// Wrapper aligned with AI Elements `CommitFileInfo`.
#[derive(Debug)]
pub struct CommitFileInfo {
    children: Vec<AnyElement>,
}

impl CommitFileInfo {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
        }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        ui::h_row(move |_cx| self.children)
            .layout(LayoutRefinement::default().min_w_0())
            .gap(Space::N2)
            .items(Items::Center)
            .into_element(cx)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommitFileStatusKind {
    Added,
    Modified,
    Deleted,
    Renamed,
}

/// Status label aligned with AI Elements `CommitFileStatus`.
#[derive(Debug)]
pub struct CommitFileStatus {
    status: CommitFileStatusKind,
    children: Vec<AnyElement>,
}

impl CommitFileStatus {
    pub fn new(status: CommitFileStatusKind) -> Self {
        Self {
            status,
            children: Vec::new(),
        }
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        if !self.children.is_empty() {
            return inline_children(cx, self.children);
        }

        let theme = Theme::global(&*cx.app).clone();

        let label: Arc<str> = match self.status {
            CommitFileStatusKind::Added => Arc::<str>::from("A"),
            CommitFileStatusKind::Deleted => Arc::<str>::from("D"),
            CommitFileStatusKind::Modified => Arc::<str>::from("M"),
            CommitFileStatusKind::Renamed => Arc::<str>::from("R"),
        };

        cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text: label,
            style: Some(monospace_text_style(&theme, Px(12.0), FontWeight::MEDIUM)),
            color: Some(status_color(&theme, self.status)),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            ink_overflow: Default::default(),
        })
    }
}

/// File icon aligned with AI Elements `CommitFileIcon`.
#[derive(Debug, Clone, Default)]
pub struct CommitFileIcon;

impl CommitFileIcon {
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        decl_icon::icon_with(
            cx,
            ids::ui::FILE,
            Some(Px(14.0)),
            Some(ColorRef::Color(muted_fg(&theme))),
        )
    }
}

/// Monospace path label aligned with AI Elements `CommitFilePath`.
pub struct CommitFilePath {
    path: Arc<str>,
    children: Vec<AnyElement>,
    on_click: Option<OnCommitFilePathClick>,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for CommitFilePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommitFilePath")
            .field("path_len", &self.path.len())
            .field("children_len", &self.children.len())
            .field("has_on_click", &self.on_click.is_some())
            .field("test_id", &self.test_id.as_deref())
            .finish()
    }
}

impl CommitFilePath {
    pub fn new(path: impl Into<Arc<str>>) -> Self {
        Self {
            path: path.into(),
            children: Vec::new(),
            on_click: None,
            test_id: None,
        }
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    pub fn on_click(mut self, on_click: OnCommitFilePathClick) -> Self {
        self.on_click = Some(on_click);
        self
    }

    pub fn test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let fg_primary = theme
            .color_by_key("primary")
            .unwrap_or_else(|| theme.color_token("foreground"));

        let path = self.path;
        let custom_children = self.children;
        let base_props = TextProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    min_width: Some(Length::Px(Px(0.0))),
                    ..Default::default()
                },
                ..Default::default()
            },
            text: path.clone(),
            style: Some(monospace_text_style(&theme, Px(12.0), FontWeight::NORMAL)),
            color: Some(theme.color_token("foreground")),
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
            align: fret_core::TextAlign::Start,
            ink_overflow: Default::default(),
        };

        let Some(on_click) = self.on_click else {
            let text = if custom_children.is_empty() {
                cx.text_props(base_props)
            } else {
                inline_children(cx, custom_children)
            };
            let Some(test_id) = self.test_id else {
                return text;
            };
            return text.attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Generic)
                    .test_id(test_id),
            );
        };

        let mut pressable = PressableProps::default();
        pressable.enabled = true;
        pressable.focusable = true;
        pressable.a11y.role = Some(SemanticsRole::Button);
        pressable.a11y.label = Some(Arc::<str>::from("Open commit file"));
        pressable.a11y.test_id = self.test_id.clone();

        let payload = path;
        let text_props = TextProps {
            color: Some(fg_primary),
            ..base_props
        };

        cx.pressable(pressable, move |cx, _st| {
            cx.pressable_on_activate({
                let on_click = on_click.clone();
                let payload = payload.clone();
                Arc::new(move |host, action_cx, _| {
                    on_click(host, action_cx, payload.clone());
                })
            });

            if custom_children.is_empty() {
                vec![cx.text_props(text_props.clone())]
            } else {
                vec![inline_children(cx, custom_children)]
            }
        })
    }
}

/// Wrapper aligned with AI Elements `CommitFileChanges`.
#[derive(Debug)]
pub struct CommitFileChanges {
    children: Vec<AnyElement>,
}

impl CommitFileChanges {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
        }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        ui::h_row(move |_cx| self.children)
            .gap(Space::N1)
            .items(Items::Center)
            .layout(LayoutRefinement::default().flex_shrink_0())
            .into_element(cx)
    }
}

/// Additions count aligned with AI Elements `CommitFileAdditions`.
#[derive(Debug)]
pub struct CommitFileAdditions {
    count: u32,
    children: Vec<AnyElement>,
}

impl CommitFileAdditions {
    pub fn new(count: u32) -> Self {
        Self {
            count,
            children: Vec::new(),
        }
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        if self.count == 0 {
            return cx.text("");
        }

        if !self.children.is_empty() {
            return inline_children(cx, self.children);
        }

        let theme = Theme::global(&*cx.app).clone();
        let color = status_color(&theme, CommitFileStatusKind::Added);
        let icon = decl_icon::icon_with(
            cx,
            ids::ui::PLUS,
            Some(Px(12.0)),
            Some(ColorRef::Color(color)),
        );
        let text = cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text: Arc::<str>::from(format!("{}", self.count)),
            style: Some(monospace_text_style(&theme, Px(12.0), FontWeight::NORMAL)),
            color: Some(color),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            ink_overflow: Default::default(),
        });

        ui::h_row(move |_cx| vec![icon, text])
            .gap(Space::N0)
            .items(Items::Center)
            .layout(LayoutRefinement::default())
            .into_element(cx)
    }
}

/// Deletions count aligned with AI Elements `CommitFileDeletions`.
#[derive(Debug)]
pub struct CommitFileDeletions {
    count: u32,
    children: Vec<AnyElement>,
}

impl CommitFileDeletions {
    pub fn new(count: u32) -> Self {
        Self {
            count,
            children: Vec::new(),
        }
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        if self.count == 0 {
            return cx.text("");
        }

        if !self.children.is_empty() {
            return inline_children(cx, self.children);
        }

        let theme = Theme::global(&*cx.app).clone();
        let color = status_color(&theme, CommitFileStatusKind::Deleted);
        let icon = decl_icon::icon_with(
            cx,
            ids::ui::MINUS,
            Some(Px(12.0)),
            Some(ColorRef::Color(color)),
        );
        let text = cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text: Arc::<str>::from(format!("{}", self.count)),
            style: Some(monospace_text_style(&theme, Px(12.0), FontWeight::NORMAL)),
            color: Some(color),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            ink_overflow: Default::default(),
        });

        ui::h_row(move |_cx| vec![icon, text])
            .gap(Space::N0)
            .items(Items::Center)
            .layout(LayoutRefinement::default())
            .into_element(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Rect, SemanticsRole, Size};
    use fret_ui::element::{ElementKind, PressableProps, TextProps};

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(420.0), Px(220.0)),
        )
    }

    fn find_text_by_content<'a>(el: &'a AnyElement, text: &str) -> Option<&'a TextProps> {
        match &el.kind {
            ElementKind::Text(props) if props.text.as_ref() == text => Some(props),
            _ => el
                .children
                .iter()
                .find_map(|child| find_text_by_content(child, text)),
        }
    }

    fn find_pressable_by_label<'a>(el: &'a AnyElement, label: &str) -> Option<&'a PressableProps> {
        match &el.kind {
            ElementKind::Pressable(props) if props.a11y.label.as_deref() == Some(label) => {
                Some(props)
            }
            _ => el
                .children
                .iter()
                .find_map(|child| find_pressable_by_label(child, label)),
        }
    }

    #[test]
    fn commit_root_children_compose_header_and_content() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "commit", |cx| {
                Commit::root()
                    .children([
                        CommitHeader::new([cx.text("header-title")]).into(),
                        CommitContent::new([cx.text("content-body")])
                            .test_id("commit-content")
                            .into(),
                    ])
                    .default_open(true)
                    .into_element(cx)
            });

        assert!(
            find_text_by_content(&element, "header-title").is_some(),
            "root children should wire the header into the trigger row"
        );
        assert!(
            find_text_by_content(&element, "content-body").is_some(),
            "root children should wire the content into the collapsible body"
        );
    }

    #[test]
    fn commit_hash_and_message_custom_children_replace_default_text() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "commit", |cx| {
                ui::v_flex(move |cx| {
                    vec![
                        CommitHash::new("abc1234")
                            .children([cx.text("custom-hash")])
                            .into_element(cx),
                        CommitMessage::new("default-message")
                            .children([cx.text("custom-message")])
                            .into_element(cx),
                    ]
                })
                .into_element(cx)
            });

        assert!(
            find_text_by_content(&element, "custom-hash").is_some(),
            "custom hash children should render"
        );
        assert!(
            find_text_by_content(&element, "abc1234").is_none(),
            "default hash text should not render when custom children are provided"
        );
        assert!(
            find_text_by_content(&element, "custom-message").is_some(),
            "custom message children should render"
        );
        assert!(
            find_text_by_content(&element, "default-message").is_none(),
            "default message text should not render when custom children are provided"
        );
    }

    #[test]
    fn commit_file_path_custom_children_keep_button_semantics() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let on_click: OnCommitFilePathClick = Arc::new(|_, _, _| {});

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "commit", |cx| {
                CommitFilePath::new("src/default.rs")
                    .on_click(on_click.clone())
                    .children([cx.text("src/custom.rs")])
                    .into_element(cx)
            });

        assert!(
            find_text_by_content(&element, "src/custom.rs").is_some(),
            "custom path children should render"
        );
        assert!(
            find_text_by_content(&element, "src/default.rs").is_none(),
            "default path text should not render when custom children are provided"
        );

        let pressable =
            find_pressable_by_label(&element, "Open commit file").expect("commit file path button");
        assert_eq!(pressable.a11y.role, Some(SemanticsRole::Button));
    }

    #[test]
    fn commit_copy_button_and_counts_support_custom_children() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "commit", |cx| {
                ui::v_flex(move |cx| {
                    vec![
                        CommitCopyButton::new("deadbeef")
                            .children([cx.text("copy-slot")])
                            .into_element(cx),
                        CommitFileAdditions::new(12)
                            .children([cx.text("add-slot")])
                            .into_element(cx),
                        CommitFileDeletions::new(5)
                            .children([cx.text("del-slot")])
                            .into_element(cx),
                    ]
                })
                .into_element(cx)
            });

        assert!(
            find_text_by_content(&element, "copy-slot").is_some(),
            "copy button custom children should render"
        );
        assert!(
            find_text_by_content(&element, "add-slot").is_some(),
            "custom additions content should render"
        );
        assert!(
            find_text_by_content(&element, "del-slot").is_some(),
            "custom deletions content should render"
        );
        assert!(
            find_text_by_content(&element, "12").is_none(),
            "default additions count should not render when custom children are provided"
        );
        assert!(
            find_text_by_content(&element, "5").is_none(),
            "default deletions count should not render when custom children are provided"
        );

        let pressable = find_pressable_by_label(&element, "Copy commit hash").expect("copy button");
        assert_eq!(pressable.a11y.role, Some(SemanticsRole::Button));
    }
}
