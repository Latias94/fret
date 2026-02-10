use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use fret_core::{
    Color, Corners, Edges, FontId, FontWeight, Px, SemanticsRole, TextOverflow, TextStyle,
    TextWrap, TimerToken,
};
use fret_icons::ids;
use fret_runtime::Effect;
use fret_ui::element::{
    AnyElement, ContainerProps, LayoutStyle, Length, PressableProps, SemanticsDecoration,
    SizeStyle, TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{
    ChromeRefinement, ColorFallback, ColorRef, LayoutRefinement, MetricRef, Radius, Space,
};
use fret_ui_shadcn::{Collapsible, CollapsibleContent};

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
        .unwrap_or_else(|| theme.color_required("foreground"))
}

fn status_color(status: CommitFileStatusKind) -> ColorRef {
    match status {
        CommitFileStatusKind::Added => ColorRef::Token {
            key: "color.commit.file.added",
            fallback: ColorFallback::Color(Color {
                r: 0.086,
                g: 0.639,
                b: 0.290,
                a: 1.0,
            }),
        },
        CommitFileStatusKind::Deleted => ColorRef::Token {
            key: "color.commit.file.deleted",
            fallback: ColorFallback::Color(Color {
                r: 0.863,
                g: 0.149,
                b: 0.149,
                a: 1.0,
            }),
        },
        CommitFileStatusKind::Modified => ColorRef::Token {
            key: "color.commit.file.modified",
            fallback: ColorFallback::Color(Color {
                r: 0.792,
                g: 0.541,
                b: 0.016,
                a: 1.0,
            }),
        },
        CommitFileStatusKind::Renamed => ColorRef::Token {
            key: "color.commit.file.renamed",
            fallback: ColorFallback::Color(Color {
                r: 0.145,
                g: 0.388,
                b: 0.922,
                a: 1.0,
            }),
        },
    }
}

fn monospace_text_style(theme: &Theme, size: Px, weight: FontWeight) -> TextStyle {
    TextStyle {
        font: FontId::monospace(),
        size,
        weight,
        slant: Default::default(),
        line_height: Some(theme.metric_required("metric.font.mono_line_height")),
        letter_spacing_em: None,
    }
}

#[derive(Debug, Default)]
struct CopyFeedback {
    copied: bool,
    token: Option<TimerToken>,
}

#[derive(Clone, Default)]
struct CopyFeedbackRef(Arc<Mutex<CopyFeedback>>);

impl CopyFeedbackRef {
    fn lock(&self) -> std::sync::MutexGuard<'_, CopyFeedback> {
        self.0.lock().unwrap_or_else(|e| e.into_inner())
    }
}

/// Commit disclosure root aligned with AI Elements `commit.tsx`.
#[derive(Debug, Clone)]
pub struct Commit {
    default_open: bool,
    header: CommitHeader,
    content: CommitContent,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl Commit {
    pub fn new(header: CommitHeader, content: CommitContent) -> Self {
        Self {
            default_open: false,
            header,
            content,
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
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

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
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

        let header = self.header;
        let content = self.content;

        Collapsible::uncontrolled(self.default_open)
            .refine_layout(self.layout)
            .refine_style(base_chrome.merge(self.chrome))
            .into_element_with_open_model(
                cx,
                move |cx, open_model, is_open| header.clone().into_trigger(cx, open_model, is_open),
                move |cx| content.clone().into_element(cx),
            )
    }
}

/// Commit disclosure header row (Collapsible trigger).
#[derive(Clone)]
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

    fn into_trigger<H: UiHost + 'static>(
        self,
        cx: &mut ElementContext<'_, H>,
        open_model: fret_runtime::Model<bool>,
        is_open: bool,
    ) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let row = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(self.layout)
                .gap(Space::N4)
                .items_center()
                .justify_between(),
            move |_cx| self.children.clone(),
        );

        let row = cx.container(
            decl_style::container_props(&theme, self.chrome, LayoutRefinement::default()),
            move |_cx| vec![row],
        );

        let trigger = fret_ui_shadcn::CollapsibleTrigger::new(open_model, vec![row])
            .a11y_label("Toggle commit details")
            .into_element(cx, is_open);

        let Some(test_id) = self.test_id else {
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
#[derive(Debug, Clone)]
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
        let border = theme.color_required("border");

        let mut wrapper = ContainerProps::default();
        wrapper.layout = decl_style::layout_style(&theme, self.layout);
        wrapper.padding = Edges::all(MetricRef::space(Space::N3).resolve(&theme));
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
#[derive(Debug, Clone)]
pub struct CommitHash {
    hash: Arc<str>,
}

impl CommitHash {
    pub fn new(hash: impl Into<Arc<str>>) -> Self {
        Self { hash: hash.into() }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let icon = decl_icon::icon_with(
            cx,
            ids::ui::GIT_COMMIT,
            Some(Px(12.0)),
            Some(ColorRef::Color(theme.color_required("foreground"))),
        );

        let text = cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text: self.hash,
            style: Some(monospace_text_style(
                &theme,
                theme
                    .metric_by_key("component.commit.hash_text_px")
                    .unwrap_or(Px(12.0)),
                FontWeight::NORMAL,
            )),
            color: Some(theme.color_required("foreground")),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
        });

        stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N1)
                .items_center()
                .layout(LayoutRefinement::default()),
            move |_cx| vec![icon, text],
        )
    }
}

