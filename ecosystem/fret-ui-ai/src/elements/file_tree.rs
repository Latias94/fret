use std::collections::HashSet;
use std::sync::Arc;

use fret_core::{Color, Edges, Point, Px, SemanticsRole, Transform2D};
use fret_icons::ids;
use fret_runtime::Model;
use fret_ui::action::{ActionCx, UiActionHost};
use fret_ui::element::{
    AnyElement, ContainerProps, LayoutStyle, Length, PressableA11y, PressableProps,
    VisualTransformProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::chrome::control_chrome_pressable_with_id_props;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{
    ChromeRefinement, ColorFallback, ColorRef, Items, LayoutRefinement, MetricRef, Radius, Space,
    ui,
};

pub type OnFileTreeSelect = Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, Arc<str>) + 'static>;
pub type OnFileTreeExpandedChange =
    Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, Arc<[Arc<str>]>) + 'static>;

fn alpha(color: Color, a: f32) -> Color {
    Color {
        a: a.clamp(0.0, 1.0),
        ..color
    }
}

fn resolve_muted(theme: &Theme) -> Color {
    theme
        .color_by_key("muted")
        .unwrap_or_else(|| theme.color_required("muted.background"))
}

fn resolve_border(theme: &Theme) -> Color {
    theme
        .color_by_key("border")
        .unwrap_or_else(|| theme.color_required("border"))
}

fn resolve_background(theme: &Theme) -> Color {
    theme
        .color_by_key("background")
        .or_else(|| theme.color_by_key("card"))
        .unwrap_or_else(|| theme.color_required("card"))
}

fn resolve_muted_fg(theme: &Theme) -> Color {
    theme
        .color_by_key("muted-foreground")
        .or_else(|| theme.color_by_key("muted_foreground"))
        .unwrap_or_else(|| theme.color_required("foreground"))
}

fn sorted_arc_slice_from_set(set: &HashSet<Arc<str>>) -> Arc<[Arc<str>]> {
    let mut v: Vec<Arc<str>> = set.iter().cloned().collect();
    v.sort_by(|a, b| a.as_ref().cmp(b.as_ref()));
    Arc::from(v)
}

#[derive(Debug, Clone)]
pub enum FileTreeItem {
    Folder(FileTreeFolder),
    File(FileTreeFile),
}

impl From<FileTreeFolder> for FileTreeItem {
    fn from(value: FileTreeFolder) -> Self {
        Self::Folder(value)
    }
}

impl From<FileTreeFile> for FileTreeItem {
    fn from(value: FileTreeFile) -> Self {
        Self::File(value)
    }
}

#[derive(Clone)]
/// AI Elements-aligned `FileTree` surface (`file-tree.tsx`).
///
/// Notes:
/// - This is a small, nested tree surface (no virtualization). For large outlines / file trees,
///   prefer the UI Kit retained/virtualized helpers under `fret-ui-kit`.
/// - Expansion state uses a set of paths (`expanded_paths`), matching the upstream contract.
pub struct FileTree {
    items: Vec<FileTreeItem>,
    expanded_paths: Option<Model<HashSet<Arc<str>>>>,
    default_expanded: Arc<[Arc<str>]>,
    selected_path: Option<Arc<str>>,
    on_select: Option<OnFileTreeSelect>,
    on_expanded_change: Option<OnFileTreeExpandedChange>,
    test_id_root: Option<Arc<str>>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for FileTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileTree")
            .field("items_len", &self.items.len())
            .field("expanded_paths", &"<model>")
            .field("default_expanded_len", &self.default_expanded.len())
            .field("selected_path", &self.selected_path.as_deref())
            .field("has_on_select", &self.on_select.is_some())
            .field("has_on_expanded_change", &self.on_expanded_change.is_some())
            .field("test_id_root", &self.test_id_root.as_deref())
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .finish()
    }
}

