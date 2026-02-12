use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use fret_core::{
    Color, Edges, FontId, FontWeight, Point, Px, SemanticsRole, TextOverflow, TextStyle, TextWrap,
    Transform2D,
};
use fret_icons::ids;
use fret_runtime::Model;
use fret_ui::action::{ActionCx, UiActionHost};
use fret_ui::element::{
    AnyElement, ContainerProps, LayoutStyle, Length, PressableA11y, PressableProps,
    VisualTransformProps,
};
use fret_ui::scroll::VirtualListScrollHandle;
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::chrome::control_chrome_pressable_with_id_props;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{
    ChromeRefinement, ColorFallback, ColorRef, Items, LayoutRefinement, MetricRef, Radius, Space,
};

use crate::model::item_key_from_salted_external_id;

pub type OnFileTreeSelect = Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, Arc<str>) + 'static>;
pub type OnFileTreeExpandedChange =
    Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, Arc<[Arc<str>]>) + 'static>;
pub type OnFileTreeActionActivate =
    Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, Arc<str>) + 'static>;

const FILE_TREE_ITEM_SALT: u64 = 0x6f2b_53d0_8a62_0d11;

fn file_tree_item_id(path: &str) -> fret_ui_kit::TreeItemId {
    item_key_from_salted_external_id(FILE_TREE_ITEM_SALT, path)
}

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
enum FileTreeRowPayload {
    Folder {
        path: Arc<str>,
        name: Arc<str>,
        actions: Vec<FileTreeAction>,
        test_id: Option<Arc<str>>,
    },
    File {
        path: Arc<str>,
        name: Arc<str>,
        icon: Option<fret_icons::IconId>,
        actions: Vec<FileTreeAction>,
        test_id: Option<Arc<str>>,
    },
}

