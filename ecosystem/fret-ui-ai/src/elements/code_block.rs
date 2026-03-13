use std::sync::{Arc, Mutex};
use std::time::Duration;

use fret_core::{
    Color, Edges, FontId, FontWeight, Px, SemanticsRole, TextOverflow, TextStyle, TextWrap,
    TimerToken,
};
use fret_runtime::Effect;
use fret_ui::element::{
    AnyElement, ContainerProps, LayoutStyle, Length, Overflow, PressableProps, TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::chrome::centered_fixed_chrome_pressable_with_id_props;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::typography;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, Items, Justify, LayoutRefinement, Radius, Space, ui,
};

/// Nearest `CodeBlock` context in scope.
#[derive(Debug, Clone)]
pub struct CodeBlockContext {
    pub code: Arc<str>,
    pub language: Option<Arc<str>>,
    pub show_line_numbers: bool,
}

#[derive(Debug, Default, Clone)]
struct CodeBlockLocalState {
    context: Option<CodeBlockContext>,
}

pub fn use_code_block_context<H: UiHost>(cx: &ElementContext<'_, H>) -> Option<CodeBlockContext> {
    cx.inherited_state::<CodeBlockLocalState>()
        .and_then(|st| st.context.clone())
}

/// AI Elements-aligned code block surface backed by `ecosystem/fret-code-view`.
///
/// This is a policy/composition layer:
/// - apps own effects (except for local clipboard copy),
/// - the component exposes stable selectors for diag automation.
pub struct CodeBlock {
    code: Arc<str>,
    language: Option<Arc<str>>,
    show_header: bool,
    show_line_numbers: bool,
    show_language: bool,
    max_height: Option<Px>,
    windowed_lines: bool,
    children: Vec<AnyElement>,
    header_left: Vec<AnyElement>,
    header_right: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for CodeBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CodeBlock")
            .field("code_len", &self.code.len())
            .field("language", &self.language.as_deref())
            .field("show_header", &self.show_header)
            .field("show_line_numbers", &self.show_line_numbers)
            .field("show_language", &self.show_language)
            .field("max_height", &self.max_height)
            .field("windowed_lines", &self.windowed_lines)
            .field("children_len", &self.children.len())
            .field("header_left_len", &self.header_left.len())
            .field("header_right_len", &self.header_right.len())
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .finish()
    }
}

impl CodeBlock {
    pub fn new(code: impl Into<Arc<str>>) -> Self {
        Self {
            code: code.into(),
            language: None,
            show_header: false,
            show_line_numbers: false,
            show_language: false,
            max_height: None,
            windowed_lines: false,
            children: Vec::new(),
            header_left: Vec::new(),
            header_right: Vec::new(),
            test_id: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
        }
    }

    pub fn language(mut self, language: impl Into<Arc<str>>) -> Self {
        self.language = Some(language.into());
        self
    }

    /// Shows a header row (language label + optional actions) above the code content.
    ///
    /// AI Elements' `CodeBlock` does not render a header by default unless the caller supplies
    /// header children; we model this explicitly so wrappers like `ToolInput` match upstream.
    pub fn show_header(mut self, show: bool) -> Self {
        self.show_header = show;
        self
    }

    pub fn show_line_numbers(mut self, show: bool) -> Self {
        self.show_line_numbers = show;
        self
    }

    pub fn show_language(mut self, show: bool) -> Self {
        self.show_language = show;
        self
    }

    pub fn max_height(mut self, max_height: Px) -> Self {
        self.max_height = Some(Px(max_height.0.max(0.0)));
        self
    }

    /// Enables a virtualized/windowed vertical list for long snippets.
    ///
    /// This keeps the UI gallery responsive for large examples while preserving shadcn-aligned
    /// scrollbar + max-height behavior.
    pub fn windowed_lines(mut self, enabled: bool) -> Self {
        self.windowed_lines = enabled;
        self
    }