impl FileTree {
    pub fn new(items: impl IntoIterator<Item = FileTreeItem>) -> Self {
        Self {
            items: items.into_iter().collect(),
            expanded_paths: None,
            default_expanded: Arc::from([]),
            selected_path: None,
            on_select: None,
            on_expanded_change: None,
            test_id_root: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn expanded_paths(mut self, expanded_paths: Model<HashSet<Arc<str>>>) -> Self {
        self.expanded_paths = Some(expanded_paths);
        self
    }

    pub fn default_expanded(mut self, default_expanded: impl Into<Arc<[Arc<str>]>>) -> Self {
        self.default_expanded = default_expanded.into();
        self
    }

    pub fn selected_path(mut self, selected_path: Option<impl Into<Arc<str>>>) -> Self {
        self.selected_path = selected_path.map(Into::into);
        self
    }

    pub fn on_select(mut self, on_select: OnFileTreeSelect) -> Self {
        self.on_select = Some(on_select);
        self
    }

    pub fn on_expanded_change(mut self, on_expanded_change: OnFileTreeExpandedChange) -> Self {
        self.on_expanded_change = Some(on_expanded_change);
        self
    }

    pub fn test_id_root(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_root = Some(id.into());
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

    fn resolve_expanded_model<H: UiHost>(
        &self,
        cx: &mut ElementContext<'_, H>,
    ) -> Model<HashSet<Arc<str>>> {
        #[derive(Default)]
        struct ExpandedState {
            model: Option<Model<HashSet<Arc<str>>>>,
        }

        if let Some(model) = self.expanded_paths.clone() {
            return model;
        }

        let existing = cx.with_state(ExpandedState::default, |st| st.model.clone());
        if let Some(existing) = existing {
            return existing;
        }

        let seed: HashSet<Arc<str>> = self.default_expanded.iter().cloned().collect();
        let model = cx.app.models_mut().insert(seed);
        cx.with_state(ExpandedState::default, |st| st.model = Some(model.clone()));
        model
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let expanded_model = self.resolve_expanded_model(cx);
        let expanded: HashSet<Arc<str>> = cx
            .watch_model(&expanded_model)
            .layout()
            .cloned()
            .unwrap_or_default();

        let chrome = ChromeRefinement::default()
            .p(Space::N2)
            .rounded(Radius::Lg)
            .border_1()
            .bg(ColorRef::Color(resolve_background(&theme)))
            .border_color(ColorRef::Color(resolve_border(&theme)))
            .merge(self.chrome);
        let layout = LayoutRefinement::default().min_w_0().merge(self.layout);

        let props = decl_style::container_props(&theme, chrome, layout);
        let selected_path = self.selected_path;
        let on_select = self.on_select;
        let on_expanded_change = self.on_expanded_change;
        let items = self.items;
        let test_id_root = self.test_id_root;

        cx.container(props, move |cx| {
            let root = stack::vstack(
                cx,
                stack::VStackProps::default()
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .gap(Space::N0),
                move |cx| {
                    render_items(
                        cx,
                        &theme,
                        expanded_model.clone(),
                        &expanded,
                        selected_path.as_ref(),
                        on_select.as_ref(),
                        on_expanded_change.as_ref(),
                        &items,
                    )
                },
            );

            let root = if let Some(test_id_root) = test_id_root.clone() {
                root.attach_semantics(fret_ui::element::SemanticsDecoration {
                    role: Some(SemanticsRole::List),
                    test_id: Some(test_id_root),
                    ..Default::default()
                })
            } else {
                root.attach_semantics(fret_ui::element::SemanticsDecoration {
                    role: Some(SemanticsRole::List),
                    ..Default::default()
                })
            };

            vec![root]
        })
    }
}

#[derive(Debug, Clone)]
pub struct FileTreeFolder {
    pub path: Arc<str>,
    pub name: Arc<str>,
    pub children: Vec<FileTreeItem>,
    pub test_id: Option<Arc<str>>,
}

impl FileTreeFolder {
    pub fn new(path: impl Into<Arc<str>>, name: impl Into<Arc<str>>) -> Self {
        Self {
            path: path.into(),
            name: name.into(),
            children: Vec::new(),
            test_id: None,
        }
    }

    pub fn child(mut self, child: impl Into<FileTreeItem>) -> Self {
        self.children.push(child.into());
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = FileTreeItem>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }
}

#[derive(Debug, Clone)]
pub struct FileTreeFile {
    pub path: Arc<str>,
    pub name: Arc<str>,
    pub icon: Option<fret_icons::IconId>,
    pub test_id: Option<Arc<str>>,
}

impl FileTreeFile {
    pub fn new(path: impl Into<Arc<str>>, name: impl Into<Arc<str>>) -> Self {
        Self {
            path: path.into(),
            name: name.into(),
            icon: None,
            test_id: None,
        }
    }

    pub fn icon(mut self, icon: fret_icons::IconId) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }
}

#[derive(Debug, Clone)]
pub struct FileTreeIcon {
    icon: fret_icons::IconId,
    color: Option<ColorRef>,
}

impl FileTreeIcon {
    pub fn new(icon: fret_icons::IconId) -> Self {
        Self { icon, color: None }
    }

    pub fn color(mut self, color: ColorRef) -> Self {
        self.color = Some(color);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        decl_icon::icon_with(cx, self.icon, Some(Px(16.0)), self.color)
    }
}

#[derive(Debug, Clone)]
pub struct FileTreeName {
    name: Arc<str>,
}

impl FileTreeName {
    pub fn new(name: impl Into<Arc<str>>) -> Self {
        Self { name: name.into() }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        ui::text(cx, self.name)
            .flex_1()
            .min_w_0()
            .truncate()
            .into_element(cx)
    }
}

fn render_items<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    expanded_model: Model<HashSet<Arc<str>>>,
    expanded: &HashSet<Arc<str>>,
    selected_path: Option<&Arc<str>>,
    on_select: Option<&OnFileTreeSelect>,
    on_expanded_change: Option<&OnFileTreeExpandedChange>,
    items: &[FileTreeItem],
) -> Vec<AnyElement> {
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        match item {
            FileTreeItem::Folder(folder) => {
                let path = folder.path.clone();
                out.push(cx.keyed(path, |cx| {
                    render_folder(
                        cx,
                        theme,
                        expanded_model.clone(),
                        expanded,
                        selected_path,
                        on_select,
                        on_expanded_change,
                        folder.clone(),
                    )
                }));
            }
            FileTreeItem::File(file) => {
                let path = file.path.clone();
                out.push(cx.keyed(path, |cx| {
                    render_file(cx, theme, selected_path, on_select, file.clone())
                }));
            }
        }
    }
    out
}

