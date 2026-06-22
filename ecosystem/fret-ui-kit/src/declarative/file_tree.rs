use std::collections::HashMap;
use std::sync::Arc;

use fret_core::{AttributedText, Color, Edges, Px, SemanticsRole, TextSpan};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, CrossAlign, FlexProps, LayoutStyle, Length, Overflow, PressableA11y,
    PressableProps, SemanticsDecoration, SpacingLength,
};
use fret_ui::scroll::{ScrollStrategy, VirtualListScrollHandle};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::declarative::CachedSubtreeExt as _;
use crate::declarative::CachedSubtreeProps;
use crate::declarative::model_watch::ModelWatchExt as _;
use crate::declarative::style as decl_style;
use crate::style::{ChromeRefinement, LayoutRefinement};
use crate::{ColorRef, MetricRef, Space, TreeEntry, TreeItem, TreeItemId, TreeState, flatten_tree};

fn resolve_list_colors(theme: &Theme) -> (Color, Color, Color) {
    let list_bg = theme
        .color_by_key("list.background")
        .or_else(|| theme.color_by_key("card"))
        .unwrap_or_else(|| theme.color_token("card"));
    let row_hover = theme
        .color_by_key("list.hover.background")
        .or_else(|| theme.color_by_key("list.row.hover"))
        .or_else(|| theme.color_by_key("accent"))
        .unwrap_or_else(|| theme.color_token("accent"));
    let row_active = theme
        .color_by_key("list.active.background")
        .or_else(|| theme.color_by_key("list.row.active"))
        .or_else(|| theme.color_by_key("accent"))
        .unwrap_or_else(|| theme.color_token("accent"));
    (list_bg, row_hover, row_active)
}

fn resolve_row_height(theme: &Theme, default: Px) -> Px {
    let base = theme
        .metric_by_key("component.list.row_height")
        .unwrap_or(default);
    Px(base.0.max(0.0))
}

fn resolve_row_padding_x(theme: &Theme) -> Px {
    MetricRef::space(Space::N2p5).resolve(theme)
}

fn resolve_row_padding_y(theme: &Theme) -> Px {
    MetricRef::space(Space::N1p5).resolve(theme)
}

fn resolve_indent(theme: &Theme) -> Px {
    MetricRef::space(Space::N4).resolve(theme)
}

/// A retained-host, cache-root friendly file-tree list helper.
///
/// This is a pragmatic "workspace surface" building block:
/// - row identity is `TreeItemId`,
/// - click selects, and folders also toggle expansion on click,
/// - virtualization uses the virt-003 retained host path (so overscan boundary updates can
///   attach/detach without rerendering a parent cache root).
///
/// `debug_row_test_id_prefix` is intended for scripted harnesses (e.g. UI Gallery torture pages).
#[derive(Debug, Clone)]
pub struct FileTreeViewProps {
    pub layout: LayoutStyle,
    pub row_height: Px,
    pub overscan: u32,
    /// Optional retained-subtree budget for `VirtualList` window shifts.
    ///
    /// When set, this overrides the default heuristic (`overscan * 2`). Larger values reduce
    /// remount/layout churn at window boundaries, at the cost of retaining more offscreen
    /// subtrees.
    pub keep_alive: Option<usize>,
    pub debug_root_test_id: Option<Arc<str>>,
    pub debug_row_test_id_prefix: Option<Arc<str>>,
}

impl Default for FileTreeViewProps {
    fn default() -> Self {
        Self {
            layout: LayoutStyle {
                size: fret_ui::element::SizeStyle {
                    width: Length::Fill,
                    height: Length::Px(Px(460.0)),
                    ..Default::default()
                },
                overflow: fret_ui::element::Overflow::Clip,
                ..Default::default()
            },
            row_height: Px(26.0),
            overscan: 12,
            keep_alive: None,
            debug_root_test_id: None,
            debug_row_test_id_prefix: None,
        }
    }
}

#[derive(Default)]
struct FileTreeRowsState {
    last_items_revision: Option<u64>,
    last_state_revision: Option<u64>,
    last_scrolled_selected: Option<TreeItemId>,
    last_scrolled_index: Option<usize>,
    last_scrolled_items_revision: Option<u64>,
    last_scrolled_state_revision: Option<u64>,
    entries: Vec<TreeEntry>,
    index_by_id: HashMap<TreeItemId, usize>,
}

