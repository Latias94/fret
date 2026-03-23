//! AI Elements-aligned `Task` surfaces.
//!
//! Upstream reference: `repo-ref/ai-elements/packages/elements/src/task.tsx`.

use std::sync::Arc;

use fret_core::{
    Color, Edges, FontWeight, Point, Px, SemanticsRole, TextOverflow, TextStyle, TextWrap,
    Transform2D,
};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ElementKind, HoverRegionProps, LayoutStyle, Length, SemanticsDecoration, TextProps,
    VisualTransformProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::typography;
use fret_ui_kit::ui;
use fret_ui_kit::{
    ChromeRefinement, ColorFallback, ColorRef, Items, LayoutRefinement, Radius, Space,
};
use fret_ui_shadcn::facade::{Collapsible, CollapsibleContent, CollapsibleTrigger};

fn muted_fg(theme: &Theme) -> Color {
    theme
        .color_by_key("muted-foreground")
        .or_else(|| theme.color_by_key("muted_foreground"))
        .unwrap_or_else(|| theme.color_token("foreground"))
}

fn border_muted(theme: &Theme) -> Color {
    theme
        .color_by_key("muted")
        .or_else(|| theme.color_by_key("border"))
        .unwrap_or_else(|| theme.color_token("border"))
}

fn chevron_down_icon_rotated<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    progress: f32,
    size: Px,
) -> AnyElement {
    let degrees = 180.0 * progress.clamp(0.0, 1.0);

    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Px(size);
    layout.size.height = Length::Px(size);

    cx.visual_transform_props(
        VisualTransformProps {
            layout,
            transform: Transform2D::rotation_about_degrees(
                degrees,
                Point::new(Px(size.0 * 0.5), Px(size.0 * 0.5)),
            ),
        },
        move |cx| {
            vec![decl_icon::icon_with(
                cx,
                fret_icons::ids::ui::CHEVRON_DOWN,
                Some(size),
                None,
            )]
        },
    )
}

fn apply_text_style_if_missing(mut el: AnyElement, style: &TextStyle) -> AnyElement {
    el.children = el
        .children
        .into_iter()
        .map(|child| apply_text_style_if_missing(child, style))
        .collect();

    match &mut el.kind {
        ElementKind::Text(props) => {
            if props.style.is_none() {
                props.style = Some(style.clone());
            }
        }
        ElementKind::StyledText(props) => {
            if props.style.is_none() {
                props.style = Some(style.clone());
            }
        }
        _ => {}
    }

    el
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

pub fn use_task_controller<H: UiHost>(cx: &ElementContext<'_, H>) -> Option<TaskController> {
    cx.provided::<TaskController>().cloned()
}

/// Collapsible task root aligned with AI Elements `Task`.
pub struct Task {
    open: Option<Model<bool>>,
    default_open: bool,
    trigger: Option<TaskTrigger>,
    content: Option<TaskContent>,
    test_id_root: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl std::fmt::Debug for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Task")
            .field("open", &self.open.as_ref().map(|_| "<model>"))
            .field("default_open", &self.default_open)
            .field("has_trigger", &self.trigger.is_some())
            .field("has_content", &self.content.is_some())
            .field("test_id_root", &self.test_id_root.as_deref())
            .field("layout", &self.layout)
            .field("chrome", &self.chrome)
            .finish()
    }
}

