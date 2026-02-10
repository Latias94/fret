//! AI Elements-aligned `EnvironmentVariables` surfaces.

use std::sync::Arc;
use std::time::Duration;

use fret_core::{
    Color, Corners, Edges, FontId, FontWeight, Px, SemanticsRole, TextOverflow, TextStyle,
    TextWrap, TimerToken,
};
use fret_icons::ids;
use fret_runtime::{Effect, Model};
use fret_ui::element::{
    AnyElement, ContainerProps, InteractivityGateProps, LayoutStyle, Length, PressableProps,
    SemanticsDecoration, SizeStyle, TextProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::declarative::controllable_state;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorFallback, ColorRef, LayoutRefinement, Radius, Space};
use fret_ui_shadcn::{Badge, BadgeVariant, Switch};

pub type OnShowValuesChange =
    Arc<dyn Fn(&mut dyn fret_ui::action::UiActionHost, fret_ui::action::ActionCx, bool) + 'static>;

#[derive(Clone)]
pub struct EnvironmentVariablesController {
    pub show_values: Model<bool>,
    pub on_show_values_change: Option<OnShowValuesChange>,
}

impl std::fmt::Debug for EnvironmentVariablesController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EnvironmentVariablesController")
            .field("show_values", &self.show_values.id())
            .field(
                "has_on_show_values_change",
                &self.on_show_values_change.is_some(),
            )
            .finish()
    }
}

#[derive(Debug, Default, Clone)]
struct EnvironmentVariablesProviderState {
    controller: Option<EnvironmentVariablesController>,
}

pub fn use_environment_variables_controller<H: UiHost>(
    cx: &ElementContext<'_, H>,
) -> Option<EnvironmentVariablesController> {
    cx.inherited_state::<EnvironmentVariablesProviderState>()
        .and_then(|st| st.controller.clone())
}

fn alpha(color: Color, a: f32) -> Color {
    Color {
        r: color.r,
        g: color.g,
        b: color.b,
        a: (color.a * a).clamp(0.0, 1.0),
    }
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

fn hidden<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    cx.interactivity_gate_props(
        InteractivityGateProps {
            layout: LayoutStyle::default(),
            present: false,
            interactive: false,
        },
        |_cx| Vec::new(),
    )
}

/// Root surface aligned with AI Elements `EnvironmentVariables`.
#[derive(Clone)]
pub struct EnvironmentVariables {
    show_values: Option<Model<bool>>,
    default_show_values: bool,
    on_show_values_change: Option<OnShowValuesChange>,
    test_id_root: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl std::fmt::Debug for EnvironmentVariables {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EnvironmentVariables")
            .field(
                "controlled_show_values",
                &self.show_values.as_ref().map(|m| m.id()),
            )
            .field("default_show_values", &self.default_show_values)
            .field(
                "has_on_show_values_change",
                &self.on_show_values_change.is_some(),
            )
            .field("test_id_root", &self.test_id_root.as_deref())
            .finish()
    }
}

impl EnvironmentVariables {
    pub fn new() -> Self {
        Self {
            show_values: None,
            default_show_values: false,
            on_show_values_change: None,
            test_id_root: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
    }

    /// Controlled `showValues` (Radix-style).
    pub fn show_values_model(mut self, model: Model<bool>) -> Self {
        self.show_values = Some(model);
        self
    }

    pub fn default_show_values(mut self, show: bool) -> Self {
        self.default_show_values = show;
        self
    }

    /// Called after the toggle is activated.
    pub fn on_show_values_change(mut self, on_change: OnShowValuesChange) -> Self {
        self.on_show_values_change = Some(on_change);
        self
    }

    pub fn test_id_root(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id_root = Some(test_id.into());
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

    pub fn into_element_with_children<H: UiHost + 'static>(
        self,
        cx: &mut ElementContext<'_, H>,
        children: impl FnOnce(
            &mut ElementContext<'_, H>,
            EnvironmentVariablesController,
        ) -> Vec<AnyElement>,
    ) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

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

        let controlled = self.show_values.clone();
        let default_show_values = self.default_show_values;
        let on_show_values_change = self.on_show_values_change.clone();
        let test_id_root = self.test_id_root.clone();

        let root = cx.container(
            decl_style::container_props(&theme, chrome, layout),
            move |cx| {
                let show_values_model =
                    controllable_state::use_controllable_model(cx, controlled.clone(), || {
                        default_show_values
                    })
                    .model();

                let controller = EnvironmentVariablesController {
                    show_values: show_values_model,
                    on_show_values_change: on_show_values_change.clone(),
                };
                cx.with_state(EnvironmentVariablesProviderState::default, |st| {
                    st.controller = Some(controller.clone());
                });

                let body = stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .layout(LayoutRefinement::default().w_full().min_w_0())
                        .gap(Space::N0),
                    move |cx| children(cx, controller),
                );
                vec![body]
            },
        );