fn rebuild_entries(
    items: Vec<TreeItem>,
    expanded: &std::collections::HashSet<TreeItemId>,
) -> (Vec<TreeEntry>, HashMap<TreeItemId, usize>) {
    let entries = flatten_tree(&items, expanded);
    let index_by_id: HashMap<TreeItemId, usize> =
        entries.iter().enumerate().map(|(i, e)| (e.id, i)).collect();
    (entries, index_by_id)
}

fn file_tree_item_a11y(
    entry: &TreeEntry,
    is_selected: bool,
    is_expanded: bool,
    test_id: Option<Arc<str>>,
) -> PressableA11y {
    PressableA11y {
        role: Some(SemanticsRole::TreeItem),
        label: Some(entry.label.clone()),
        level: u32::try_from(entry.depth.saturating_add(1)).ok(),
        selected: is_selected,
        expanded: entry.has_children.then_some(is_expanded),
        test_id,
        ..Default::default()
    }
}

fn file_tree_row_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    icon: impl Into<Arc<str>>,
    label: impl Into<Arc<str>>,
) -> AnyElement {
    let icon = icon.into();
    let label = label.into();
    let mut text = String::with_capacity(icon.len() + 1 + label.len());
    text.push_str(&icon);
    text.push(' ');
    text.push_str(&label);

    let text_len = text.len();
    let rich = AttributedText::new(
        Arc::<str>::from(text),
        Arc::<[TextSpan]>::from([TextSpan::new(text_len)]),
    );
    crate::declarative::text::text_list_row_label_attributed(cx, rich)
}

fn file_tree_missing_virtual_row_placeholder<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> AnyElement {
    cx.spacer(fret_ui::element::SpacerProps::default())
}

fn file_tree_retained_row_layout(row_h: Px) -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Px(row_h);
    layout.overflow = Overflow::Clip;
    layout
}

fn file_tree_row_content_props(theme: &Theme) -> FlexProps {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Fill;

    FlexProps {
        layout,
        gap: SpacingLength::Px(MetricRef::space(Space::N2).resolve(theme)),
        align: CrossAlign::Center,
        ..FlexProps::default()
    }
}