/// Commit message label aligned with AI Elements `CommitMessage`.
#[derive(Debug, Clone)]
pub struct CommitMessage {
    text: Arc<str>,
}

impl CommitMessage {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text: self.text,
            style: Some(TextStyle {
                font: FontId::default(),
                size: theme
                    .metric_by_key("component.commit.message_text_px")
                    .unwrap_or_else(|| theme.metric_required("font.size")),
                weight: FontWeight::MEDIUM,
                slant: Default::default(),
                line_height: Some(theme.metric_required("font.line_height")),
                letter_spacing_em: None,
            }),
            color: Some(theme.color_required("foreground")),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
        })
    }
}

/// Metadata row aligned with AI Elements `CommitMetadata`.
#[derive(Debug, Clone)]
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
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(self.layout)
                .gap(Space::N2)
                .items_center(),
            move |_cx| self.children,
        )
    }
}

/// Separator token aligned with AI Elements `CommitSeparator`.
#[derive(Debug, Clone)]
pub struct CommitSeparator {
    text: Arc<str>,
}

impl Default for CommitSeparator {
    fn default() -> Self {
        Self {
            text: Arc::<str>::from("•"),
        }
    }
}

impl CommitSeparator {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text: self.text,
            style: Some(TextStyle {
                font: FontId::default(),
                size: Px(12.0),
                weight: FontWeight::NORMAL,
                slant: Default::default(),
                line_height: Some(Px(16.0)),
                letter_spacing_em: None,
            }),
            color: Some(muted_fg(&theme)),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
        })
    }
}

/// Vertical info column aligned with AI Elements `CommitInfo`.
#[derive(Debug, Clone)]
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
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().flex_1().min_w_0())
                .gap(Space::N1),
            move |_cx| self.children,
        )
    }
}

/// Author row aligned with AI Elements `CommitAuthor`.
#[derive(Debug, Clone)]
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
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N2)
                .items_center()
                .layout(LayoutRefinement::default()),
            move |_cx| self.children,
        )
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
        fret_ui_shadcn::Avatar::new([fret_ui_shadcn::AvatarFallback::new(self.initials)
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
#[derive(Debug, Clone)]
pub struct CommitTimestamp {
    date: SystemTime,
}

impl CommitTimestamp {
    pub fn new(date: SystemTime) -> Self {
        Self { date }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let text = relative_days_label(self.date);
        cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text: Arc::<str>::from(text),
            style: Some(TextStyle {
                font: FontId::default(),
                size: Px(12.0),
                weight: FontWeight::NORMAL,
                slant: Default::default(),
                line_height: Some(Px(16.0)),
                letter_spacing_em: None,
            }),
            color: Some(muted_fg(&theme)),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
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
#[derive(Debug, Clone)]
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
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N1)
                .items_center()
                .layout(LayoutRefinement::default().flex_shrink_0()),
            move |_cx| self.children,
        )
    }
}