impl Task {
    /// Docs-shaped compound root aligned with upstream `<Task>...</Task>`.
    pub fn root() -> Self {
        Self {
            open: None,
            default_open: true,
            trigger: None,
            content: None,
            test_id_root: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn new(trigger: TaskTrigger, content: TaskContent) -> Self {
        Self::root().trigger(trigger).content(content)
    }

    pub fn children(mut self, children: impl IntoIterator<Item = TaskChild>) -> Self {
        for child in children {
            match child {
                TaskChild::Trigger(trigger) => {
                    if self.trigger.is_some() {
                        debug_assert!(false, "Task expects a single TaskTrigger");
                    }
                    self.trigger = Some(trigger);
                }
                TaskChild::Content(content) => {
                    if self.content.is_some() {
                        debug_assert!(false, "Task expects a single TaskContent");
                    }
                    self.content = Some(content);
                }
            }
        }
        self
    }

    pub fn trigger(mut self, trigger: TaskTrigger) -> Self {
        self.trigger = Some(trigger);
        self
    }

    pub fn content(mut self, content: TaskContent) -> Self {
        self.content = Some(content);
        self
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
        let Some(trigger) = self.trigger else {
            debug_assert!(false, "Task requires a TaskTrigger");
            return cx.container(Default::default(), |_| Vec::new());
        };
        let chrome = self.chrome;
        let layout = self.layout;
        let content = self
            .content
            .unwrap_or_else(|| TaskContent::new(Vec::<AnyElement>::new()));
        let test_id_root = self.test_id_root;
        let open = fret_ui_kit::primitives::collapsible::CollapsibleRoot::new()
            .open(self.open.clone())
            .default_open(self.default_open)
            .use_open_model(cx)
            .model();
        let is_open = cx
            .get_model_copied(&open, Invalidation::Layout)
            .unwrap_or(false);
        let controller = TaskController {
            open: open.clone(),
            is_open,
        };

        let collapsible = Collapsible::new(open)
            .refine_style(chrome)
            .refine_layout(layout);

        let root = cx.provide(controller, |cx| {
            collapsible.into_element_with_open_model(
                cx,
                move |cx, open, is_open| trigger.into_element_with_open(cx, open, is_open),
                move |cx| content.into_element(cx),
            )
        });

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

pub enum TaskChild {
    Trigger(TaskTrigger),
    Content(TaskContent),
}

impl std::fmt::Debug for TaskChild {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Trigger(_) => f.write_str("TaskChild::Trigger(..)"),
            Self::Content(_) => f.write_str("TaskChild::Content(..)"),
        }
    }
}

impl From<TaskTrigger> for TaskChild {
    fn from(value: TaskTrigger) -> Self {
        Self::Trigger(value)
    }
}

impl From<TaskContent> for TaskChild {
    fn from(value: TaskContent) -> Self {
        Self::Content(value)
    }
}

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

        let trigger = if self.children.is_empty() {
            let title = self.title;
            let icon_size = Px(16.0);
            let hover_layout =
                decl_style::layout_style(&theme, LayoutRefinement::default().w_full().min_w_0());

            let row = cx.hover_region(
                HoverRegionProps {
                    layout: hover_layout,
                },
                move |cx, hovered| {
                    let theme = Theme::global(&*cx.app).clone();

                    let fg = if hovered {
                        theme.color_token("foreground")
                    } else {
                        muted_fg(&theme)
                    };

                    let row = {
                        let search = decl_icon::icon_with(
                            cx,
                            fret_icons::IconId::new_static("lucide.search"),
                            Some(icon_size),
                            None,
                        );

                        let title = cx.text_props(TextProps {
                            layout: LayoutStyle::default(),
                            text: title,
                            style: Some(typography::preset_text_style_with_overrides(
                                &theme,
                                typography::TypographyPreset::control_ui(
                                    typography::UiTextSize::Sm,
                                ),
                                Some(FontWeight::NORMAL),
                                None,
                            )),
                            color: None,
                            wrap: TextWrap::None,
                            overflow: TextOverflow::Clip,
                            align: fret_core::TextAlign::Start,
                            ink_overflow: Default::default(),
                        });

                        let chevron = chevron_down_icon_rotated(
                            cx,
                            if is_open { 1.0 } else { 0.0 },
                            icon_size,
                        );

                        let row = ui::h_row(move |_cx| vec![search, title, chevron])
                            .layout(LayoutRefinement::default().w_full().min_w_0())
                            .items(Items::Center)
                            .gap(Space::N2)
                            .into_element(cx);

                        row
                    };
                    vec![row.inherit_foreground(fg)]
                },
            );

            CollapsibleTrigger::new(open, [row]).a11y_label("Toggle task")
        } else {
            CollapsibleTrigger::new(open, self.children).a11y_label("Toggle task")
        };

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

        let body = ui::v_stack(move |_cx| self.children)
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N2)
            .into_element(cx);

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

        let text_style = typography::preset_text_style_with_overrides(
            &theme,
            typography::TypographyPreset::control_ui(typography::UiTextSize::Sm),
            Some(FontWeight::NORMAL),
            None,
        );
        let children = self
            .children
            .into_iter()
            .map(|child| apply_text_style_if_missing(child, &text_style))
            .collect::<Vec<_>>();

        let row = ui::h_row(move |_cx| children)
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .items(Items::Center)
            .gap(Space::N1)
            .into_element(cx);

        let chrome = ChromeRefinement::default()
            .text_color(ColorRef::Color(muted_fg(&theme)))
            .merge(self.chrome);
        let mut props = decl_style::container_props(&theme, chrome, self.layout);
        props.background = None;
        props.border = Edges::all(Px(0.0));
        props.border_color = None;