fn build_tree_items_and_rows(
    items: &[FileTreeItem],
    rows_by_id: &mut HashMap<fret_ui_kit::TreeItemId, FileTreeRowPayload>,
) -> Vec<fret_ui_kit::TreeItem> {
    items
        .iter()
        .map(|item| match item {
            FileTreeItem::Folder(folder) => {
                let id = file_tree_item_id(folder.path.as_ref());
                rows_by_id.insert(
                    id,
                    FileTreeRowPayload::Folder {
                        path: folder.path.clone(),
                        name: folder.name.clone(),
                        actions: folder.actions.clone(),
                        test_id: folder.test_id.clone(),
                    },
                );
                let children = build_tree_items_and_rows(&folder.children, rows_by_id);
                fret_ui_kit::TreeItem::new(id, folder.name.clone()).children(children)
            }
            FileTreeItem::File(file) => {
                let id = file_tree_item_id(file.path.as_ref());
                rows_by_id.insert(
                    id,
                    FileTreeRowPayload::File {
                        path: file.path.clone(),
                        name: file.name.clone(),
                        icon: file.icon.clone(),
                        actions: file.actions.clone(),
                        test_id: file.test_id.clone(),
                    },
                );
                fret_ui_kit::TreeItem::new(id, file.name.clone())
            }
        })
        .collect()
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
/// - This surface uses a flattened list representation so it can be virtualized under height
///   constraints (e.g. when hosted inside a panel).
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
        let theme = Arc::new(Theme::global(&*cx.app).clone());

        let expanded_model = self.resolve_expanded_model(cx);
        let expanded: HashSet<Arc<str>> = cx
            .watch_model(&expanded_model)
            .layout()
            .cloned()
            .unwrap_or_default();
        let expanded_ids: HashSet<fret_ui_kit::TreeItemId> = expanded
            .iter()
            .map(|p| file_tree_item_id(p.as_ref()))
            .collect();

        let chrome = ChromeRefinement::default()
            .p(Space::N2)
            .rounded(Radius::Lg)
            .border_1()
            .bg(ColorRef::Color(resolve_background(theme.as_ref())))
            .border_color(ColorRef::Color(resolve_border(theme.as_ref())))
            .merge(self.chrome);
        let layout = LayoutRefinement::default().min_w_0().merge(self.layout);
        let has_height_constraint = layout
            .size
            .as_ref()
            .map(|s| s.height.is_some() || s.max_height.is_some())
            .unwrap_or(false);

        let props = decl_style::container_props(theme.as_ref(), chrome, layout);
        let selected_path = self.selected_path;
        let on_select = self.on_select;
        let on_expanded_change = self.on_expanded_change;
        let items = self.items;
        let test_id_root = self.test_id_root;

        let mut rows_by_id: HashMap<fret_ui_kit::TreeItemId, FileTreeRowPayload> = HashMap::new();
        let tree_items = build_tree_items_and_rows(&items, &mut rows_by_id);
        let entries: Arc<Vec<fret_ui_kit::TreeEntry>> =
            Arc::new(fret_ui_kit::flatten_tree(&tree_items, &expanded_ids));
        let rows_by_id: Arc<HashMap<fret_ui_kit::TreeItemId, FileTreeRowPayload>> =
            Arc::new(rows_by_id);

        let scroll = cx.with_state(VirtualListScrollHandle::new, |h| h.clone());

        let row_height = theme
            .metric_by_key("fret.ai.file_tree.row_height")
            .unwrap_or(Px(26.0));
        let row_height = Px(row_height.0.max(0.0));

        let overscan = theme
            .metric_by_key("fret.ai.file_tree.overscan")
            .map(|v| v.0.round().max(0.0) as usize)
            .unwrap_or(12);

        let mut options = fret_ui::element::VirtualListOptions::fixed(row_height, overscan)
            .keep_alive(overscan.saturating_mul(2));
        options.items_revision = cx.app.models().revision(&expanded_model).unwrap_or(0);
        if entries.len() > 10_000 {
            options.key_cache = fret_ui::element::VirtualListKeyCacheMode::VisibleOnly;
        }

        let list_layout = LayoutStyle {
            size: fret_ui::element::SizeStyle {
                width: Length::Fill,
                height: if has_height_constraint {
                    Length::Fill
                } else {
                    Length::Auto
                },
                ..Default::default()
            },
            overflow: fret_ui::element::Overflow::Clip,
            ..Default::default()
        };

        let expanded_snapshot: Arc<HashSet<Arc<str>>> = Arc::new(expanded);

        let row: Arc<dyn for<'a> Fn(&mut ElementContext<'a, H>, usize) -> AnyElement> = Arc::new({
            let theme = Arc::clone(&theme);
            let entries = Arc::clone(&entries);
            let rows_by_id = Arc::clone(&rows_by_id);
            let expanded_model = expanded_model.clone();
            let expanded_snapshot = Arc::clone(&expanded_snapshot);
            move |cx, index| {
                let Some(entry) = entries.get(index) else {
                    return cx.text("");
                };
                let Some(payload) = rows_by_id.get(&entry.id).cloned() else {
                    return cx.text("");
                };

                match payload {
                    FileTreeRowPayload::Folder {
                        path,
                        name,
                        actions,
                        test_id,
                    } => render_folder_row(
                        cx,
                        theme.as_ref(),
                        row_height,
                        entry.depth,
                        &expanded_model,
                        expanded_snapshot.as_ref(),
                        selected_path.as_ref(),
                        on_select.as_ref(),
                        on_expanded_change.as_ref(),
                        FileTreeFolder {
                            path,
                            name,
                            children: Vec::new(),
                            actions,
                            test_id,
                        },
                    ),
                    FileTreeRowPayload::File {
                        path,
                        name,
                        icon,
                        actions,
                        test_id,
                    } => render_file_row(
                        cx,
                        theme.as_ref(),
                        row_height,
                        entry.depth,
                        selected_path.as_ref(),
                        on_select.as_ref(),
                        FileTreeFile {
                            path,
                            name,
                            icon,
                            actions,
                            test_id,
                        },
                    ),
                }
            }
        });

        let key_at: Arc<dyn Fn(usize) -> fret_ui::ItemKey> = Arc::new({
            let entries: Arc<Vec<fret_ui_kit::TreeEntry>> = Arc::clone(&entries);
            move |i: usize| -> fret_ui::ItemKey { entries.get(i).map(|e| e.id).unwrap_or_default() }
        });

        let list = cx.virtual_list_keyed_retained_with_layout(
            list_layout,
            entries.len(),
            options,
            &scroll,
            key_at,
            row,
        );

        let tree = cx.container(props, move |_cx| vec![list]);

        tree.attach_semantics(fret_ui::element::SemanticsDecoration {
            // Fret currently does not model a distinct `Tree` role at the contract layer; we
            // expose the surface as a list root with `TreeItem` children (consistent with
            // `fret-ui-kit` file tree helpers).
            role: Some(SemanticsRole::List),
            test_id: test_id_root,
            ..Default::default()
        })
    }
}

#[derive(Debug, Clone)]
pub struct FileTreeFolder {
    pub path: Arc<str>,
    pub name: Arc<str>,
    pub children: Vec<FileTreeItem>,
    pub actions: Vec<FileTreeAction>,
    pub test_id: Option<Arc<str>>,
}