        let Some(test_id) = test_id_root else {
            return root;
        };
        root.attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Group)
                .test_id(test_id),
        )
    }
}

/// Header wrapper aligned with AI Elements `EnvironmentVariablesHeader`.
#[derive(Debug, Clone)]
pub struct EnvironmentVariablesHeader {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl EnvironmentVariablesHeader {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            test_id: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default().px(Space::N4).py(Space::N3),
        }
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let chrome = self.chrome.border_color(ColorRef::Token {
            key: "border",
            fallback: ColorFallback::ThemePanelBorder,
        });

        let row = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(self.layout)
                .items_center()
                .justify_between()
                .gap(Space::N3),
            move |_cx| self.children,
        );

        let mut props = decl_style::container_props(&theme, chrome, LayoutRefinement::default());
        props.border.bottom = Px(1.0);
        let el = cx.container(props, move |_cx| vec![row]);

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

/// Title text aligned with AI Elements `EnvironmentVariablesTitle`.
#[derive(Debug, Clone)]
pub struct EnvironmentVariablesTitle {
    text: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
}

impl EnvironmentVariablesTitle {
    pub fn new() -> Self {
        Self {
            text: None,
            test_id: None,
        }
    }

    pub fn text(mut self, text: impl Into<Arc<str>>) -> Self {
        self.text = Some(text.into());
        self
    }

    pub fn test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let fg = theme.color_required("foreground");
        let text = self
            .text
            .unwrap_or_else(|| Arc::<str>::from("Environment Variables"));

        let px = theme.metric_required("font.size");
        let style = TextStyle {
            font: FontId::default(),
            size: px,
            weight: FontWeight::MEDIUM,
            slant: Default::default(),
            line_height: Some(theme.metric_required("font.line_height")),
            letter_spacing_em: None,
        };

        let el = cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text,
            style: Some(style),
            color: Some(fg),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
        });

        let Some(test_id) = self.test_id else {
            return el;
        };
        el.attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Text)
                .test_id(test_id),
        )
    }
}

/// Toggle aligned with AI Elements `EnvironmentVariablesToggle`.
#[derive(Debug, Clone)]
pub struct EnvironmentVariablesToggle {
    a11y_label: Arc<str>,
    test_id: Option<Arc<str>>,
    switch_test_id: Option<Arc<str>>,
    icon_test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl EnvironmentVariablesToggle {
    pub fn new() -> Self {
        Self {
            a11y_label: Arc::<str>::from("Toggle value visibility"),
            test_id: None,
            switch_test_id: None,
            icon_test_id: None,
            layout: LayoutRefinement::default().min_w_0(),
        }
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = label.into();
        self
    }

    pub fn test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
        self
    }

    pub fn test_id_switch(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.switch_test_id = Some(test_id.into());
        self
    }

    pub fn test_id_icon(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.icon_test_id = Some(test_id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(controller) = use_environment_variables_controller(cx) else {
            return hidden(cx);
        };

        let theme = Theme::global(&*cx.app).clone();
        let fg_muted = theme.color_required("muted-foreground");
        let show_values = cx
            .get_model_copied(&controller.show_values, Invalidation::Paint)
            .unwrap_or(false);

        let icon_id = if show_values {
            ids::ui::EYE
        } else {
            ids::ui::EYE_OFF
        };
        let icon =
            decl_icon::icon_with(cx, icon_id, Some(Px(14.0)), Some(ColorRef::Color(fg_muted)));
        let icon = if let Some(test_id) = self.icon_test_id.clone() {
            icon.attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Generic)
                    .test_id(test_id),
            )
        } else {
            icon
        };

        let mut switch = Switch::new(controller.show_values.clone()).a11y_label(self.a11y_label);
        if let Some(test_id) = self.switch_test_id.clone() {
            switch = switch.test_id(test_id);
        }
        let switch = switch.into_element(cx);

        if let Some(on_change) = controller.on_show_values_change.clone() {
            cx.pressable_add_on_activate_for(
                switch.id,
                Arc::new(move |host, action_cx, _reason| {
                    on_change(host, action_cx, !show_values);
                }),
            );
        }

        let row = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(self.layout)
                .gap(Space::N2)
                .items_center(),
            move |_cx| vec![icon, switch],
        );

        let Some(test_id) = self.test_id else {
            return row;
        };
        row.attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Group)
                .test_id(test_id),
        )
    }
}

