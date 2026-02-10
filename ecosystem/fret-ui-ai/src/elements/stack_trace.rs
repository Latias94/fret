//! AI Elements-aligned `StackTrace` surfaces.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use fret_core::{
    Color, Corners, Edges, FontId, FontWeight, Point, Px, SemanticsRole, TextOverflow, TextStyle,
    TextWrap, TimerToken, Transform2D,
};
use fret_icons::ids;
use fret_runtime::{Effect, Model};
use fret_ui::action::{ActionCx, UiActionHost};
use fret_ui::element::{
    AnyElement, ContainerProps, LayoutStyle, Length, PressableA11y, PressableProps,
    SemanticsDecoration, SemanticsProps, SizeStyle, TextProps, VisualTransformProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{
    ChromeRefinement, ColorFallback, ColorRef, Items, Justify, LayoutRefinement, MetricRef, Radius,
    Space,
};
use fret_ui_shadcn::{Collapsible, ScrollArea};

pub type OnStackTraceFilePathClick =
    Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, Arc<str>, Option<u32>, Option<u32>) + 'static>;

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

fn monospace_style(theme: &Theme, size: Px, weight: FontWeight) -> TextStyle {
    TextStyle {
        font: FontId::monospace(),
        size,
        weight,
        slant: Default::default(),
        line_height: Some(theme.metric_required("metric.font.mono_line_height")),
        letter_spacing_em: None,
    }
}

#[derive(Debug, Clone)]
pub struct StackFrame {
    pub raw: Arc<str>,
    pub function_name: Option<Arc<str>>,
    pub file_path: Option<Arc<str>>,
    pub line_number: Option<u32>,
    pub column_number: Option<u32>,
    pub is_internal: bool,
}

#[derive(Debug, Clone)]
pub struct ParsedStackTrace {
    pub error_type: Option<Arc<str>>,
    pub error_message: Arc<str>,
    pub frames: Arc<[StackFrame]>,
    pub raw: Arc<str>,
}

