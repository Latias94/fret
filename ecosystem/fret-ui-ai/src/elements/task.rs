//! AI Elements-aligned `Task` surfaces.
//!
//! Upstream reference: `repo-ref/ai-elements/packages/elements/src/task.tsx`.

use std::sync::Arc;

use fret_core::{
    Color, Edges, FontId, FontWeight, Px, SemanticsRole, TextOverflow, TextStyle, TextWrap,
};
use fret_runtime::Model;
use fret_ui::element::{AnyElement, LayoutStyle, SemanticsDecoration, TextProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{
    ChromeRefinement, ColorFallback, ColorRef, Justify, LayoutRefinement, Radius, Space,
};
use fret_ui_shadcn::{Collapsible, CollapsibleContent, CollapsibleTrigger};

fn muted_fg(theme: &Theme) -> Color {
    theme
        .color_by_key("muted-foreground")
        .or_else(|| theme.color_by_key("muted_foreground"))
        .unwrap_or_else(|| theme.color_required("foreground"))
}

fn border_muted(theme: &Theme) -> Color {
    theme
        .color_by_key("muted")
        .or_else(|| theme.color_by_key("border"))
        .unwrap_or_else(|| theme.color_required("border"))
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

#[derive(Clone)]
pub struct TaskController {
    pub open: Model<bool>,
    pub is_open: bool,
}

impl std::fmt::Debug for TaskController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskController")
            .field("open", &"<model>")
            .field("is_open", &self.is_open)
            .finish()
    }
}

#[derive(Debug, Default, Clone)]
struct TaskProviderState {
    controller: Option<TaskController>,
}

pub fn use_task_controller<H: UiHost>(cx: &ElementContext<'_, H>) -> Option<TaskController> {
    cx.inherited_state::<TaskProviderState>()
        .and_then(|st| st.controller.clone())
}

#[derive(Clone)]
/// Collapsible task root aligned with AI Elements `Task`.
pub struct Task {
    open: Option<Model<bool>>,
    default_open: bool,
    trigger: TaskTrigger,
    content: TaskContent,
    test_id_root: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl std::fmt::Debug for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Task")
            .field("open", &self.open.as_ref().map(|_| "<model>"))
            .field("default_open", &self.default_open)
            .field("test_id_root", &self.test_id_root.as_deref())
            .field("layout", &self.layout)
            .field("chrome", &self.chrome)
            .finish()
    }
}

impl Task {
    pub fn new(trigger: TaskTrigger, content: TaskContent) -> Self {
        Self {
            open: None,
            default_open: true,
            trigger,
            content,
            test_id_root: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn open_model(mut self, open: Model<bool>) -> Self {
        self.open = Some(open);
        self
    }

    pub fn default_open(mut self, default_open: bool) -> Self {
        self.default_open = default_open;
        self
    }

    pub fn test_id_root(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_root = Some(id.into());
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
        let chrome = self.chrome;
        let layout = self.layout;
        let trigger = self.trigger;
        let content = self.content;
        let test_id_root = self.test_id_root;

        let collapsible = if let Some(open) = self.open {
            Collapsible::new(open)
        } else {
            Collapsible::uncontrolled(self.default_open)
        }
        .refine_style(chrome)
        .refine_layout(layout);

        let root = collapsible.into_element_with_open_model(
            cx,
            move |cx, open, is_open| {
                let controller = TaskController {
                    open: open.clone(),
                    is_open,
                };
                cx.with_state(TaskProviderState::default, |st| {
                    st.controller = Some(controller)
                });
                trigger.into_element_with_open(cx, open, is_open)
            },
            move |cx| content.into_element(cx),
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

#[derive(Clone)]
/// Collapsible trigger aligned with AI Elements `TaskTrigger`.
pub struct TaskTrigger {
    title: Arc<str>,
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl std::fmt::Debug for TaskTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskTrigger")
            .field("title", &self.title)
            .field("children_len", &self.children.len())
            .field("test_id", &self.test_id.as_deref())
            .finish()
    }
}

impl TaskTrigger {
    pub fn new(title: impl Into<Arc<str>>) -> Self {
        Self {
            title: title.into(),
            children: Vec::new(),
            test_id: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
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

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    fn into_element_with_open<H: UiHost + 'static>(
        self,
        cx: &mut ElementContext<'_, H>,
        open: Model<bool>,
        is_open: bool,
    ) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let children = if self.children.is_empty() {
            let fg = muted_fg(&theme);
            let search = decl_icon::icon_with(
                cx,
                fret_icons::IconId::new_static("lucide.search"),
                Some(Px(16.0)),
                Some(ColorRef::Color(fg)),
            );
            let chevron = decl_icon::icon_with(
                cx,
                if is_open {
                    fret_icons::ids::ui::CHEVRON_UP
                } else {
                    fret_icons::ids::ui::CHEVRON_DOWN
                },
                Some(Px(16.0)),
                Some(ColorRef::Color(fg)),
            );
            let title = cx.text_props(TextProps {
                layout: LayoutStyle::default(),
                text: self.title,
                style: Some(text_sm(&theme, FontWeight::NORMAL)),
                color: Some(fg),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
            });
            let left = stack::hstack(
                cx,
                stack::HStackProps::default().items_center().gap(Space::N2),
                move |_cx| vec![search, title],
            );
            let row = stack::hstack(
                cx,
                stack::HStackProps::default()
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .items_center()
                    .justify(Justify::Between)
                    .gap(Space::N2),
                move |_cx| vec![left, chevron],
            );
            vec![row]
        } else {
            self.children
        };

        let trigger = CollapsibleTrigger::new(open, children).a11y_label("Toggle task");
        let el = trigger.into_element(cx, is_open);
        let el = cx.container(
            decl_style::container_props(&theme, self.chrome, self.layout),
            move |_cx| vec![el],
        );

        let Some(test_id) = self.test_id else {
            return el;
        };
        el.attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Button)
                .test_id(test_id),
        )
    }
}

#[derive(Clone)]
/// Collapsible content wrapper aligned with AI Elements `TaskContent`.
pub struct TaskContent {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl std::fmt::Debug for TaskContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskContent")
            .field("children_len", &self.children.len())
            .field("test_id", &self.test_id.as_deref())
            .finish()
    }
}

impl TaskContent {
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

    fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let body = stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .gap(Space::N2),
            move |_cx| self.children,
        );

        let chrome = ChromeRefinement::default().pl(Space::N4).merge(self.chrome);
        let layout = LayoutRefinement::default()
            .w_full()
            .min_w_0()
            .mt(Space::N4)
            .merge(self.layout);

        let mut wrapper = decl_style::container_props(&theme, chrome, layout);
        wrapper.background = None;
        wrapper.border = Edges {
            left: Px(2.0),
            right: Px(0.0),
            top: Px(0.0),
            bottom: Px(0.0),
        };
        wrapper.border_color = Some(border_muted(&theme));

        let content = cx.container(wrapper, move |_cx| vec![body]);
        let content = CollapsibleContent::new([content]).into_element(cx);

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

#[derive(Clone)]
/// Task item wrapper aligned with AI Elements `TaskItem`.
pub struct TaskItem {
    children: Vec<AnyElement>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl std::fmt::Debug for TaskItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskItem")
            .field("children_len", &self.children.len())
            .finish()
    }
}

impl TaskItem {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
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

        let row = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .items_center()
                .gap(Space::N1),
            move |_cx| self.children,
        );

        let chrome = ChromeRefinement::default()
            .text_color(ColorRef::Color(muted_fg(&theme)))
            .merge(self.chrome);
        let mut props = decl_style::container_props(&theme, chrome, self.layout);
        props.background = None;
        props.border = Edges::all(Px(0.0));
        props.border_color = None;

        cx.container(props, move |_cx| vec![row])
    }
}

#[derive(Clone)]
/// File pill aligned with AI Elements `TaskItemFile`.
pub struct TaskItemFile {
    children: Vec<AnyElement>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl std::fmt::Debug for TaskItemFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskItemFile")
            .field("children_len", &self.children.len())
            .finish()
    }
}

impl TaskItemFile {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            layout: LayoutRefinement::default(),
            chrome: ChromeRefinement::default(),
        }
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

        let chrome = ChromeRefinement::default()
            .rounded(Radius::Md)
            .border_1()
            .bg(ColorRef::Token {
                key: "secondary",
                fallback: ColorFallback::ThemeHoverBackground,
            })
            .px(Space::N1p5)
            .py(Space::N0p5)
            .text_color(ColorRef::Token {
                key: "foreground",
                fallback: ColorFallback::ThemeTextPrimary,
            })
            .merge(self.chrome);

        let mut props = decl_style::container_props(&theme, chrome, self.layout);
        props.border_color = Some(theme.color_required("border"));

        let content = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default())
                .items_center()
                .gap(Space::N1),
            move |_cx| self.children,
        );

        cx.container(props, move |_cx| vec![content])
    }
}