    pub fn header_left<I>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.header_left.extend(children);
        self
    }

    pub fn header_right<I>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = AnyElement>,
    {
        self.header_right.extend(children);
        self
    }

    pub fn test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element_with_children<H: UiHost + 'static>(
        self,
        cx: &mut ElementContext<'_, H>,
        children: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
    ) -> AnyElement {
        let context = CodeBlockContext {
            code: self.code.clone(),
            language: self.language.clone(),
            show_line_numbers: self.show_line_numbers,
        };
        cx.root_state(CodeBlockLocalState::default, |st| {
            st.context = Some(context);
        });
        self.children(children(cx)).into_element(cx)
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        if !self.children.is_empty() {
            return self.into_element_composable(cx);
        }

        let theme = Theme::global(&*cx.app).clone();

        let mut header =
            fret_code_view::CodeBlockHeaderSlots::default().show_language(self.show_language);
        header.left = self.header_left;
        header.right = self.header_right;

        let header_visible = self.show_header
            || !header.left.is_empty()
            || !header.right.is_empty()
            || (self.show_language && self.language.is_some());

        let options = fret_code_view::CodeBlockUiOptions {
            show_header: self.show_header,
            header_divider: header_visible,
            header_background: if header_visible {
                fret_code_view::CodeBlockHeaderBackground::Muted80
            } else {
                fret_code_view::CodeBlockHeaderBackground::None
            },
            show_copy_button: false,
            copy_button_on_hover: true,
            copy_button_placement: fret_code_view::CodeBlockCopyButtonPlacement::Overlay,
            border: true,
            wrap: fret_code_view::CodeBlockWrap::ScrollX,
            disable_ligatures: true,
            disable_contextual_alternates: true,
            max_height: self.max_height,
            windowed_lines: false,
            windowed_lines_overscan: 6,
            show_scrollbar_x: true,
            scrollbar_x_on_hover: true,
            show_scrollbar_y: true,
            scrollbar_y_on_hover: true,
        };
        let options = fret_code_view::CodeBlockUiOptions {
            windowed_lines: self.windowed_lines,
            ..options
        };

        let code = self.code;
        let language = self.language;
        let show_line_numbers = self.show_line_numbers;
        let content = fret_code_view::code_block_with_header_slots(
            cx,
            &code,
            language.as_deref(),
            show_line_numbers,
            options,
            header,
        );

        let el = cx.container(
            ContainerProps {
                layout: decl_style::layout_style(&theme, self.layout),
                ..Default::default()
            },
            move |_cx| vec![content],
        );

        let Some(test_id) = self.test_id else {
            return el;
        };

        el.attach_semantics(
            fret_ui::element::SemanticsDecoration::default()
                .role(SemanticsRole::Group)
                .test_id(test_id),
        )
    }

    fn into_element_composable<H: UiHost + 'static>(
        self,
        cx: &mut ElementContext<'_, H>,
    ) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let mut chrome = ChromeRefinement::default().rounded(Radius::Md);
        chrome = chrome
            .border_1()
            .bg(ColorRef::Color(theme.color_token("card")))
            .border_color(ColorRef::Color(theme.color_token("border")));

        let mut props = decl_style::container_props(&theme, chrome, self.layout);
        props.layout.overflow = Overflow::Clip;

        let content = fret_code_view::code_block_with_header_slots(
            cx,
            &self.code,
            self.language.as_deref(),
            self.show_line_numbers,
            fret_code_view::CodeBlockUiOptions {
                show_header: false,
                header_divider: false,
                header_background: fret_code_view::CodeBlockHeaderBackground::None,
                show_copy_button: false,
                copy_button_on_hover: true,
                copy_button_placement: fret_code_view::CodeBlockCopyButtonPlacement::Overlay,
                border: false,
                wrap: fret_code_view::CodeBlockWrap::ScrollX,
                disable_ligatures: true,
                disable_contextual_alternates: true,
                max_height: self.max_height,
                windowed_lines: self.windowed_lines,
                windowed_lines_overscan: 6,
                show_scrollbar_x: true,
                scrollbar_x_on_hover: true,
                show_scrollbar_y: true,
                scrollbar_y_on_hover: true,
            },
            fret_code_view::CodeBlockHeaderSlots::default(),
        );

        let children = self.children;
        let el = cx.container(props, move |cx| {
            vec![
                ui::v_flex(move |_cx| {
                    let mut out = children;
                    out.push(content);
                    out
                })
                .gap(Space::N0)
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .into_element(cx),
            ]
        });

        let Some(test_id) = self.test_id else {
            return el;
        };

        el.attach_semantics(
            fret_ui::element::SemanticsDecoration::default()
                .role(SemanticsRole::Group)
                .test_id(test_id),
        )
    }
}

/// Header row aligned with AI Elements `CodeBlockHeader`.
#[derive(Debug)]
pub struct CodeBlockHeader {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
}

impl CodeBlockHeader {
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
        let pad_x = fret_ui_kit::MetricRef::space(Space::N3).resolve(&theme);
        let pad_y = fret_ui_kit::MetricRef::space(Space::N2).resolve(&theme);
        let muted = theme.color_token("muted");
        let children = self.children;