fn is_error_type(s: &str) -> bool {
    let s = s.trim();
    if s == "Error" {
        return true;
    }
    if !s.ends_with("Error") {
        return false;
    }
    s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn is_internal_path(path: &str) -> bool {
    path.contains("node_modules") || path.starts_with("node:") || path.contains("internal/")
}

fn parse_file_location(loc: &str) -> (Option<Arc<str>>, Option<u32>, Option<u32>) {
    let loc = loc.trim();
    let Some(last_colon) = loc.rfind(':') else {
        return (Some(Arc::<str>::from(loc)), None, None);
    };
    let (left, col_s) = loc.split_at(last_colon);
    let col_s = col_s.trim_start_matches(':');
    let column = col_s.parse::<u32>().ok();

    let Some(second_last_colon) = left.rfind(':') else {
        return (Some(Arc::<str>::from(loc)), None, column);
    };
    let (file_s, line_s) = left.split_at(second_last_colon);
    let line_s = line_s.trim_start_matches(':');
    let line = line_s.parse::<u32>().ok();

    (Some(Arc::<str>::from(file_s)), line, column)
}

fn parse_stack_frame(line: &str) -> StackFrame {
    let trimmed = line.trim();
    let without_at = trimmed.strip_prefix("at ").unwrap_or(trimmed).trim();

    if let (Some(open), true) = (without_at.rfind(" ("), without_at.ends_with(')')) {
        let function_name = without_at[..open].trim();
        let loc = &without_at[(open + 2)..(without_at.len() - 1)];
        let (file_path, line_number, column_number) = parse_file_location(loc);
        let is_internal = file_path
            .as_deref()
            .map(is_internal_path)
            .unwrap_or_else(|| is_internal_path(trimmed));
        return StackFrame {
            raw: Arc::<str>::from(trimmed),
            function_name: (!function_name.is_empty()).then(|| Arc::<str>::from(function_name)),
            file_path,
            line_number,
            column_number,
            is_internal,
        };
    }

    let (file_path, line_number, column_number) = parse_file_location(without_at);
    let is_internal = file_path
        .as_deref()
        .map(is_internal_path)
        .unwrap_or_else(|| is_internal_path(trimmed));
    StackFrame {
        raw: Arc::<str>::from(trimmed),
        function_name: None,
        file_path,
        line_number,
        column_number,
        is_internal,
    }
}

pub fn parse_stack_trace(raw: impl Into<Arc<str>>) -> ParsedStackTrace {
    let raw: Arc<str> = raw.into();
    let mut lines: Vec<&str> = raw
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();

    if lines.is_empty() {
        return ParsedStackTrace {
            error_type: None,
            error_message: raw.clone(),
            frames: Arc::from([]),
            raw,
        };
    }

    let first = lines[0];
    let (error_type, error_message) = first
        .split_once(':')
        .map(|(ty, msg)| {
            let ty = ty.trim();
            let msg = msg.trim();
            if is_error_type(ty) {
                (
                    Some(Arc::<str>::from(ty)),
                    Arc::<str>::from(msg.to_string()),
                )
            } else {
                (None, Arc::<str>::from(first))
            }
        })
        .unwrap_or_else(|| (None, Arc::<str>::from(first)));

    let mut frames: Vec<StackFrame> = Vec::new();
    for line in lines.drain(1..) {
        let trimmed = line.trim();
        if !trimmed.starts_with("at ") {
            continue;
        }
        frames.push(parse_stack_frame(trimmed));
    }

    ParsedStackTrace {
        error_type,
        error_message,
        frames: Arc::from(frames),
        raw,
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

/// Copy button aligned with AI Elements `StackTraceCopyButton`.
#[derive(Clone)]
pub struct StackTraceCopyButton {
    raw: Arc<str>,
    on_copy: Option<
        Arc<dyn Fn(&mut dyn fret_ui::action::UiActionHost, fret_ui::action::ActionCx) + 'static>,
    >,
    timeout: Duration,
    test_id: Option<Arc<str>>,
    copied_marker_test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for StackTraceCopyButton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StackTraceCopyButton")
            .field("raw_len", &self.raw.len())
            .field("timeout_ms", &self.timeout.as_millis())
            .field("test_id", &self.test_id.as_deref())
            .field(
                "copied_marker_test_id",
                &self.copied_marker_test_id.as_deref(),
            )
            .finish()
    }
}

impl StackTraceCopyButton {
    pub fn new(raw: impl Into<Arc<str>>) -> Self {
        Self {
            raw: raw.into(),
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

        let raw = self.raw;
        let on_copy = self.on_copy;
        let timeout = self.timeout;
        let test_id = self.test_id;
        let copied_marker_test_id = self.copied_marker_test_id;

        cx.pressable_with_id_props(move |cx, st, id| {
            let copied = feedback.lock().copied;
            let label: Arc<str> = if copied {
                Arc::<str>::from("Copied")
            } else {
                Arc::<str>::from("Copy stack trace")
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
                let raw = raw.clone();
                let feedback = feedback.clone();
                let on_copy = on_copy.clone();
                Arc::new(move |host, action_cx, _reason| {
                    host.push_effect(Effect::ClipboardSetText {
                        text: raw.to_string(),
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
                .color_by_key("muted")
                .unwrap_or_else(|| theme.color_required("accent"));
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

#[derive(Clone)]
pub struct StackTraceFrames {
    frames: Arc<[StackFrame]>,
    show_internal_frames: bool,
    on_file_path_click: Option<OnStackTraceFilePathClick>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl std::fmt::Debug for StackTraceFrames {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StackTraceFrames")
            .field("frames_len", &self.frames.len())
            .field("show_internal_frames", &self.show_internal_frames)
            .field("has_on_file_path_click", &self.on_file_path_click.is_some())
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .field("chrome", &self.chrome)
            .finish()
    }
}

impl StackTraceFrames {
    pub fn new(frames: impl Into<Arc<[StackFrame]>>) -> Self {
        Self {
            frames: frames.into(),
            show_internal_frames: true,
            on_file_path_click: None,
            test_id: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default().p(Space::N3),
        }
    }

    pub fn show_internal_frames(mut self, show: bool) -> Self {
        self.show_internal_frames = show;
        self
    }

    pub fn on_file_path_click(mut self, on_click: OnStackTraceFilePathClick) -> Self {
        self.on_file_path_click = Some(on_click);
        self
    }

    pub fn test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
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
        let theme = Theme::global(&*cx.app).clone();

        let frames: Vec<StackFrame> = if self.show_internal_frames {
            self.frames.to_vec()
        } else {
            self.frames
                .iter()
                .filter(|f| !f.is_internal)
                .cloned()
                .collect()
        };

        let on_file_path_click = self.on_file_path_click;
        let text_px = theme
            .metric_by_key("fret.ai.stack_trace.frames.text_px")
            .or_else(|| theme.metric_by_key("component.code_block.text_px"))
            .unwrap_or(Px(11.0));
        let style = monospace_style(&theme, text_px, FontWeight::NORMAL);
        let fg_muted = muted_fg(&theme);
        let fg_normal = alpha(theme.color_required("foreground"), 0.90);
        let fg_internal = alpha(fg_muted, 0.50);
        let fg_primary = theme
            .color_by_key("primary")
            .unwrap_or_else(|| theme.color_required("foreground"));

        let rows: Vec<AnyElement> = if frames.is_empty() {
            vec![cx.text_props(TextProps {
                layout: LayoutStyle::default(),
                text: Arc::<str>::from("No stack frames"),
                style: Some(style.clone()),
                color: Some(fg_muted),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
            })]
        } else {
            frames
                .into_iter()
                .enumerate()
                .map(|(index, frame)| {
                    let style = style.clone();
                    let fg_primary = fg_primary;
                    let on_file_path_click = on_file_path_click.clone();
                    cx.keyed(format!("stack-frame-{index}"), move |cx| {
                        let fg = if frame.is_internal {
                            fg_internal
                        } else {
                            fg_normal
                        };

                        let label_at = cx.text_props(TextProps {
                            layout: LayoutStyle::default(),
                            text: Arc::<str>::from("at "),
                            style: Some(style.clone()),
                            color: Some(fg_muted),
                            wrap: TextWrap::None,
                            overflow: TextOverflow::Clip,
                        });

                        let mut parts: Vec<AnyElement> = vec![label_at];

                        if let Some(function_name) = frame.function_name.clone() {
                            parts.push(cx.text_props(TextProps {
                                layout: LayoutStyle::default(),
                                text: Arc::<str>::from(format!("{function_name} ")),
                                style: Some(style.clone()),
                                color: Some(fg),
                                wrap: TextWrap::None,
                                overflow: TextOverflow::Clip,
                            }));
                        }

                        if let Some(file_path) = frame.file_path.clone() {
                            parts.push(cx.text_props(TextProps {
                                layout: LayoutStyle::default(),
                                text: Arc::<str>::from("("),
                                style: Some(style.clone()),
                                color: Some(fg_muted),
                                wrap: TextWrap::None,
                                overflow: TextOverflow::Clip,
                            }));

                            let label = {
                                let mut s = file_path.to_string();
                                if let Some(line) = frame.line_number {
                                    s.push(':');
                                    s.push_str(&line.to_string());
                                }
                                if let Some(col) = frame.column_number {
                                    s.push(':');
                                    s.push_str(&col.to_string());
                                }
                                Arc::<str>::from(s)
                            };

                            let enabled = on_file_path_click.is_some();
                            let mut pressable = PressableProps::default();
                            pressable.enabled = enabled;
                            pressable.focusable = enabled;
                            pressable.a11y = PressableA11y {
                                role: Some(SemanticsRole::Button),
                                label: Some(Arc::<str>::from("Open stack frame location")),
                                ..Default::default()
                            };

                            let click_payload = on_file_path_click.clone();
                            let file_path_payload = file_path.clone();
                            let line_payload = frame.line_number;
                            let col_payload = frame.column_number;
                            let style_for_button = style.clone();
                            let button = cx.pressable(pressable, move |cx, _st| {
                                if let Some(click) = click_payload.clone() {
                                    cx.pressable_on_activate(Arc::new(
                                        move |host, action_cx, _| {
                                            click(
                                                host,
                                                action_cx,
                                                file_path_payload.clone(),
                                                line_payload,
                                                col_payload,
                                            );
                                        },
                                    ));
                                }

                                vec![cx.text_props(TextProps {
                                    layout: LayoutStyle::default(),
                                    text: label.clone(),
                                    style: Some(style_for_button.clone()),
                                    color: Some(if enabled { fg_primary } else { fg }),
                                    wrap: TextWrap::None,
                                    overflow: TextOverflow::Clip,
                                })]
                            });

                            parts.push(button);

                            parts.push(cx.text_props(TextProps {
                                layout: LayoutStyle::default(),
                                text: Arc::<str>::from(")"),
                                style: Some(style.clone()),
                                color: Some(fg_muted),
                                wrap: TextWrap::None,
                                overflow: TextOverflow::Clip,
                            }));
                        }

                        if frame.file_path.is_none() && frame.function_name.is_none() {
                            let raw = frame.raw.trim().strip_prefix("at ").unwrap_or(&frame.raw);
                            parts.push(cx.text_props(TextProps {
                                layout: LayoutStyle::default(),
                                text: Arc::<str>::from(raw),
                                style: Some(style.clone()),
                                color: Some(fg),
                                wrap: TextWrap::None,
                                overflow: TextOverflow::Clip,
                            }));
                        }

                        stack::hstack(
                            cx,
                            stack::HStackProps::default()
                                .layout(LayoutRefinement::default().w_full().min_w_0())
                                .gap(Space::N0)
                                .items(Items::Center),
                            move |_cx| parts,
                        )
                    })
                })
                .collect()
        };

        let props = decl_style::container_props(&theme, self.chrome, self.layout);
        let list = cx.container(props, move |cx| {
            vec![stack::vstack(
                cx,
                stack::VStackProps::default()
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .gap(Space::N1),
                move |_cx| rows,
            )]
        });

        let Some(test_id) = self.test_id else {
            return list;
        };
        cx.semantics(
            SemanticsProps {
                role: SemanticsRole::Group,
                test_id: Some(test_id),
                ..Default::default()
            },
            move |_cx| vec![list],
        )
    }
}

/// Stack trace disclosure surface aligned with AI Elements `stack-trace.tsx`.
#[derive(Clone)]
pub struct StackTrace {
    trace: Arc<str>,
    open: Option<Model<bool>>,
    default_open: bool,
    show_internal_frames: bool,
    max_height: Px,
    on_file_path_click: Option<OnStackTraceFilePathClick>,
    test_id_root: Option<Arc<str>>,
    test_id_header_trigger: Option<Arc<str>>,
    test_id_copy_button: Option<Arc<str>>,
    test_id_copy_copied_marker: Option<Arc<str>>,
    test_id_frames: Option<Arc<str>>,
    test_id_content: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl std::fmt::Debug for StackTrace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StackTrace")
            .field("trace_len", &self.trace.len())
            .field("open", &"<model>")
            .field("default_open", &self.default_open)
            .field("show_internal_frames", &self.show_internal_frames)
            .field("max_height", &self.max_height)
            .field("has_on_file_path_click", &self.on_file_path_click.is_some())
            .field("test_id_root", &self.test_id_root.as_deref())
            .field(
                "test_id_header_trigger",
                &self.test_id_header_trigger.as_deref(),
            )
            .field("test_id_copy_button", &self.test_id_copy_button.as_deref())
            .field(
                "test_id_copy_copied_marker",
                &self.test_id_copy_copied_marker.as_deref(),
            )
            .field("test_id_frames", &self.test_id_frames.as_deref())
            .field("test_id_content", &self.test_id_content.as_deref())
            .field("layout", &self.layout)
            .field("chrome", &self.chrome)
            .finish()
    }
}

impl StackTrace {
    pub fn new(trace: impl Into<Arc<str>>) -> Self {
        Self {
            trace: trace.into(),
            open: None,
            default_open: false,
            show_internal_frames: true,
            max_height: Px(400.0),
            on_file_path_click: None,
            test_id_root: None,
            test_id_header_trigger: None,
            test_id_copy_button: None,
            test_id_copy_copied_marker: None,
            test_id_frames: None,
            test_id_content: None,
            layout: LayoutRefinement::default()
                .w_full()
                .min_w_0()
                .overflow_hidden(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn open(mut self, open: Model<bool>) -> Self {
        self.open = Some(open);
        self
    }

    pub fn default_open(mut self, default_open: bool) -> Self {
        self.default_open = default_open;
        self
    }

    pub fn show_internal_frames(mut self, show_internal_frames: bool) -> Self {
        self.show_internal_frames = show_internal_frames;
        self
    }

    pub fn max_height(mut self, max_height: Px) -> Self {
        self.max_height = Px(max_height.0.max(0.0));
        self
    }

    pub fn on_file_path_click(mut self, on_click: OnStackTraceFilePathClick) -> Self {
        self.on_file_path_click = Some(on_click);
        self
    }

    pub fn test_id_root(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id_root = Some(test_id.into());
        self
    }

    pub fn test_id_header_trigger(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id_header_trigger = Some(test_id.into());
        self
    }

    pub fn test_id_copy_button(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id_copy_button = Some(test_id.into());
        self
    }

    pub fn test_id_copy_copied_marker(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id_copy_copied_marker = Some(test_id.into());
        self
    }

    pub fn test_id_content(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id_content = Some(test_id.into());
        self
    }

    pub fn test_id_frames(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id_frames = Some(test_id.into());
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
        let theme = Theme::global(&*cx.app).clone();
        let theme_header = theme.clone();
        let theme_content = theme.clone();

        let trace_raw = self.trace.clone();
        let parsed = parse_stack_trace(self.trace);

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

        let chrome = base_chrome.merge(self.chrome);
        let layout = LayoutRefinement::default()
            .w_full()
            .min_w_0()
            .merge(self.layout);

        let error_fg = theme
            .color_by_key("destructive")
            .unwrap_or_else(|| theme.color_required("foreground"));
        let text_px = theme
            .metric_by_key("fret.ai.stack_trace.header.text_px")
            .or_else(|| theme.metric_by_key("component.code_block.text_px"))
            .unwrap_or(Px(12.0));
        let style_normal = monospace_style(&theme, text_px, FontWeight::NORMAL);
        let style_bold = monospace_style(&theme, text_px, FontWeight::SEMIBOLD);
        let header_fg = theme.color_required("foreground");
        let chevron_fg = muted_fg(&theme);
        let content_border = theme.color_required("border");
        let content_bg = theme
            .color_by_key("muted")
            .map(|c| alpha(c, 0.30))
            .unwrap_or_else(|| alpha(theme.color_required("accent"), 0.18));

        let error_type = parsed
            .error_type
            .clone()
            .unwrap_or_else(|| Arc::<str>::from("Error"));
        let error_message = parsed.error_message.clone();

        let on_file_path_click = self.on_file_path_click;
        let show_internal_frames = self.show_internal_frames;
        let max_height = self.max_height;
        let test_id_header_trigger = self.test_id_header_trigger;
        let test_id_copy_button = self.test_id_copy_button;
        let test_id_copy_copied_marker = self.test_id_copy_copied_marker;
        let test_id_content = self.test_id_content;
        let test_id_frames = self.test_id_frames;

        let collapsible = if let Some(open) = self.open {
            Collapsible::new(open)
        } else {
            Collapsible::uncontrolled(self.default_open)
        };

        let root = collapsible
            .refine_layout(layout)
            .refine_style(chrome)
            .into_element_with_open_model(
                cx,
                move |cx, open_model, is_open| {
                    let icon = decl_icon::icon_with(
                        cx,
                        ids::ui::ALERT_TRIANGLE,
                        Some(Px(16.0)),
                        Some(ColorRef::Color(error_fg)),
                    );

                    let ty = cx.text_props(TextProps {
                        layout: LayoutStyle::default(),
                        text: error_type.clone(),
                        style: Some(style_bold),
                        color: Some(error_fg),
                        wrap: TextWrap::None,
                        overflow: TextOverflow::Clip,
                    });

                    let msg = cx.text_props(TextProps {
                        layout: LayoutStyle {
                            size: SizeStyle {
                                width: Length::Fill,
                                height: Length::Auto,
                                min_width: Some(Px(0.0)),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        text: error_message.clone(),
                        style: Some(style_normal.clone()),
                        color: Some(header_fg),
                        wrap: TextWrap::None,
                        overflow: TextOverflow::Ellipsis,
                    });

                    let error_row = stack::hstack(
                        cx,
                        stack::HStackProps::default()
                            .layout(LayoutRefinement::default().flex_1().min_w_0())
                            .gap(Space::N2)
                            .items_center(),
                        move |_cx| vec![icon, ty, msg],
                    );

                    let mut copy = StackTraceCopyButton::new(trace_raw.clone());
                    if let Some(test_id) = test_id_copy_button.clone() {
                        copy = copy.test_id(test_id);
                    }
                    if let Some(test_id) = test_id_copy_copied_marker.clone() {
                        copy = copy.copied_marker_test_id(test_id);
                    }
                    let copy = copy.into_element(cx);

                    let chevron_size = Px(28.0);
                    let chevron_icon_size = Px(16.0);
                    let center = Point::new(Px(8.0), Px(8.0));
                    let rotation = if is_open { 180.0 } else { 0.0 };
                    let chevron_transform = Transform2D::rotation_about_degrees(rotation, center);
                    let chevron = cx.visual_transform_props(
                        VisualTransformProps {
                            layout: decl_style::layout_style(
                                &theme_header,
                                LayoutRefinement::default()
                                    .w_px(MetricRef::Px(chevron_size))
                                    .h_px(MetricRef::Px(chevron_size))
                                    .flex_shrink_0(),
                            ),
                            transform: chevron_transform,
                        },
                        move |cx| {
                            vec![decl_icon::icon_with(
                                cx,
                                ids::ui::CHEVRON_DOWN,
                                Some(chevron_icon_size),
                                Some(ColorRef::Color(chevron_fg)),
                            )]
                        },
                    );

                    let actions = stack::hstack(
                        cx,
                        stack::HStackProps::default()
                            .layout(LayoutRefinement::default().flex_shrink_0())
                            .gap(Space::N1)
                            .items_center(),
                        move |_cx| vec![copy, chevron],
                    );

                    let row = stack::hstack(
                        cx,
                        stack::HStackProps::default()
                            .layout(LayoutRefinement::default().w_full().min_w_0())
                            .gap(Space::N3)
                            .justify(Justify::Between)
                            .items_center(),
                        move |_cx| vec![error_row, actions],
                    );

                    let header = cx.container(
                        decl_style::container_props(
                            &theme_header,
                            ChromeRefinement::default().p(Space::N3),
                            LayoutRefinement::default().w_full().min_w_0(),
                        ),
                        move |_cx| vec![row],
                    );

                    let trigger = fret_ui_shadcn::CollapsibleTrigger::new(open_model, vec![header])
                        .a11y_label("Toggle stack trace details")
                        .into_element(cx, is_open);

                    let Some(test_id) = test_id_header_trigger.clone() else {
                        return trigger;
                    };
                    trigger.attach_semantics(
                        SemanticsDecoration::default()
                            .role(SemanticsRole::Button)
                            .test_id(test_id),
                    )
                },
                move |cx| {
                    let mut frames = StackTraceFrames::new(parsed.frames.clone())
                        .show_internal_frames(show_internal_frames);
                    if let Some(on_file_path_click) = on_file_path_click.clone() {
                        frames = frames.on_file_path_click(on_file_path_click);
                    }
                    if let Some(test_id) = test_id_frames.clone() {
                        frames = frames.test_id(test_id);
                    }
                    let frames = frames.into_element(cx);

                    let scroll = ScrollArea::new([frames])
                        .refine_layout(
                            LayoutRefinement::default()
                                .w_full()
                                .min_w_0()
                                .max_h(MetricRef::Px(max_height)),
                        )
                        .into_element(cx);

                    let mut content_props = ContainerProps::default();
                    content_props.layout = decl_style::layout_style(
                        &theme_content,
                        LayoutRefinement::default().w_full(),
                    );
                    content_props.background = Some(content_bg);
                    content_props.border = Edges {
                        top: Px(1.0),
                        right: Px(0.0),
                        bottom: Px(0.0),
                        left: Px(0.0),
                    };
                    content_props.border_color = Some(content_border);

                    let content = cx.container(content_props, move |_cx| vec![scroll]);

                    let content = if let Some(test_id) = test_id_content.clone() {
                        content.attach_semantics(
                            SemanticsDecoration::default()
                                .role(SemanticsRole::Group)
                                .test_id(test_id),
                        )
                    } else {
                        content
                    };

                    fret_ui_shadcn::CollapsibleContent::new([content]).into_element(cx)
                },
            );

        let Some(test_id) = self.test_id_root else {
            return root;
        };
        root.attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Group)
                .test_id(test_id),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_error_type_and_message() {
        let parsed = parse_stack_trace("TypeError: Boom\nat foo (a.js:1:2)");
        assert_eq!(parsed.error_type.as_deref(), Some("TypeError"));
        assert_eq!(parsed.error_message.as_ref(), "Boom");
        assert_eq!(parsed.frames.len(), 1);
    }

    #[test]
    fn parses_frame_with_function_and_location() {
        let parsed = parse_stack_trace("Error: X\nat foo (/a/b/c.js:10:20)");
        let f = &parsed.frames[0];
        assert_eq!(f.function_name.as_deref(), Some("foo"));
        assert_eq!(f.file_path.as_deref(), Some("/a/b/c.js"));
        assert_eq!(f.line_number, Some(10));
        assert_eq!(f.column_number, Some(20));
    }

    #[test]
    fn parses_frame_without_function() {
        let parsed = parse_stack_trace("Error: X\nat /a/b/c.js:10:20");
        let f = &parsed.frames[0];
        assert_eq!(f.function_name.as_deref(), None);
        assert_eq!(f.file_path.as_deref(), Some("/a/b/c.js"));
        assert_eq!(f.line_number, Some(10));
        assert_eq!(f.column_number, Some(20));
    }
}