        let out = cx.container(props, move |_cx| vec![row]);
        out.inherit_foreground(muted_fg(&theme))
    }
}

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

        let text_style = typography::preset_text_style_with_overrides(
            &theme,
            typography::TypographyPreset::control_ui(typography::UiTextSize::Xs),
            Some(FontWeight::NORMAL),
            None,
        );
        let children = self
            .children
            .into_iter()
            .map(|child| apply_text_style_if_missing(child, &text_style))
            .collect::<Vec<_>>();

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
        props.border_color = Some(theme.color_token("border"));

        let content = ui::h_row(move |_cx| children)
            .layout(LayoutRefinement::default())
            .items(Items::Center)
            .gap(Space::N1)
            .into_element(cx);

        let out = cx.container(props, move |_cx| vec![content]);
        out.inherit_foreground(theme.color_token("foreground"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size};

    fn contains_foreground_scope(el: &AnyElement) -> bool {
        matches!(el.kind, ElementKind::ForegroundScope(_))
            || el.children.iter().any(contains_foreground_scope)
    }

    fn find_first_inherited_foreground_node(el: &AnyElement) -> Option<&AnyElement> {
        if el.inherited_foreground.is_some() {
            return Some(el);
        }
        el.children
            .iter()
            .find_map(find_first_inherited_foreground_node)
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

    #[test]
    fn task_trigger_default_row_attaches_foreground_without_wrapper() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(160.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "task-trigger", |cx| {
            let theme = Theme::global(&*cx.app).clone();
            let expected_fg = muted_fg(&theme);
            let open = cx.app.models_mut().insert(true);

            let el = TaskTrigger::new("Search docs").into_element_with_open(cx, open, true);
            let inherited = find_first_inherited_foreground_node(&el)
                .expect("expected task trigger subtree to carry inherited foreground");

            assert_eq!(inherited.inherited_foreground, Some(expected_fg));
            assert!(
                !contains_foreground_scope(&el),
                "expected task trigger default row to attach inherited foreground without inserting a ForegroundScope"
            );
        });
    }

    #[test]
    fn task_surfaces_use_shared_typography_presets() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(160.0)),
        );

        let el = fret_ui::elements::with_element_cx(&mut app, window, bounds, "task-style", |cx| {
            let open = cx.app.models_mut().insert(true);
            ui::v_stack(|cx| {
                vec![
                    TaskTrigger::new("Search docs").into_element_with_open(cx, open, true),
                    TaskItem::new([cx.text("Gather requirements")]).into_element(cx),
                    TaskItemFile::new([cx.text("Cargo.toml")]).into_element(cx),
                ]
            })
            .into_element(cx)
        });

        let theme = Theme::global(&app).clone();
        let trigger_title = find_text_by_content(&el, "Search docs").expect("task trigger text");
        assert_eq!(
            trigger_title.style,
            Some(typography::preset_text_style_with_overrides(
                &theme,
                typography::TypographyPreset::control_ui(typography::UiTextSize::Sm),
                Some(FontWeight::NORMAL),
                None,
            ))
        );

        let task_item = find_text_by_content(&el, "Gather requirements").expect("task item text");
        assert_eq!(
            task_item.style,
            Some(typography::preset_text_style_with_overrides(
                &theme,
                typography::TypographyPreset::control_ui(typography::UiTextSize::Sm),
                Some(FontWeight::NORMAL),
                None,
            ))
        );

        let file_name = find_text_by_content(&el, "Cargo.toml").expect("task item file text");
        assert_eq!(
            file_name.style,
            Some(typography::preset_text_style_with_overrides(
                &theme,
                typography::TypographyPreset::control_ui(typography::UiTextSize::Xs),
                Some(FontWeight::NORMAL),
                None,
            ))
        );
    }

    #[test]
    fn task_compound_children_surface_resolves_trigger_and_content() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(360.0), Px(220.0)),
        );

        let el = fret_ui::elements::with_element_cx(&mut app, window, bounds, "task-root", |cx| {
            Task::root()
                .default_open(true)
                .children([
                    TaskChild::Trigger(TaskTrigger::new("Found project files")),
                    TaskChild::Content(TaskContent::new([
                        TaskItem::new([cx.text("Read layout.tsx")]).into_element(cx),
                        TaskItem::new([cx.text("Scan 52 files")]).into_element(cx),
                    ])),
                ])
                .into_element(cx)
        });

        assert!(
            find_text_by_content(&el, "Found project files").is_some(),
            "compound Task root should render the trigger title"
        );
        assert!(
            find_text_by_content(&el, "Read layout.tsx").is_some(),
            "compound Task root should render TaskContent children"
        );
    }
}