fn render_folder<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    expanded_model: Model<HashSet<Arc<str>>>,
    expanded: &HashSet<Arc<str>>,
    selected_path: Option<&Arc<str>>,
    on_select: Option<&OnFileTreeSelect>,
    on_expanded_change: Option<&OnFileTreeExpandedChange>,
    folder: FileTreeFolder,
) -> AnyElement {
    let is_expanded = expanded.contains(folder.path.as_ref());
    let is_selected = selected_path.is_some_and(|p| p.as_ref() == folder.path.as_ref());

    let muted = resolve_muted(theme);
    let hover_bg = alpha(muted, 0.5);
    let selected_bg = muted;

    let row_test_id = folder.test_id.clone();

    let row = control_chrome_pressable_with_id_props(cx, |cx, st, _id| {
        cx.pressable_on_activate({
            let path = folder.path.clone();
            let expanded_model = expanded_model.clone();
            let on_select = on_select.cloned();
            let on_expanded_change = on_expanded_change.cloned();
            let has_children = !folder.children.is_empty();
            Arc::new(move |host, action_cx, _reason| {
                if let Some(on_select) = on_select.as_ref() {
                    on_select(host, action_cx, path.clone());
                }

                if has_children {
                    let expanded_snapshot = host
                        .models_mut()
                        .update(&expanded_model, |set| {
                            if !set.insert(path.clone()) {
                                set.remove(path.as_ref());
                            }
                            sorted_arc_slice_from_set(set)
                        })
                        .ok();

                    if let (Some(on_expanded_change), Some(expanded_snapshot)) =
                        (on_expanded_change.as_ref(), expanded_snapshot)
                    {
                        on_expanded_change(host, action_cx, expanded_snapshot);
                    }
                }

                host.notify(action_cx);
                host.request_redraw(action_cx.window);
            })
        });

        let mut pressable = PressableProps::default();
        pressable.enabled = true;
        pressable.a11y = PressableA11y {
            role: Some(SemanticsRole::TreeItem),
            label: Some(folder.name.clone()),
            selected: is_selected,
            test_id: row_test_id.clone(),
            expanded: Some(is_expanded),
            ..Default::default()
        };

        let bg = if is_selected {
            Some(selected_bg)
        } else if st.hovered {
            Some(hover_bg)
        } else {
            None
        };

        let mut chrome = ContainerProps::default();
        chrome.padding = Edges {
            top: MetricRef::space(Space::N1).resolve(theme),
            right: MetricRef::space(Space::N2).resolve(theme),
            bottom: MetricRef::space(Space::N1).resolve(theme),
            left: MetricRef::space(Space::N2).resolve(theme),
        };
        chrome.background = bg;
        chrome.corner_radii = fret_core::Corners::all(MetricRef::radius(Radius::Sm).resolve(theme));
        chrome.border = Edges::all(Px(0.0));
        chrome.layout =
            decl_style::layout_style(theme, LayoutRefinement::default().w_full().min_w_0());

        let chevron_fg = resolve_muted_fg(theme);
        let chevron_rotation = if is_expanded { 90.0 } else { 0.0 };
        let chevron_size = Px(16.0);
        let center = Point::new(Px(8.0), Px(8.0));
        let chevron_transform = Transform2D::rotation_about_degrees(chevron_rotation, center);
        let chevron = cx.visual_transform_props(
            VisualTransformProps {
                layout: decl_style::layout_style(
                    theme,
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
                    ids::ui::CHEVRON_RIGHT,
                    Some(chevron_size),
                    Some(ColorRef::Color(chevron_fg)),
                )]
            },
        );

        let folder_icon = FileTreeIcon::new(if is_expanded {
            ids::ui::FOLDER_OPEN
        } else {
            ids::ui::FOLDER
        })
        .color(ColorRef::Token {
            key: "primary",
            fallback: ColorFallback::ThemeAccent,
        })
        .into_element(cx);

        let name = FileTreeName::new(folder.name.clone()).into_element(cx);

        let row_contents = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .gap(Space::N1)
                .items(Items::Center),
            move |_cx| vec![chevron, folder_icon, name],
        );

        (pressable, chrome, move |_cx| vec![row_contents])
    });

    if !is_expanded || folder.children.is_empty() {
        return row;
    }

    let indent_ml = MetricRef::space(Space::N4).resolve(theme);
    let indent_pl = MetricRef::space(Space::N2).resolve(theme);
    let border = resolve_border(theme);

    let mut content_props = ContainerProps::default();
    content_props.layout = {
        let mut layout = LayoutStyle::default();
        layout.margin.left = fret_ui::element::MarginEdge::Px(indent_ml);
        layout
    };
    content_props.padding = Edges {
        top: Px(0.0),
        right: Px(0.0),
        bottom: Px(0.0),
        left: indent_pl,
    };
    content_props.border = Edges {
        top: Px(0.0),
        right: Px(0.0),
        bottom: Px(0.0),
        left: Px(1.0),
    };
    content_props.border_color = Some(border);

    let children = folder.children;
    let content = cx.container(content_props, move |cx| {
        vec![stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .gap(Space::N0),
            move |cx| {
                render_items(
                    cx,
                    theme,
                    expanded_model.clone(),
                    expanded,
                    selected_path,
                    on_select,
                    on_expanded_change,
                    &children,
                )
            },
        )]
    });

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N0),
        move |_cx| vec![row, content],
    )
}