#[track_caller]
pub fn file_tree_view_retained_v0<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    items: Model<Vec<TreeItem>>,
    state: Model<TreeState>,
    scroll: &VirtualListScrollHandle,
    props: FileTreeViewProps,
) -> AnyElement {
    let items_revision = cx.app.models().revision(&items).unwrap_or(0);
    let state_revision = cx.app.models().revision(&state).unwrap_or(0);
    let TreeState { selected, expanded } = cx.watch_model(&state).paint().cloned_or_default();
    let items_value = cx.watch_model(&items).layout().cloned_or_default();

    let (list_bg, row_hover, row_active, row_h, row_px, row_py, indent) = {
        let theme = Theme::global(&*cx.app);
        let (list_bg, row_hover, row_active) = resolve_list_colors(theme);
        let row_h = resolve_row_height(theme, props.row_height);
        let row_px = resolve_row_padding_x(theme);
        let row_py = resolve_row_padding_y(theme);
        let indent = resolve_indent(theme);
        (
            list_bg, row_hover, row_active, row_h, row_px, row_py, indent,
        )
    };

    let entries: Arc<Vec<TreeEntry>> = cx.slot_state(FileTreeRowsState::default, |rows_state| {
        if rows_state.last_items_revision != Some(items_revision)
            || rows_state.last_state_revision != Some(state_revision)
        {
            rows_state.last_items_revision = Some(items_revision);
            rows_state.last_state_revision = Some(state_revision);
            let (entries, index_by_id) = rebuild_entries(items_value, &expanded);
            rows_state.entries = entries;
            rows_state.index_by_id = index_by_id;
        }

        let selected_idx = selected.and_then(|id| rows_state.index_by_id.get(&id).copied());
        if let Some(selected_id) = selected
            && let Some(idx) = selected_idx
        {
            let should_scroll = rows_state.last_scrolled_selected != Some(selected_id)
                || rows_state.last_scrolled_index != Some(idx)
                || rows_state.last_scrolled_items_revision != Some(items_revision)
                || rows_state.last_scrolled_state_revision != Some(state_revision);
            if should_scroll {
                scroll.scroll_to_item(idx, ScrollStrategy::Nearest);
                rows_state.last_scrolled_selected = Some(selected_id);
                rows_state.last_scrolled_index = Some(idx);
                rows_state.last_scrolled_items_revision = Some(items_revision);
                rows_state.last_scrolled_state_revision = Some(state_revision);
            }
        } else {
            rows_state.last_scrolled_selected = None;
            rows_state.last_scrolled_index = None;
            rows_state.last_scrolled_items_revision = Some(items_revision);
            rows_state.last_scrolled_state_revision = Some(state_revision);
        }

        Arc::new(rows_state.entries.clone())
    });

    let state_for_row = state.clone();
    let entries_for_row = Arc::clone(&entries);

    let mut options =
        fret_ui::element::VirtualListOptions::known(row_h, props.overscan as usize, move |_i| {
            row_h
        });
    // VirtualList windowing should react to entry-list changes (expand/collapse + tree updates).
    // We conservatively fold both model revisions into the virtualizer revision.
    options.items_revision = items_revision ^ state_revision.rotate_left(1);
    options.keep_alive = props
        .keep_alive
        .unwrap_or_else(|| (props.overscan as usize).saturating_mul(2));

    let expanded_for_row = expanded.clone();
    let selected_for_row = selected;
    let row_test_id_prefix = props.debug_row_test_id_prefix.clone();
    let row = move |cx: &mut ElementContext<'_, H>, i: usize| {
        let Some(entry) = entries_for_row.get(i).cloned() else {
            return file_tree_missing_virtual_row_placeholder(cx);
        };

        let is_selected = selected_for_row == Some(entry.id);
        let is_expanded = entry.has_children && expanded_for_row.contains(&entry.id);

        let debug_test_id: Option<Arc<str>> = row_test_id_prefix
            .as_ref()
            .map(|prefix| Arc::from(format!("{prefix}-{}", entry.id)));

        let enabled = !entry.disabled;
        let pad_left = Px(row_px.0 + indent.0 * (entry.depth as f32).max(0.0));
        let state_for_row = state_for_row.clone();

        cx.pressable(
            PressableProps {
                layout: file_tree_retained_row_layout(row_h),
                enabled,
                a11y: file_tree_item_a11y(&entry, is_selected, is_expanded, debug_test_id),
                ..Default::default()
            },
            move |cx, st| {
                let row_id = entry.id;
                let row_has_children = entry.has_children;
                let state_for_activate = state_for_row.clone();
                cx.pressable_add_on_activate(Arc::new(move |host, action_cx, _reason| {
                    let _ = host.models_mut().update(&state_for_activate, |st| {
                        st.selected = Some(row_id);
                        if row_has_children && !st.expanded.insert(row_id) {
                            st.expanded.remove(&row_id);
                        }
                    });

                    // Ensure at least one frame is produced even under aggressive cache reuse.
                    host.request_redraw(action_cx.window);
                }));

                let background = if is_selected {
                    row_active
                } else if enabled && (st.hovered || st.pressed) {
                    row_hover
                } else {
                    list_bg
                };

                let icon = if entry.has_children {
                    if is_expanded { "v" } else { ">" }
                } else {
                    "-"
                };

                let mut row_props = {
                    let theme = Theme::global(&*cx.app);
                    decl_style::container_props(
                        theme,
                        ChromeRefinement::default().bg(ColorRef::Color(background)),
                        LayoutRefinement::default()
                            .w_full()
                            .h_px(MetricRef::Px(row_h)),
                    )
                };
                row_props.layout.overflow = Overflow::Clip;
                row_props.padding = Edges {
                    top: row_py,
                    right: row_px,
                    bottom: row_py,
                    left: pad_left,
                }
                .into();

                let row_content_props = file_tree_row_content_props(Theme::global(&*cx.app));

                vec![cx.container(row_props, |cx| {
                    vec![cx.flex(row_content_props, |cx| {
                        vec![file_tree_row_text(cx, icon, entry.label.clone())]
                    })]
                })]
            },
        )
    };

    let key_at = {
        let entries: Arc<Vec<TreeEntry>> = Arc::clone(&entries);
        move |i: usize| -> TreeItemId {
            entries.get(i).map(|e: &TreeEntry| e.id).unwrap_or_default()
        }
    };

    let layout = props.layout;
    let list = cx.virtual_list_keyed_retained_with_layout_fn(
        layout,
        entries.len(),
        options,
        scroll,
        key_at,
        row,
    );

    let list = list.attach_semantics(SemanticsDecoration {
        role: Some(fret_core::SemanticsRole::List),
        test_id: props.debug_root_test_id.clone(),
        ..Default::default()
    });

    // Keep a cache root boundary so the file-tree surface can be adopted as a panel-level unit.
    // Consumers can still wrap this in their own cache roots if needed.
    cx.cached_subtree_with(
        CachedSubtreeProps::default().contain_layout_when_bounds_known(true),
        |_cx| vec![list],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_app::App;
    use fret_core::{
        AppWindowId, PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle,
        Point, Rect, SvgId, SvgService, TextBlobId, TextConstraints, TextInput, TextMetrics,
        TextOverflow, TextService, TextWrap,
    };
    use fret_ui::element::ElementKind;
    use fret_ui::{ThemeConfig, UiTree};

    fn test_bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(160.0), Px(40.0)),
        )
    }

    fn entry(has_children: bool, depth: usize) -> TreeEntry {
        TreeEntry {
            id: 7,
            label: Arc::from("Folder"),
            depth,
            parent: None,
            has_children,
            disabled: false,
        }
    }

    #[derive(Default)]
    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _input: &TextInput,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: fret_core::Size::new(Px(0.0), Px(0.0)),
                    baseline: Px(0.0),
                },
            )
        }

        fn release(&mut self, _blob: TextBlobId) {}
    }

    impl PathService for FakeServices {
        fn prepare(
            &mut self,
            _commands: &[PathCommand],
            _style: PathStyle,
            _constraints: PathConstraints,
        ) -> (PathId, PathMetrics) {
            (PathId::default(), PathMetrics::default())
        }

        fn release(&mut self, _path: PathId) {}
    }

    impl SvgService for FakeServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
            SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: SvgId) -> bool {
            true
        }
    }

    impl fret_core::MaterialService for FakeServices {
        fn register_material(
            &mut self,
            _desc: fret_core::MaterialDescriptor,
        ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
            Err(fret_core::MaterialRegistrationError::Unsupported)
        }

        fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
            true
        }
    }

    fn render_file_tree(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut FakeServices,
        window: AppWindowId,
        bounds: Rect,
    ) {
        let items = app.models_mut().insert(vec![TreeItem::new(
            7,
            "Long file tree row that must not paint outside a fixed row",
        )]);
        let state = app.models_mut().insert(TreeState::default());
        let scroll = VirtualListScrollHandle::new();

        for _ in 0..3 {
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "file-tree-row-clip-test",
                |cx| {
                    vec![file_tree_view_retained_v0(
                        cx,
                        items.clone(),
                        state.clone(),
                        &scroll,
                        FileTreeViewProps {
                            row_height: Px(26.0),
                            debug_row_test_id_prefix: Some(Arc::from("file-tree-row")),
                            ..FileTreeViewProps::default()
                        },
                    )]
                },
            );
            ui.set_root(root);
            ui.request_semantics_snapshot();
            ui.layout_all(app, services, bounds, 1.0);
            let mut scene = fret_core::Scene::default();
            ui.paint_all(app, services, bounds, &mut scene, 1.0);
            let next_frame = fret_runtime::FrameId(app.frame_id().0.saturating_add(1));
            app.set_frame_id(next_frame);
        }
    }

    fn semantics_node_id_by_test_id(ui: &UiTree<App>, test_id: &str) -> fret_core::NodeId {
        ui.semantics_snapshot()
            .expect("semantics snapshot")
            .nodes
            .iter()
            .find(|node| node.test_id.as_deref() == Some(test_id))
            .map(|node| node.id)
            .unwrap_or_else(|| panic!("expected semantics node with test_id {test_id:?}"))
    }

    fn only_child_with_kind(
        ui: &UiTree<App>,
        app: &mut App,
        window: AppWindowId,
        node: fret_core::NodeId,
        kind: &'static str,
    ) -> fret_core::NodeId {
        let children = ui.debug_node_children(node);
        assert_eq!(
            children.len(),
            1,
            "expected {kind} to be the only child of {node:?}, got {children:?}"
        );
        let child = children[0];
        assert_eq!(
            ui.debug_declarative_instance_kind(app, window, child),
            Some(kind),
            "unexpected child kind for {child:?}"
        );
        child
    }

    #[test]
    fn file_tree_item_a11y_sets_level_and_expanded_for_parent_rows() {
        let a11y = file_tree_item_a11y(&entry(true, 0), true, true, Some(Arc::from("file-row")));

        assert_eq!(a11y.role, Some(SemanticsRole::TreeItem));
        assert_eq!(a11y.level, Some(1));
        assert!(a11y.selected);
        assert_eq!(a11y.expanded, Some(true));
        assert_eq!(a11y.test_id.as_deref(), Some("file-row"));
    }

    #[test]
    fn file_tree_item_a11y_omits_expanded_for_leaf_rows() {
        let a11y = file_tree_item_a11y(&entry(false, 2), false, false, None);

        assert_eq!(a11y.level, Some(3));
        assert_eq!(a11y.expanded, None);
    }

    #[test]
    fn file_tree_row_text_collapses_icon_and_label_into_one_styled_text() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let element =
            fret_ui::elements::with_element_cx(&mut app, window, test_bounds(), "test", |cx| {
                file_tree_row_text(cx, ">", "Very long nested file name that should not wrap")
            });

        let ElementKind::StyledText(props) = &element.kind else {
            panic!("file tree row text should be styled text");
        };

        assert_eq!(
            props.rich.text.as_ref(),
            "> Very long nested file name that should not wrap"
        );
        assert_eq!(props.rich.spans.len(), 1);
        assert_eq!(props.rich.spans[0].len, props.rich.text.len());
        assert_eq!(props.layout.size.width, Length::Fill);
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.layout.flex.grow, 1.0);
        assert_eq!(props.layout.flex.shrink, 1.0);
        assert_eq!(props.layout.flex.basis, Length::Px(Px(0.0)));
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
    }

    #[test]
    fn missing_file_tree_virtual_row_placeholder_is_not_text() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let element =
            fret_ui::elements::with_element_cx(&mut app, window, test_bounds(), "test", |cx| {
                file_tree_missing_virtual_row_placeholder(cx)
            });

        let ElementKind::Spacer(props) = &element.kind else {
            panic!("missing file tree virtual row placeholder should be a spacer");
        };
        assert_eq!(props.min, Px(0.0));
    }

    #[test]
    fn file_tree_retained_row_layout_clips_to_row_height() {
        let layout = file_tree_retained_row_layout(Px(26.0));

        assert_eq!(layout.size.width, Length::Fill);
        assert_eq!(layout.size.height, Length::Px(Px(26.0)));
        assert_eq!(layout.overflow, Overflow::Clip);
    }

    #[test]
    fn file_tree_retained_rows_mount_as_clip_boundaries() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui = UiTree::new();
        ui.set_window(window);
        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(240.0), Px(96.0)),
        );

        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                ..ThemeConfig::default()
            });
        });

        render_file_tree(&mut ui, &mut app, &mut services, window, bounds);

        let row_node = semantics_node_id_by_test_id(&ui, "file-tree-row-7");
        let row_bounds = ui.debug_node_bounds(row_node).expect("row bounds");

        assert_eq!(
            ui.debug_declarative_instance_kind(&mut app, window, row_node),
            Some("Pressable")
        );
        assert!(
            row_bounds.size.height.0 <= 26.5,
            "retained file-tree row should keep configured row height, got {row_bounds:?}"
        );
        assert_eq!(
            ui.debug_node_clips_hit_test(row_node),
            Some(true),
            "retained file-tree row should clip oversized/wrapping row content"
        );

        let row_background = only_child_with_kind(&ui, &mut app, window, row_node, "Container");
        let row_content = only_child_with_kind(&ui, &mut app, window, row_background, "Flex");
        assert_eq!(
            ui.debug_node_children(row_content).len(),
            1,
            "retained file-tree rows should collapse icon and label under the content Flex"
        );
        let row_text = only_child_with_kind(&ui, &mut app, window, row_content, "StyledText");
        assert_eq!(
            ui.debug_node_children(row_text).len(),
            0,
            "retained file-tree row text should be a leaf"
        );
    }
}