        let el = cx.container(
            ContainerProps {
                layout: LayoutStyle {
                    size: fret_ui::element::SizeStyle {
                        width: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                padding: Edges::symmetric(pad_x, pad_y).into(),
                background: Some(Color { a: 0.8, ..muted }),
                border: Edges {
                    top: Px(0.0),
                    right: Px(0.0),
                    bottom: Px(1.0),
                    left: Px(0.0),
                },
                border_color: Some(theme.color_token("border")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    ui::h_flex(move |_cx| children)
                        .gap(Space::N2)
                        .justify(Justify::Between)
                        .items(Items::Center)
                        .layout(LayoutRefinement::default().w_full().min_w_0())
                        .into_element(cx),
                ]
            },
        );

        let Some(test_id) = self.test_id else {
            return el;
        };

        el.attach_semantics(
            fret_ui::element::SemanticsDecoration::default()
                .role(SemanticsRole::Group)
                .test_id(test_id),
        )
    }
}

/// Left-aligned title row aligned with AI Elements `CodeBlockTitle`.
#[derive(Debug)]
pub struct CodeBlockTitle {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
}

impl CodeBlockTitle {
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
        let children = self.children;
        let el = ui::h_row(move |_cx| children)
            .gap(Space::N2)
            .items_center()
            .layout(LayoutRefinement::default().min_w_0())
            .into_element(cx);

        let Some(test_id) = self.test_id else {
            return el;
        };

        el.attach_semantics(
            fret_ui::element::SemanticsDecoration::default()
                .role(SemanticsRole::Group)
                .test_id(test_id),
        )
    }
}

/// Right-aligned actions row aligned with AI Elements `CodeBlockActions`.
#[derive(Debug)]
pub struct CodeBlockActions {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
}

impl CodeBlockActions {
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
        let children = self.children;
        let el = ui::h_row(move |_cx| children)
            .gap(Space::N2)
            .items_center()
            .into_element(cx);

        let Some(test_id) = self.test_id else {
            return el;
        };

        el.attach_semantics(
            fret_ui::element::SemanticsDecoration::default()
                .role(SemanticsRole::Group)
                .test_id(test_id),
        )
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

fn alpha(color: Color, a: f32) -> Color {
    Color {
        r: color.r,
        g: color.g,
        b: color.b,
        a: (color.a * a).clamp(0.0, 1.0),
    }
}

/// Copy button aligned with AI Elements `CodeBlockCopyButton`.
#[derive(Clone)]
pub struct CodeBlockCopyButton {
    code: Option<Arc<str>>,
    on_copy: Option<
        Arc<dyn Fn(&mut dyn fret_ui::action::UiActionHost, fret_ui::action::ActionCx) + 'static>,
    >,
    timeout: Duration,
    test_id: Option<Arc<str>>,
    copied_marker_test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for CodeBlockCopyButton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CodeBlockCopyButton")
            .field("code_len", &self.code.as_ref().map(|code| code.len()))
            .field("timeout_ms", &self.timeout.as_millis())
            .field("test_id", &self.test_id.as_deref())
            .field(
                "copied_marker_test_id",
                &self.copied_marker_test_id.as_deref(),
            )
            .finish()
    }
}

impl CodeBlockCopyButton {
    pub fn new(code: impl Into<Arc<str>>) -> Self {
        Self {
            code: Some(code.into()),
            on_copy: None,
            timeout: Duration::from_millis(2000),
            test_id: None,
            copied_marker_test_id: None,
        }
    }