fn render_file<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    selected_path: Option<&Arc<str>>,
    on_select: Option<&OnFileTreeSelect>,
    file: FileTreeFile,
) -> AnyElement {
    let is_selected = selected_path.is_some_and(|p| p.as_ref() == file.path.as_ref());

    let muted = resolve_muted(theme);
    let hover_bg = alpha(muted, 0.5);
    let selected_bg = muted;

    let row_test_id = file.test_id.clone();

    control_chrome_pressable_with_id_props(cx, |cx, st, _id| {
        cx.pressable_on_activate({
            let path = file.path.clone();
            let on_select = on_select.cloned();
            Arc::new(move |host, action_cx, _reason| {
                if let Some(on_select) = on_select.as_ref() {
                    on_select(host, action_cx, path.clone());
                }
                host.notify(action_cx);
                host.request_redraw(action_cx.window);
            })
        });

        let mut pressable = PressableProps::default();
        pressable.enabled = true;
        pressable.a11y = PressableA11y {
            role: Some(SemanticsRole::TreeItem),
            label: Some(file.name.clone()),
            selected: is_selected,
            test_id: row_test_id.clone(),
            ..Default::default()
        };

        let bg = if is_selected {
            Some(selected_bg)
        } else if st.hovered {
            Some(hover_bg)
        } else {
            None
        };

        let mut chrome = ContainerProps::default();
        chrome.padding = Edges {
            top: MetricRef::space(Space::N1).resolve(theme),
            right: MetricRef::space(Space::N2).resolve(theme),
            bottom: MetricRef::space(Space::N1).resolve(theme),
            left: MetricRef::space(Space::N2).resolve(theme),
        };
        chrome.background = bg;
        chrome.corner_radii = fret_core::Corners::all(MetricRef::radius(Radius::Sm).resolve(theme));
        chrome.border = Edges::all(Px(0.0));
        chrome.layout =
            decl_style::layout_style(theme, LayoutRefinement::default().w_full().min_w_0());

        let spacer = cx.container(
            ContainerProps {
                layout: {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Px(Px(16.0));
                    layout.size.height = Length::Px(Px(16.0));
                    layout
                },
                ..Default::default()
            },
            |_cx| Vec::new(),
        );

        let icon = FileTreeIcon::new(file.icon.unwrap_or(ids::ui::FILE))
            .color(ColorRef::Color(resolve_muted_fg(theme)))
            .into_element(cx);
        let name = FileTreeName::new(file.name.clone()).into_element(cx);

        let row_contents = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .gap(Space::N1)
                .items(Items::Center),
            move |_cx| vec![spacer, icon, name],
        );

        (pressable, chrome, move |_cx| vec![row_contents])
    })
}