impl FileTreeFolder {
    pub fn new(path: impl Into<Arc<str>>, name: impl Into<Arc<str>>) -> Self {
        Self {
            path: path.into(),
            name: name.into(),
            children: Vec::new(),
            actions: Vec::new(),
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

    pub fn action(mut self, action: FileTreeAction) -> Self {
        self.actions.push(action);
        self
    }

    pub fn actions(mut self, actions: impl IntoIterator<Item = FileTreeAction>) -> Self {
        self.actions = actions.into_iter().collect();
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
    pub actions: Vec<FileTreeAction>,
    pub test_id: Option<Arc<str>>,
}

impl FileTreeFile {
    pub fn new(path: impl Into<Arc<str>>, name: impl Into<Arc<str>>) -> Self {
        Self {
            path: path.into(),
            name: name.into(),
            icon: None,
            actions: Vec::new(),
            test_id: None,
        }
    }

    pub fn icon(mut self, icon: fret_icons::IconId) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn action(mut self, action: FileTreeAction) -> Self {
        self.actions.push(action);
        self
    }

    pub fn actions(mut self, actions: impl IntoIterator<Item = FileTreeAction>) -> Self {
        self.actions = actions.into_iter().collect();
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }
}

#[derive(Clone)]
pub struct FileTreeAction {
    icon: fret_icons::IconId,
    label: Arc<str>,
    on_activate: OnFileTreeActionActivate,
    disabled: bool,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for FileTreeAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileTreeAction")
            .field("icon", &self.icon)
            .field("label", &self.label)
            .field("disabled", &self.disabled)
            .field("test_id", &self.test_id.as_deref())
            .finish()
    }
}

impl FileTreeAction {
    pub fn new(
        icon: fret_icons::IconId,
        label: impl Into<Arc<str>>,
        on_activate: OnFileTreeActionActivate,
    ) -> Self {
        Self {
            icon,
            label: label.into(),
            on_activate,
            disabled: false,
            test_id: None,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
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
        let theme = Theme::global(&*cx.app).clone();

        cx.text_props(fret_ui::element::TextProps {
            layout: fret_ui::element::LayoutStyle {
                size: fret_ui::element::SizeStyle {
                    width: fret_ui::element::Length::Fill,
                    height: fret_ui::element::Length::Auto,
                    min_width: Some(Px(0.0)),
                    ..Default::default()
                },
                flex: fret_ui::element::FlexItemStyle {
                    grow: 1.0,
                    shrink: 1.0,
                    basis: fret_ui::element::Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            text: self.name,
            style: Some(TextStyle {
                font: FontId::monospace(),
                size: theme.metric_required("metric.font.mono_size"),
                weight: FontWeight::NORMAL,
                slant: Default::default(),
                line_height: Some(theme.metric_required("metric.font.mono_line_height")),
                letter_spacing_em: None,
            }),
            color: Some(theme.color_required("foreground")),
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
        })
    }
}

fn render_actions<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    row_path: Arc<str>,
    actions: Vec<FileTreeAction>,
) -> Option<AnyElement> {
    if actions.is_empty() {
        return None;
    }

    let muted = resolve_muted(theme);
    let hover_bg = alpha(muted, 0.5);
    let icon_fg = resolve_muted_fg(theme);

    let group = cx.container(
        ContainerProps {
            layout: decl_style::layout_style(
                theme,
                LayoutRefinement::default().ml_auto().flex_shrink_0(),
            ),
            ..Default::default()
        },
        move |cx| {
            let buttons = actions
                .into_iter()
                .enumerate()
                .map(|(i, action)| {
                    let key: Arc<str> = Arc::from(format!("action-{i}"));
                    let row_path = row_path.clone();
                    cx.keyed(key, |cx| {
                        let label = action.label.clone();
                        let icon = action.icon;
                        let disabled = action.disabled;
                        let test_id = action.test_id.clone();
                        let on_activate = action.on_activate.clone();
                        let row_path = row_path.clone();

                        control_chrome_pressable_with_id_props(cx, move |cx, st, _id| {
                            cx.pressable_on_activate(Arc::new(move |host, action_cx, _reason| {
                                on_activate(host, action_cx, row_path.clone());
                                host.notify(action_cx);
                                host.request_redraw(action_cx.window);
                            }));

                            let mut pressable = PressableProps::default();
                            pressable.enabled = !disabled;
                            pressable.a11y = PressableA11y {
                                role: Some(SemanticsRole::Button),
                                label: Some(label.clone()),
                                test_id: test_id.clone(),
                                ..Default::default()
                            };

                            let bg = if disabled {
                                None
                            } else if st.hovered || st.pressed {
                                Some(hover_bg)
                            } else {
                                None
                            };

                            let mut chrome = ContainerProps::default();
                            chrome.layout = decl_style::layout_style(
                                theme,
                                LayoutRefinement::default()
                                    .w_px(MetricRef::Px(Px(20.0)))
                                    .h_px(MetricRef::Px(Px(20.0)))
                                    .flex_shrink_0(),
                            );
                            chrome.background = bg;
                            chrome.corner_radii = fret_core::Corners::all(
                                MetricRef::radius(Radius::Sm).resolve(theme),
                            );
                            chrome.border = Edges::all(Px(0.0));
                            chrome.padding = Edges::all(Px(2.0));

                            let icon = decl_icon::icon_with(
                                cx,
                                icon,
                                Some(Px(16.0)),
                                Some(ColorRef::Color(icon_fg)),
                            );

                            (pressable, chrome, move |_cx| vec![icon])
                        })
                    })
                })
                .collect::<Vec<_>>();

            vec![stack::hstack(
                cx,
                stack::HStackProps::default()
                    .layout(LayoutRefinement::default().flex_shrink_0())
                    .gap(Space::N1)
                    .items(Items::Center),
                move |_cx| buttons,
            )]
        },
    );

    Some(group)
}

fn file_tree_indent_el<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    row_height: Px,
    depth: usize,
) -> AnyElement {
    let indent_ml = MetricRef::space(Space::N4).resolve(theme);
    let indent_pl = MetricRef::space(Space::N2).resolve(theme);
    let pad_x = MetricRef::space(Space::N2).resolve(theme);
    let border = resolve_border(theme);

    let mut segments: Vec<AnyElement> = Vec::new();
    for _ in 0..depth {
        let spacer_ml = cx.container(
            ContainerProps {
                layout: LayoutStyle {
                    size: fret_ui::element::SizeStyle {
                        width: Length::Px(indent_ml),
                        height: Length::Px(row_height),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            |_cx| Vec::new(),
        );
        let border_line = cx.container(
            ContainerProps {
                layout: LayoutStyle {
                    size: fret_ui::element::SizeStyle {
                        width: Length::Px(Px(1.0)),
                        height: Length::Px(row_height),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                background: Some(border),
                ..Default::default()
            },
            |_cx| Vec::new(),
        );
        let spacer_pl = cx.container(
            ContainerProps {
                layout: LayoutStyle {
                    size: fret_ui::element::SizeStyle {
                        width: Length::Px(indent_pl),
                        height: Length::Px(row_height),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            |_cx| Vec::new(),
        );
        segments.push(spacer_ml);
        segments.push(border_line);
        segments.push(spacer_pl);
    }

    let pad = cx.container(
        ContainerProps {
            layout: LayoutStyle {
                size: fret_ui::element::SizeStyle {
                    width: Length::Px(pad_x),
                    height: Length::Px(row_height),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        },
        |_cx| Vec::new(),
    );
    segments.push(pad);

    stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(
                LayoutRefinement::default()
                    .h_px(MetricRef::Px(row_height))
                    .flex_shrink_0(),
            )
            .gap(Space::N0)
            .items(Items::Center),
        move |_cx| segments,
    )
}

fn render_folder_row<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    row_height: Px,
    depth: usize,
    expanded_model: &Model<HashSet<Arc<str>>>,
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
            Arc::new(move |host, action_cx, _reason| {
                if let Some(on_select) = on_select.as_ref() {
                    on_select(host, action_cx, path.clone());
                }

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
            left: Px(0.0),
        };
        chrome.background = bg;
        chrome.corner_radii = fret_core::Corners::all(MetricRef::radius(Radius::Sm).resolve(theme));
        chrome.border = Edges::all(Px(0.0));
        chrome.layout =
            decl_style::layout_style(theme, LayoutRefinement::default().w_full().min_w_0());
        chrome.layout.size.height = Length::Px(row_height);

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
        let actions = render_actions(cx, theme, folder.path.clone(), folder.actions.clone());

        let row_contents = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .gap(Space::N1)
                .items(Items::Center),
            move |cx| {
                let indent = file_tree_indent_el(cx, theme, row_height, depth);
                let mut out = vec![indent, chevron, folder_icon, name];
                if let Some(actions) = actions.clone() {
                    out.push(actions);
                }
                out
            },
        );

        (pressable, chrome, move |_cx| vec![row_contents])
    });

    row
}

fn render_file_row<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    row_height: Px,
    depth: usize,
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
            left: Px(0.0),
        };
        chrome.background = bg;
        chrome.corner_radii = fret_core::Corners::all(MetricRef::radius(Radius::Sm).resolve(theme));
        chrome.border = Edges::all(Px(0.0));
        chrome.layout =
            decl_style::layout_style(theme, LayoutRefinement::default().w_full().min_w_0());
        chrome.layout.size.height = Length::Px(row_height);

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
        let actions = render_actions(cx, theme, file.path.clone(), file.actions.clone());

        let row_contents = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .gap(Space::N1)
                .items(Items::Center),
            move |cx| {
                let indent = file_tree_indent_el(cx, theme, row_height, depth);
                let mut out = vec![indent, spacer, icon, name];
                if let Some(actions) = actions.clone() {
                    out.push(actions);
                }
                out
            },
        );

        (pressable, chrome, move |_cx| vec![row_contents])
    })
}