/// Copy button aligned with AI Elements `CommitCopyButton`.
#[derive(Clone)]
pub struct CommitCopyButton {
    hash: Arc<str>,
    on_copy: Option<
        Arc<dyn Fn(&mut dyn fret_ui::action::UiActionHost, fret_ui::action::ActionCx) + 'static>,
    >,
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
            timeout: Duration::from_millis(2000),
            test_id: None,
            copied_marker_test_id: None,
        }
    }

    /// Called after the copy intent is issued.
    ///
    /// Note: this callback does not currently model "copy failed" (platform effects are
    /// best-effort).
    pub fn on_copy(
        mut self,
        on_copy: Arc<
            dyn Fn(&mut dyn fret_ui::action::UiActionHost, fret_ui::action::ActionCx) + 'static,
        >,
    ) -> Self {
        self.on_copy = Some(on_copy);
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

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let feedback = cx.with_state(CopyFeedbackRef::default, |st| st.clone());

        let hash = self.hash;
        let on_copy = self.on_copy;
        let timeout = self.timeout;
        let test_id = self.test_id;
        let copied_marker_test_id = self.copied_marker_test_id;

        cx.pressable_with_id_props(move |cx, st, id| {
            let copied = feedback.lock().copied;
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
                        let mut feedback = feedback.lock();
                        if feedback.token != Some(token) {
                            return false;
                        }
                        feedback.token = None;
                        feedback.copied = false;
                        host.notify(action_cx);
                        host.request_redraw(action_cx.window);
                        true
                    }
                }),
            );

            cx.pressable_on_activate({
                let hash = hash.clone();
                let feedback = feedback.clone();
                let on_copy = on_copy.clone();
                Arc::new(move |host, action_cx, _reason| {
                    if feedback.lock().copied {
                        return;
                    }

                    host.push_effect(Effect::ClipboardSetText {
                        text: hash.to_string(),
                    });
                    if let Some(on_copy) = on_copy.as_ref() {
                        on_copy(host, action_cx);
                    }

                    let (prev, token) = {
                        let mut feedback = feedback.lock();
                        let prev = feedback.token.take();
                        let token = host.next_timer_token();
                        feedback.copied = true;
                        feedback.token = Some(token);
                        (prev, token)
                    };

                    if let Some(prev) = prev {
                        host.push_effect(Effect::CancelTimer { token: prev });
                    }
                    host.push_effect(Effect::SetTimer {
                        window: Some(action_cx.window),
                        token,
                        after: timeout,
                        repeat: None,
                    });
                    host.notify(action_cx);
                    host.request_redraw(action_cx.window);
                })
            });

            let mut pressable = PressableProps::default();
            pressable.enabled = true;
            pressable.focusable = true;
            pressable.a11y.role = Some(SemanticsRole::Button);
            pressable.a11y.label = Some(label);
            pressable.a11y.test_id = test_id.clone();

            let fg = theme.color_required("muted-foreground");
            let bg_hover = theme
                .color_by_key("color.menu.item.hover")
                .unwrap_or_else(|| theme.color_required("secondary"));
            let bg_pressed = theme
                .color_by_key("accent")
                .unwrap_or_else(|| theme.color_required("secondary"));

            let bg = if st.pressed {
                alpha(bg_pressed, 0.9)
            } else if st.hovered {
                alpha(bg_hover, 0.9)
            } else {
                Color::TRANSPARENT
            };

            let size = Px(28.0);
            let radius = theme.metric_required("metric.radius.sm");

            let icon_id = if copied {
                ids::ui::CHECK
            } else {
                ids::ui::COPY
            };
            let icon = decl_icon::icon_with(cx, icon_id, Some(Px(14.0)), Some(ColorRef::Color(fg)));

            let mut content_props = ContainerProps::default();
            content_props.layout.size.width = Length::Px(size);
            content_props.layout.size.height = Length::Px(size);
            content_props.layout.flex.shrink = 0.0;
            content_props.background = Some(bg);
            content_props.corner_radii = Corners::all(radius);
            content_props.border = Edges::all(Px(0.0));
            content_props.padding = Edges::all(Px(0.0));

            let content = cx.container(content_props, move |cx| {
                let row = stack::hstack(
                    cx,
                    stack::HStackProps::default()
                        .items_center()
                        .justify_center()
                        .layout(LayoutRefinement::default().w_full().h_full()),
                    move |_cx| vec![icon],
                );
                vec![row]
            });

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
                    })
                    .attach_semantics(
                        SemanticsDecoration::default()
                            .role(SemanticsRole::Generic)
                            .test_id(marker_id),
                    )
                })
            });

            let mut children = vec![content];
            if let Some(marker) = marker {
                children.push(marker);
            }
            (pressable, children)
        })
    }
}

