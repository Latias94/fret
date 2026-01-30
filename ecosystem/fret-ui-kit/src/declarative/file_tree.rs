use std::collections::HashMap;
use std::sync::Arc;

use fret_core::{Color, Edges, Px, SemanticsRole};
use fret_runtime::Model;
use fret_ui::element::{AnyElement, LayoutStyle, Length, PressableA11y, PressableProps};
use fret_ui::scroll::{ScrollStrategy, VirtualListScrollHandle};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::declarative::CachedSubtreeExt as _;
use crate::declarative::model_watch::ModelWatchExt as _;
use crate::declarative::style as decl_style;
use crate::declarative::{CachedSubtreeProps, stack};
use crate::style::{ChromeRefinement, LayoutRefinement};
use crate::{ColorRef, MetricRef, Space, TreeEntry, TreeItem, TreeItemId, TreeState, flatten_tree};

fn resolve_list_colors(theme: &Theme) -> (Color, Color, Color) {
    let list_bg = theme
        .color_by_key("list.background")
        .or_else(|| theme.color_by_key("card"))
        .unwrap_or_else(|| theme.color_required("card"));
    let row_hover = theme
        .color_by_key("list.hover.background")
        .or_else(|| theme.color_by_key("list.row.hover"))
        .or_else(|| theme.color_by_key("accent"))
        .unwrap_or_else(|| theme.color_required("accent"));
    let row_active = theme
        .color_by_key("list.active.background")
        .or_else(|| theme.color_by_key("list.row.active"))
        .or_else(|| theme.color_by_key("accent"))
        .unwrap_or_else(|| theme.color_required("accent"));
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
            debug_root_test_id: None,
            debug_row_test_id_prefix: None,
        }
    }
}

#[derive(Default)]
struct FileTreeRowsState {
    last_items_revision: Option<u64>,
    last_state_revision: Option<u64>,
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

pub fn file_tree_view_retained_v0<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    items: Model<Vec<TreeItem>>,
    state: Model<TreeState>,
    scroll: &VirtualListScrollHandle,
    props: FileTreeViewProps,
) -> AnyElement
where
    H: 'static,
{
    let theme = Arc::new(Theme::global(&*cx.app).clone());

    let items_revision = cx.app.models().revision(&items).unwrap_or(0);
    let state_revision = cx.app.models().revision(&state).unwrap_or(0);
    let TreeState { selected, expanded } =
        cx.watch_model(&state).paint().cloned().unwrap_or_default();
    let items_value = cx.watch_model(&items).layout().cloned().unwrap_or_default();

    let (list_bg, row_hover, row_active) = resolve_list_colors(theme.as_ref());
    let row_h = resolve_row_height(theme.as_ref(), props.row_height);
    let row_px = resolve_row_padding_x(theme.as_ref());
    let row_py = resolve_row_padding_y(theme.as_ref());
    let indent = resolve_indent(theme.as_ref());

    let (entries, index_by_id): (Arc<Vec<TreeEntry>>, HashMap<TreeItemId, usize>) =
        cx.with_state(FileTreeRowsState::default, |rows_state| {
            if rows_state.last_items_revision != Some(items_revision)
                || rows_state.last_state_revision != Some(state_revision)
            {
                rows_state.last_items_revision = Some(items_revision);
                rows_state.last_state_revision = Some(state_revision);
                let (entries, index_by_id) = rebuild_entries(items_value, &expanded);
                rows_state.entries = entries;
                rows_state.index_by_id = index_by_id;
            }

            (
                Arc::new(rows_state.entries.clone()),
                rows_state.index_by_id.clone(),
            )
        });
    if let Some(id) = selected
        && let Some(idx) = index_by_id.get(&id).copied()
    {
        scroll.scroll_to_item(idx, ScrollStrategy::Nearest);
    }

    let state_for_row = state.clone();
    let entries_for_row = Arc::clone(&entries);

    let mut options =
        fret_ui::element::VirtualListOptions::known(row_h, props.overscan as usize, move |_i| {
            row_h
        });
    // VirtualList windowing should react to entry-list changes (expand/collapse + tree updates).
    // We conservatively fold both model revisions into the virtualizer revision.
    options.items_revision = items_revision ^ state_revision.rotate_left(1);

    let expanded_for_row = expanded.clone();
    let selected_for_row = selected;
    let row_test_id_prefix = props.debug_row_test_id_prefix.clone();
    let row = move |cx: &mut ElementContext<'_, H>, i: usize| {
        let Some(entry) = entries_for_row.get(i).cloned() else {
            return cx.text("");
        };

        let is_selected = selected_for_row == Some(entry.id);
        let is_expanded = entry.has_children && expanded_for_row.contains(&entry.id);

        let debug_test_id: Option<Arc<str>> = row_test_id_prefix
            .as_ref()
            .map(|prefix| Arc::from(format!("{prefix}-{}", entry.id)));

        let enabled = !entry.disabled;
        let pad_left = Px(row_px.0 + indent.0 * (entry.depth as f32).max(0.0));
        let theme = Arc::clone(&theme);
        let state_for_row = state_for_row.clone();

        cx.pressable(
            PressableProps {
                enabled,
                a11y: PressableA11y {
                    role: Some(SemanticsRole::TreeItem),
                    label: Some(entry.label.clone()),
                    selected: is_selected,
                    test_id: debug_test_id,
                    ..Default::default()
                },
                ..Default::default()
            },
            move |cx, st| {
                let row_id = entry.id;
                let row_has_children = entry.has_children;
                let state_for_activate = state_for_row.clone();
                cx.pressable_add_on_activate(Arc::new(move |host, action_cx, _reason| {
                    let _ = host.models_mut().update(&state_for_activate, |st| {
                        st.selected = Some(row_id);
                        if row_has_children {
                            if !st.expanded.insert(row_id) {
                                st.expanded.remove(&row_id);
                            }
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

                let mut row_props = decl_style::container_props(
                    theme.as_ref(),
                    ChromeRefinement::default().bg(ColorRef::Color(background)),
                    LayoutRefinement::default()
                        .w_full()
                        .h_px(MetricRef::Px(row_h)),
                );
                row_props.layout.overflow = fret_ui::element::Overflow::Clip;
                row_props.padding = Edges {
                    top: row_py,
                    right: row_px,
                    bottom: row_py,
                    left: pad_left,
                };

                vec![cx.container(row_props, |cx| {
                    vec![stack::hstack(
                        cx,
                        stack::HStackProps::default()
                            .layout(LayoutRefinement::default().w_full().h_full())
                            .gap(Space::N2)
                            .items_center(),
                        |cx| vec![cx.text(icon), cx.text(entry.label.as_ref())],
                    )]
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
        layout.clone(),
        entries.len(),
        options,
        scroll,
        key_at,
        row,
    );

    let mut semantics = fret_ui::element::SemanticsProps::default();
    semantics.role = fret_core::SemanticsRole::List;
    semantics.layout = layout;
    semantics.test_id = props.debug_root_test_id.clone();

    let list = cx.semantics(semantics, |_cx| vec![list]);

    // Keep a cache root boundary so the file-tree surface can be adopted as a panel-level unit.
    // Consumers can still wrap this in their own cache roots if needed.
    cx.cached_subtree_with(
        CachedSubtreeProps::default().contained_layout(true),
        |_cx| vec![list],
    )
}