/// Content wrapper aligned with AI Elements `EnvironmentVariablesContent`.
#[derive(Debug, Clone)]
pub struct EnvironmentVariablesContent {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

#[derive(Debug, Clone)]
struct EnvironmentVariableData {
    name: Arc<str>,
    value: Arc<str>,
}

#[derive(Debug, Default, Clone)]
struct EnvironmentVariableState {
    data: Option<EnvironmentVariableData>,
}

fn use_environment_variable_data<H: UiHost>(
    cx: &ElementContext<'_, H>,
) -> Option<EnvironmentVariableData> {
    cx.inherited_state::<EnvironmentVariableState>()
        .and_then(|st| st.data.clone())
}

/// Row aligned with AI Elements `EnvironmentVariable`.
#[derive(Debug, Clone)]
pub struct EnvironmentVariable {
    name: Arc<str>,
    value: Arc<str>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl EnvironmentVariable {
    pub fn new(name: impl Into<Arc<str>>, value: impl Into<Arc<str>>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            test_id: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default().px(Space::N4).py(Space::N3),
        }
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.into_element_with_children(cx, |cx| {
            vec![
                EnvironmentVariableGroup::new([EnvironmentVariableName::new().into_element(cx)])
                    .into_element(cx),
                EnvironmentVariableValue::new().into_element(cx),
            ]
        })
    }

    pub fn into_element_with_children<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        children: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
    ) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let data = EnvironmentVariableData {
            name: self.name,
            value: self.value,
        };

        let chrome = self.chrome;
        let layout = self.layout;
        let test_id = self.test_id;

        let el = cx.container(
            decl_style::container_props(&theme, chrome, LayoutRefinement::default()),
            move |cx| {
                cx.with_state(EnvironmentVariableState::default, |st| {
                    st.data = Some(data);
                });

                let row = stack::hstack(
                    cx,
                    stack::HStackProps::default()
                        .layout(layout)
                        .gap(Space::N4)
                        .items_center()
                        .justify_between(),
                    move |cx| children(cx),
                );

                vec![row]
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

/// Group aligned with AI Elements `EnvironmentVariableGroup`.
#[derive(Debug, Clone)]
pub struct EnvironmentVariableGroup {
    children: Vec<AnyElement>,
}

impl EnvironmentVariableGroup {
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
                .layout(LayoutRefinement::default().min_w_0()),
            move |_cx| self.children,
        )
    }
}

/// Name aligned with AI Elements `EnvironmentVariableName`.
#[derive(Debug, Clone)]
pub struct EnvironmentVariableName {
    text: Option<Arc<str>>,
}

impl EnvironmentVariableName {
    pub fn new() -> Self {
        Self { text: None }
    }

    pub fn text(mut self, text: impl Into<Arc<str>>) -> Self {
        self.text = Some(text.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let data = use_environment_variable_data(cx);
        let text = self
            .text
            .or_else(|| data.as_ref().map(|d| d.name.clone()))
            .unwrap_or_else(|| Arc::<str>::from(""));

        let fg = theme.color_required("foreground");
        let px = theme.metric_required("font.size");
        let style = monospace_style(&theme, px, FontWeight::NORMAL);

        cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text,
            style: Some(style),
            color: Some(fg),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
        })
    }
}

fn masked_value(value: &str) -> Arc<str> {
    let len = value.chars().count();
    let n = len.min(20);
    let s: String = std::iter::repeat('•').take(n).collect();
    Arc::<str>::from(s)
}

/// Value aligned with AI Elements `EnvironmentVariableValue`.
#[derive(Debug, Clone)]
pub struct EnvironmentVariableValue {
    text: Option<Arc<str>>,
}

impl EnvironmentVariableValue {
    pub fn new() -> Self {
        Self { text: None }
    }

    pub fn text(mut self, text: impl Into<Arc<str>>) -> Self {
        self.text = Some(text.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let data = use_environment_variable_data(cx);
        let value = self
            .text
            .or_else(|| data.as_ref().map(|d| d.value.clone()))
            .unwrap_or_else(|| Arc::<str>::from(""));

        let show_values = use_environment_variables_controller(cx)
            .and_then(|c| cx.get_model_copied(&c.show_values, Invalidation::Paint))
            .unwrap_or(true);

        let display_value = if show_values {
            value.clone()
        } else {
            masked_value(&value)
        };

        let fg = theme.color_required("muted-foreground");
        let px = theme.metric_required("font.size");
        let style = monospace_style(&theme, px, FontWeight::NORMAL);

        cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text: display_value,
            style: Some(style),
            color: Some(fg),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnvironmentVariableCopyFormat {
    Name,
    Value,
    Export,
}

impl Default for EnvironmentVariableCopyFormat {
    fn default() -> Self {
        Self::Value
    }
}

#[derive(Debug, Default)]
struct CopyFeedback {
    copied: bool,
    token: Option<TimerToken>,
}

#[derive(Clone, Default)]
struct CopyFeedbackRef(Arc<std::sync::Mutex<CopyFeedback>>);

impl CopyFeedbackRef {
    fn lock(&self) -> std::sync::MutexGuard<'_, CopyFeedback> {
        self.0.lock().unwrap_or_else(|e| e.into_inner())
    }
}

pub type OnEnvironmentVariableCopy =
    Arc<dyn Fn(&mut dyn fret_ui::action::UiActionHost, fret_ui::action::ActionCx) + 'static>;

/// Copy button aligned with AI Elements `EnvironmentVariableCopyButton`.
#[derive(Clone)]
pub struct EnvironmentVariableCopyButton {
    on_copy: Option<OnEnvironmentVariableCopy>,
    timeout: Duration,
    copy_format: EnvironmentVariableCopyFormat,
    test_id: Option<Arc<str>>,
    copied_marker_test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for EnvironmentVariableCopyButton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EnvironmentVariableCopyButton")
            .field("timeout_ms", &self.timeout.as_millis())
            .field("copy_format", &self.copy_format)
            .field("test_id", &self.test_id.as_deref())
            .field(
                "copied_marker_test_id",
                &self.copied_marker_test_id.as_deref(),
            )
            .finish()
    }
}

impl EnvironmentVariableCopyButton {
    pub fn new() -> Self {
        Self {
            on_copy: None,
            timeout: Duration::from_millis(2000),
            copy_format: EnvironmentVariableCopyFormat::Value,
            test_id: None,
            copied_marker_test_id: None,
        }
    }