/// Wrapper aligned with AI Elements `CommitFiles`.
#[derive(Debug, Clone)]
pub struct CommitFiles {
    children: Vec<AnyElement>,
}

impl CommitFiles {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
        }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .gap(Space::N1),
            move |_cx| self.children,
        )
    }
}

/// Wrapper aligned with AI Elements `CommitFile`.
#[derive(Debug, Clone)]
pub struct CommitFile {
    children: Vec<AnyElement>,
}

impl CommitFile {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
        }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let hover_bg = theme
            .color_by_key("muted")
            .map(|c| alpha(c, 0.5))
            .unwrap_or_else(|| alpha(theme.color_required("accent"), 0.2));

        let children = self.children;
        cx.hover_region(
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
                );
                props.background = bg;
                props.corner_radii = Corners::all(MetricRef::radius(Radius::Sm).resolve(&theme));

                let row = stack::hstack(
                    cx,
                    stack::HStackProps::default()
                        .layout(LayoutRefinement::default().w_full().min_w_0())
                        .gap(Space::N2)
                        .items_center()
                        .justify_between(),
                    move |_cx| children.clone(),
                );
                vec![cx.container(props, move |_cx| vec![row])]
            },
        )
    }
}

/// Wrapper aligned with AI Elements `CommitFileInfo`.
#[derive(Debug, Clone)]
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
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().min_w_0())
                .gap(Space::N2)
                .items_center(),
            move |_cx| self.children,
        )
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
#[derive(Debug, Clone)]
pub struct CommitFileStatus {
    status: CommitFileStatusKind,
}

impl CommitFileStatus {
    pub fn new(status: CommitFileStatusKind) -> Self {
        Self { status }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
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
            color: Some(status_color(self.status).resolve(&theme)),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
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
#[derive(Debug, Clone)]
pub struct CommitFilePath {
    path: Arc<str>,
}

impl CommitFilePath {
    pub fn new(path: impl Into<Arc<str>>) -> Self {
        Self { path: path.into() }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        cx.text_props(TextProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    min_width: Some(Px(0.0)),
                    ..Default::default()
                },
                ..Default::default()
            },
            text: self.path,
            style: Some(monospace_text_style(&theme, Px(12.0), FontWeight::NORMAL)),
            color: Some(theme.color_required("foreground")),
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
        })
    }
}

/// Wrapper aligned with AI Elements `CommitFileChanges`.
#[derive(Debug, Clone)]
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
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N1)
                .items_center()
                .layout(LayoutRefinement::default().flex_shrink_0()),
            move |_cx| self.children,
        )
    }
}

/// Additions count aligned with AI Elements `CommitFileAdditions`.
#[derive(Debug, Clone)]
pub struct CommitFileAdditions {
    count: u32,
}

impl CommitFileAdditions {
    pub fn new(count: u32) -> Self {
        Self { count }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        if self.count == 0 {
            return cx.text("");
        }

        let theme = Theme::global(&*cx.app).clone();
        let color = status_color(CommitFileStatusKind::Added).resolve(&theme);
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
        });

        stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N0)
                .items_center()
                .layout(LayoutRefinement::default()),
            move |_cx| vec![icon, text],
        )
    }
}

/// Deletions count aligned with AI Elements `CommitFileDeletions`.
#[derive(Debug, Clone)]
pub struct CommitFileDeletions {
    count: u32,
}

impl CommitFileDeletions {
    pub fn new(count: u32) -> Self {
        Self { count }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        if self.count == 0 {
            return cx.text("");
        }

        let theme = Theme::global(&*cx.app).clone();
        let color = status_color(CommitFileStatusKind::Deleted).resolve(&theme);
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
        });

        stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N0)
                .items_center()
                .layout(LayoutRefinement::default()),
            move |_cx| vec![icon, text],
        )
    }
}