    pub fn from_context() -> Self {
        Self {
            code: None,
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
        let feedback = cx.slot_state(CopyFeedbackRef::default, |st| st.clone());

        let code = self
            .code
            .or_else(|| use_code_block_context(cx).map(|context| context.code))
            .unwrap_or_else(|| Arc::<str>::from(""));
        let on_copy = self.on_copy;
        let timeout = self.timeout;
        let test_id = self.test_id;
        let copied_marker_test_id = self.copied_marker_test_id;

        centered_fixed_chrome_pressable_with_id_props(cx, move |cx, st, id| {
            let copied = feedback.lock().copied;
            let label = if copied { "Copied" } else { "Copy" };

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
                let code = code.clone();
                let feedback = feedback.clone();
                let on_copy = on_copy.clone();
                Arc::new(move |host, action_cx, _reason| {
                    if feedback.lock().copied {
                        return;
                    }

                    host.push_effect(Effect::ClipboardSetText {
                        text: code.to_string(),
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

            let mut props = PressableProps::default();
            props.enabled = true;
            props.focusable = true;
            props.a11y.role = Some(SemanticsRole::Button);
            props.a11y.label = Some(Arc::<str>::from(label));
            props.a11y.test_id = test_id.clone();

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
            props.layout.size.width = Length::Px(size);
            props.layout.size.height = Length::Px(size);
            props.layout.flex.shrink = 0.0;

            let icon_id = if copied {
                fret_icons::ids::ui::CHECK
            } else {
                fret_icons::ids::ui::COPY
            };

            let icon = decl_icon::icon_with(cx, icon_id, Some(Px(14.0)), Some(ColorRef::Color(fg)));
            let mut chrome_props = ContainerProps::default();
            chrome_props.layout.size.width = Length::Px(size);
            chrome_props.layout.size.height = Length::Px(size);
            chrome_props.background = Some(bg);
            chrome_props.corner_radii = fret_core::Corners::all(radius);
            chrome_props.border = Edges::all(Px(0.0));
            chrome_props.padding = Edges::all(Px(0.0)).into();

            (props, chrome_props, move |cx| {
                let row = ui::h_flex(move |_cx| vec![icon])
                    .items_center()
                    .justify_center()
                    .layout(LayoutRefinement::default().w_full().h_full())
                    .into_element(cx);

                let marker = copied_marker_test_id.clone().and_then(|marker_id| {
                    copied.then(|| {
                        cx.text_props(TextProps {
                            layout: LayoutStyle {
                                // Keep it out of layout flow.
                                size: fret_ui::element::SizeStyle {
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
                            fret_ui::element::SemanticsDecoration::default()
                                .role(SemanticsRole::Group)
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

/// Monospace filename label for `CodeBlock` headers.
#[derive(Clone)]
pub struct CodeBlockFilename {
    text: Arc<str>,
}

impl CodeBlockFilename {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        cx.text_props(TextProps {
            layout: Default::default(),
            text: self.text,
            style: Some(typography::as_control_text(TextStyle {
                font: FontId::monospace(),
                size: theme.metric_token("metric.font.mono_size"),
                weight: FontWeight::NORMAL,
                slant: Default::default(),
                line_height: Some(theme.metric_token("metric.font.mono_line_height")),
                letter_spacing_em: None,
                ..Default::default()
            })),
            color: Some(theme.color_token("muted-foreground")),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            ink_overflow: Default::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Rect, Size};

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(720.0), Px(480.0)),
        )
    }

    fn has_test_id(element: &AnyElement, test_id: &str) -> bool {
        if element
            .semantics_decoration
            .as_ref()
            .and_then(|d| d.test_id.as_deref())
            == Some(test_id)
        {
            return true;
        }

        element
            .children
            .iter()
            .any(|child| has_test_id(child, test_id))
    }

    #[test]
    fn code_block_keeps_group_role_when_stamping_test_id() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                CodeBlock::new("fn main() {}")
                    .test_id("ui-ai-code-block-root")
                    .into_element(cx)
            });

        assert_eq!(
            element.semantics_decoration.as_ref().and_then(|d| d.role),
            Some(SemanticsRole::Group)
        );
        assert_eq!(
            element
                .semantics_decoration
                .as_ref()
                .and_then(|d| d.test_id.as_deref()),
            Some("ui-ai-code-block-root")
        );
    }

    #[test]
    fn code_block_children_can_consume_inherited_context() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                CodeBlock::new("fn greet() {}")
                    .language("rust")
                    .test_id("ui-ai-code-block-root")
                    .into_element_with_children(cx, |cx| {
                        vec![
                            CodeBlockHeader::new([
                                CodeBlockTitle::new([CodeBlockFilename::new("greet.rs")
                                    .into_element(cx)
                                    .test_id("filename")])
                                .into_element(cx),
                                CodeBlockActions::new([
                                    use_code_block_context(cx)
                                        .map(|_| {
                                            cx.text("context-visible").test_id("context-visible")
                                        })
                                        .expect("code block context"),
                                    CodeBlockCopyButton::from_context()
                                        .test_id("copy")
                                        .into_element(cx),
                                ])
                                .into_element(cx),
                            ])
                            .test_id("header")
                            .into_element(cx),
                        ]
                    })
            });

        assert!(has_test_id(&element, "header"));
        assert!(has_test_id(&element, "filename"));
        assert!(has_test_id(&element, "context-visible"));
    }
}