    pub fn on_copy(mut self, on_copy: OnEnvironmentVariableCopy) -> Self {
        self.on_copy = Some(on_copy);
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn copy_format(mut self, format: EnvironmentVariableCopyFormat) -> Self {
        self.copy_format = format;
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn copied_marker_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.copied_marker_test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(data) = use_environment_variable_data(cx) else {
            return hidden(cx);
        };

        let theme = Theme::global(&*cx.app).clone();
        let feedback = cx.with_state(CopyFeedbackRef::default, |st| st.clone());

        let name = data.name;
        let value = data.value;
        let on_copy = self.on_copy;
        let timeout = self.timeout;
        let copy_format = self.copy_format;
        let test_id = self.test_id;
        let copied_marker_test_id = self.copied_marker_test_id;

        cx.pressable_with_id_props(move |cx, st, id| {
            let copied = feedback.lock().copied;
            let label: Arc<str> = if copied {
                Arc::<str>::from("Copied")
            } else {
                Arc::<str>::from("Copy environment variable")
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
                let name = name.clone();
                let value = value.clone();
                let feedback = feedback.clone();
                let on_copy = on_copy.clone();
                Arc::new(move |host, action_cx, _reason| {
                    let text = match copy_format {
                        EnvironmentVariableCopyFormat::Name => name.to_string(),
                        EnvironmentVariableCopyFormat::Value => value.to_string(),
                        EnvironmentVariableCopyFormat::Export => {
                            format!("export {name}=\"{value}\"")
                        }
                    };

                    host.push_effect(Effect::ClipboardSetText { text });
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

            let size = Px(24.0);
            let radius = theme.metric_required("metric.radius.sm");

            let icon_id = if copied {
                ids::ui::CHECK
            } else {
                ids::ui::COPY
            };
            let icon = decl_icon::icon_with(cx, icon_id, Some(Px(12.0)), Some(ColorRef::Color(fg)));

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

/// Badge aligned with AI Elements `EnvironmentVariableRequired`.
#[derive(Debug, Clone)]
pub struct EnvironmentVariableRequired {
    text: Option<Arc<str>>,
}

impl EnvironmentVariableRequired {
    pub fn new() -> Self {
        Self { text: None }
    }

    pub fn text(mut self, text: impl Into<Arc<str>>) -> Self {
        self.text = Some(text.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let label = self.text.unwrap_or_else(|| Arc::<str>::from("Required"));

        Badge::new(label)
            .variant(BadgeVariant::Secondary)
            .refine_layout(LayoutRefinement::default().flex_shrink_0())
            .into_element(cx)
    }
}

impl EnvironmentVariablesContent {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            test_id: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let mut divided = Vec::new();
        for (idx, child) in self.children.into_iter().enumerate() {
            if idx != 0 {
                divided.push(environment_variable_divider(cx, &theme));
            }
            divided.push(child);
        }

        let body = stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(self.layout)
                .gap(Space::N0),
            move |_cx| divided,
        );

        let el = cx.container(
            decl_style::container_props(&theme, self.chrome, LayoutRefinement::default()),
            move |_cx| vec![body],
        );

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

fn environment_variable_divider<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
) -> AnyElement {
    let border = theme.color_required("border");
    cx.container(
        decl_style::container_props(
            theme,
            ChromeRefinement::default().bg(ColorRef::Color(border)),
            LayoutRefinement::default().w_full().h_px(Px(1.0)),
        ),
        |_cx| Vec::new(),
    )
}
