use std::sync::Arc;
use std::time::Duration;

use fret_core::{
    ClipboardAccessError, Color, Edges, FontId, FontWeight, Px, SemanticsRole, TextOverflow,
    TextStyle, TextWrap,
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
use fret_ui_shadcn::facade::{
    IntoBoolModel, IntoOptionalTextValueModel, Select as ShadcnSelect, SelectAlign,
    SelectContent as ShadcnSelectContent, SelectEntry as ShadcnSelectEntry,
    SelectItem as ShadcnSelectItem, SelectItemIndicator, SelectItemText, SelectScrollButtons,
    SelectScrollDownButton, SelectScrollUpButton, SelectSide, SelectTrigger as ShadcnSelectTrigger,
    SelectTriggerSize, SelectValue as ShadcnSelectValue,
};

use super::clipboard_copy::{
    ClipboardCopyFeedbackRef, begin_request, finish_request, handle_reset_timer,
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
    windowed: Option<fret_code_view::CodeBlockWindowedOptions>,
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
            .field("windowed", &self.windowed)
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
            windowed: None,
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

    /// Enables the explicit retained/windowed renderer for long snippets.
    ///
    /// This requires the explicit retained entrypoints `into_element_windowed` /
    /// `into_element_with_children_windowed`. The default `into_element` methods stay
    /// non-windowed so downstream composition does not inherit a `'static` bound by default.
    pub fn windowed(mut self, windowed: fret_code_view::CodeBlockWindowedOptions) -> Self {
        self.windowed = Some(windowed);
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

    pub fn into_element_with_children<H: UiHost>(
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

    pub fn into_element_with_children_windowed<H: UiHost + 'static>(
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
        self.children(children(cx)).into_element_windowed(cx)
    }

    pub fn into_element_with_children_non_windowed<H: UiHost>(
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
        self.children(children(cx)).into_element_non_windowed(cx)
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        debug_assert!(
            self.windowed.is_none(),
            "CodeBlock::windowed(...) requires CodeBlock::into_element_windowed(...)"
        );

        if !self.children.is_empty() {
            return self.into_element_composable_non_windowed(cx);
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
            show_scrollbar_x: true,
            scrollbar_x_on_hover: true,
            show_scrollbar_y: true,
            scrollbar_y_on_hover: true,
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

    pub fn into_element_windowed<H: UiHost + 'static>(
        self,
        cx: &mut ElementContext<'_, H>,
    ) -> AnyElement {
        let windowed = self.windowed.unwrap_or_default();
        if !self.children.is_empty() {
            let theme = Theme::global(&*cx.app).clone();
            let mut chrome = ChromeRefinement::default().rounded(Radius::Md);
            chrome = chrome
                .border_1()
                .bg(ColorRef::Color(theme.color_token("card")))
                .border_color(ColorRef::Color(theme.color_token("border")));

            let mut props = decl_style::container_props(&theme, chrome, self.layout);
            props.layout.overflow = Overflow::Clip;

            let content = fret_code_view::code_block_with_header_slots_windowed(
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
                    show_scrollbar_x: true,
                    scrollbar_x_on_hover: true,
                    show_scrollbar_y: true,
                    scrollbar_y_on_hover: true,
                },
                fret_code_view::CodeBlockHeaderSlots::default(),
                windowed,
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

            return match self.test_id {
                Some(test_id) => el.attach_semantics(
                    fret_ui::element::SemanticsDecoration::default()
                        .role(SemanticsRole::Group)
                        .test_id(test_id),
                ),
                None => el,
            };
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
            show_scrollbar_x: true,
            scrollbar_x_on_hover: true,
            show_scrollbar_y: true,
            scrollbar_y_on_hover: true,
        };

        let code = self.code;
        let language = self.language;
        let show_line_numbers = self.show_line_numbers;
        let content = fret_code_view::code_block_with_header_slots_windowed(
            cx,
            &code,
            language.as_deref(),
            show_line_numbers,
            options,
            header,
            windowed,
        );

        let el = cx.container(
            ContainerProps {
                layout: decl_style::layout_style(&theme, self.layout),
                ..Default::default()
            },
            move |_cx| vec![content],
        );

        match self.test_id {
            Some(test_id) => el.attach_semantics(
                fret_ui::element::SemanticsDecoration::default()
                    .role(SemanticsRole::Group)
                    .test_id(test_id),
            ),
            None => el,
        }
    }

    pub fn into_element_non_windowed<H: UiHost>(
        mut self,
        cx: &mut ElementContext<'_, H>,
    ) -> AnyElement {
        self.windowed = None;
        self.into_element(cx)
    }

    fn into_element_composable_non_windowed<H: UiHost>(
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

        let content = fret_code_view::code_block_with_header_slots_non_windowed(
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
    on_error: Option<
        Arc<
            dyn Fn(
                    &mut dyn fret_ui::action::UiActionHost,
                    fret_ui::action::ActionCx,
                    ClipboardAccessError,
                ) + 'static,
        >,
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
            on_error: None,
            timeout: Duration::from_millis(2000),
            test_id: None,
            copied_marker_test_id: None,
        }
    }

    pub fn from_context() -> Self {
        Self {
            code: None,
            on_copy: None,
            on_error: None,
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
    /// This is intended for `fretboard-dev diag` scripts.
    pub fn copied_marker_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.copied_marker_test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let feedback = cx.slot_state(ClipboardCopyFeedbackRef::default, |st| st.clone());

        let code = self
            .code
            .or_else(|| use_code_block_context(cx).map(|context| context.code))
            .unwrap_or_else(|| Arc::<str>::from(""));
        let on_copy = self.on_copy;
        let on_error = self.on_error;
        let timeout = self.timeout;
        let test_id = self.test_id;
        let copied_marker_test_id = self.copied_marker_test_id;

        centered_fixed_chrome_pressable_with_id_props(cx, move |cx, st, id| {
            let copied = feedback.is_copied();
            let label = if copied { "Copied" } else { "Copy" };

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
                let code = code.clone();
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
                        text: code.to_string(),
                    });
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

/// Docs-aligned `CodeBlockLanguageSelector` wrapper backed by shadcn `Select`.
#[derive(Clone)]
pub struct CodeBlockLanguageSelector {
    inner: ShadcnSelect,
}

impl std::fmt::Debug for CodeBlockLanguageSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CodeBlockLanguageSelector")
            .finish_non_exhaustive()
    }
}

impl CodeBlockLanguageSelector {
    pub fn new(model: impl IntoOptionalTextValueModel, open: impl IntoBoolModel) -> Self {
        Self {
            inner: ShadcnSelect::new(model, open),
        }
    }

    pub fn on_value_change(
        mut self,
        f: impl Fn(&mut dyn fret_ui::action::UiActionHost, fret_ui::action::ActionCx, Arc<str>)
        + 'static,
    ) -> Self {
        self.inner = self.inner.on_value_change(f);
        self
    }

    pub fn trigger(mut self, trigger: CodeBlockLanguageSelectorTrigger) -> Self {
        self.inner = self.inner.trigger(trigger.into_inner());
        self
    }

    pub fn value(mut self, value: CodeBlockLanguageSelectorValue) -> Self {
        self.inner = self.inner.value(value.into_inner());
        self
    }

    pub fn content(mut self, content: CodeBlockLanguageSelectorContent) -> Self {
        self.inner = self.inner.content(content.into_inner());
        self
    }

    pub fn item(mut self, item: CodeBlockLanguageSelectorItem) -> Self {
        self.inner = self.inner.item(item.into_inner());
        self
    }

    pub fn entry<E>(mut self, entry: E) -> Self
    where
        E: Into<ShadcnSelectEntry>,
    {
        self.inner = self.inner.entry(entry);
        self
    }

    pub fn entries<I, E>(mut self, entries: I) -> Self
    where
        I: IntoIterator<Item = E>,
        E: Into<ShadcnSelectEntry>,
    {
        self.inner = self.inner.entries(entries.into_iter().map(Into::into));
        self
    }

    pub fn trigger_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.inner = self.inner.trigger_test_id(id);
        self
    }

    pub fn test_id_prefix(mut self, prefix: impl Into<Arc<str>>) -> Self {
        self.inner = self.inner.test_id_prefix(prefix);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.inner = self.inner.disabled(disabled);
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.inner = self.inner.a11y_label(label);
        self
    }

    pub fn into_inner(self) -> ShadcnSelect {
        self.inner
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.inner.into_element(cx)
    }
}

impl From<CodeBlockLanguageSelector> for ShadcnSelect {
    fn from(value: CodeBlockLanguageSelector) -> Self {
        value.into_inner()
    }
}

/// Docs-aligned `CodeBlockLanguageSelectorItem` wrapper backed by shadcn `SelectItem`.
#[derive(Debug, Clone)]
pub struct CodeBlockLanguageSelectorItem {
    inner: ShadcnSelectItem,
}

impl CodeBlockLanguageSelectorItem {
    pub fn new(value: impl Into<Arc<str>>, label: impl Into<Arc<str>>) -> Self {
        Self {
            inner: ShadcnSelectItem::new(value, label),
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.inner = self.inner.test_id(id);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.inner = self.inner.disabled(disabled);
        self
    }

    pub fn show_value_in_list(mut self, show: bool) -> Self {
        self.inner = self.inner.show_value_in_list(show);
        self
    }

    pub fn item_text(mut self, text: SelectItemText) -> Self {
        self.inner = self.inner.item_text(text);
        self
    }

    pub fn indicator(mut self, indicator: SelectItemIndicator) -> Self {
        self.inner = self.inner.indicator(indicator);
        self
    }

    pub fn label_font_feature(mut self, tag: impl Into<String>, value: u32) -> Self {
        self.inner = self.inner.label_font_feature(tag, value);
        self
    }

    pub fn label_font_axis(mut self, tag: impl Into<String>, value: f32) -> Self {
        self.inner = self.inner.label_font_axis(tag, value);
        self
    }

    pub fn label_tabular_nums(mut self) -> Self {
        self.inner = self.inner.label_tabular_nums();
        self
    }

    pub fn into_inner(self) -> ShadcnSelectItem {
        self.inner
    }
}

impl From<CodeBlockLanguageSelectorItem> for ShadcnSelectItem {
    fn from(value: CodeBlockLanguageSelectorItem) -> Self {
        value.into_inner()
    }
}

impl From<CodeBlockLanguageSelectorItem> for ShadcnSelectEntry {
    fn from(value: CodeBlockLanguageSelectorItem) -> Self {
        value.into_inner().into()
    }
}

/// Docs-aligned `CodeBlockLanguageSelectorValue` wrapper backed by shadcn `SelectValue`.
#[derive(Debug, Clone, Default)]
pub struct CodeBlockLanguageSelectorValue {
    inner: ShadcnSelectValue,
}

impl CodeBlockLanguageSelectorValue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.inner = self.inner.placeholder(placeholder);
        self
    }

    pub fn into_inner(self) -> ShadcnSelectValue {
        self.inner
    }
}

impl From<CodeBlockLanguageSelectorValue> for ShadcnSelectValue {
    fn from(value: CodeBlockLanguageSelectorValue) -> Self {
        value.into_inner()
    }
}

/// Docs-aligned `CodeBlockLanguageSelectorContent` wrapper with the official default
/// `align="end"` surface.
#[derive(Debug, Clone)]
pub struct CodeBlockLanguageSelectorContent {
    inner: ShadcnSelectContent,
}

impl Default for CodeBlockLanguageSelectorContent {
    fn default() -> Self {
        Self {
            inner: ShadcnSelectContent::new().align(SelectAlign::End),
        }
    }
}

impl CodeBlockLanguageSelectorContent {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn align(mut self, align: SelectAlign) -> Self {
        self.inner = self.inner.align(align);
        self
    }

    pub fn side(mut self, side: SelectSide) -> Self {
        self.inner = self.inner.side(side);
        self
    }

    pub fn align_offset(mut self, offset: Px) -> Self {
        self.inner = self.inner.align_offset(offset);
        self
    }

    pub fn side_offset(mut self, offset: Px) -> Self {
        self.inner = self.inner.side_offset(offset);
        self
    }

    pub fn arrow(mut self, arrow: bool) -> Self {
        self.inner = self.inner.arrow(arrow);
        self
    }

    pub fn arrow_size(mut self, size: Px) -> Self {
        self.inner = self.inner.arrow_size(size);
        self
    }

    pub fn arrow_padding(mut self, padding: Px) -> Self {
        self.inner = self.inner.arrow_padding(padding);
        self
    }

    pub fn scroll_buttons(mut self, buttons: SelectScrollButtons) -> Self {
        self.inner = self.inner.scroll_buttons(buttons);
        self
    }

    pub fn scroll_up_button(mut self, button: SelectScrollUpButton) -> Self {
        self.inner = self.inner.scroll_up_button(button);
        self
    }

    pub fn scroll_down_button(mut self, button: SelectScrollDownButton) -> Self {
        self.inner = self.inner.scroll_down_button(button);
        self
    }

    pub fn into_inner(self) -> ShadcnSelectContent {
        self.inner
    }
}

impl From<CodeBlockLanguageSelectorContent> for ShadcnSelectContent {
    fn from(value: CodeBlockLanguageSelectorContent) -> Self {
        value.into_inner()
    }
}

/// Code-block-scoped Select trigger aligned with AI Elements header chrome.
#[derive(Debug, Clone)]
pub struct CodeBlockLanguageSelectorTrigger {
    inner: ShadcnSelectTrigger,
}

impl Default for CodeBlockLanguageSelectorTrigger {
    fn default() -> Self {
        let transparent = ColorRef::Color(Color::TRANSPARENT);
        let chrome = ChromeRefinement::default()
            .shadow_none()
            .bg(transparent.clone())
            .border_width(Px(0.0))
            .border_color(transparent)
            .px(Space::N2)
            .py(Space::N0p5);

        Self {
            inner: ShadcnSelectTrigger::new()
                .size(SelectTriggerSize::Sm)
                .refine_style(chrome),
        }
    }
}

impl CodeBlockLanguageSelectorTrigger {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn size(mut self, size: SelectTriggerSize) -> Self {
        self.inner = self.inner.size(size);
        self
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.inner = self.inner.refine_style(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.inner = self.inner.refine_layout(layout);
        self
    }

    pub fn into_inner(self) -> ShadcnSelectTrigger {
        self.inner
    }
}

impl From<CodeBlockLanguageSelectorTrigger> for ShadcnSelectTrigger {
    fn from(value: CodeBlockLanguageSelectorTrigger) -> Self {
        value.into_inner()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Rect, Size};
    use fret_runtime::Model;

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

    #[test]
    fn code_block_language_selector_wrappers_integrate_with_header_children() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let language: Model<Option<Arc<str>>> = app
            .models_mut()
            .insert(Some(Arc::<str>::from("typescript")));
        let open: Model<bool> = app.models_mut().insert(false);

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                CodeBlock::new("function greet() {}")
                    .language("typescript")
                    .test_id("root")
                    .into_element_with_children(cx, |cx| {
                        vec![
                            CodeBlockHeader::new([CodeBlockActions::new([
                                CodeBlockLanguageSelector::new(language.clone(), open.clone())
                                    .trigger_test_id("language-trigger")
                                    .trigger(CodeBlockLanguageSelectorTrigger::new())
                                    .value(
                                        CodeBlockLanguageSelectorValue::new()
                                            .placeholder("Language"),
                                    )
                                    .content(CodeBlockLanguageSelectorContent::new())
                                    .entries([
                                        CodeBlockLanguageSelectorItem::new(
                                            "typescript",
                                            "TypeScript",
                                        ),
                                        CodeBlockLanguageSelectorItem::new("python", "Python"),
                                    ])
                                    .into_element(cx),
                            ])
                            .test_id("header")
                            .into_element(cx)])
                            .into_element(cx),
                        ]
                    })
            });

        assert!(has_test_id(&element, "root"));
        assert!(has_test_id(&element, "header"));
    }
}
