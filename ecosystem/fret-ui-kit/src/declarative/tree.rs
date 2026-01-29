use std::collections::HashMap;
use std::sync::Arc;

use fret_core::{Color, Corners, Edges, Px, SemanticsRole};
use fret_runtime::{CommandId, Model};
use fret_ui::element::{
    AnyElement, ContainerProps, Elements, LayoutStyle, Length, PressableA11y, PressableProps,
    SpacerProps,
};
use fret_ui::scroll::{ScrollStrategy, VirtualListScrollHandle};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::declarative::action_hooks::ActionHooksExt;
use crate::declarative::model_watch::ModelWatchExt as _;
use crate::declarative::stack;
use crate::{
    Items, Justify, MetricRef, Size, Space, TreeEntry, TreeItemId, TreeRowRenderer, TreeRowState,
    TreeState, flatten_tree,
};

fn resolve_list_colors(theme: &Theme) -> (Color, Color, Color, Color) {
    let list_bg = theme
        .color_by_key("list.background")
        .or_else(|| theme.color_by_key("card"))
        .unwrap_or_else(|| theme.color_required("card"));
    let border = theme
        .color_by_key("border")
        .or_else(|| theme.color_by_key("list.border"))
        .unwrap_or_else(|| theme.color_required("border"));
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
    (list_bg, border, row_hover, row_active)
}

fn resolve_row_height(theme: &Theme, size: Size) -> Px {
    let base = theme
        .metric_by_key("component.list.row_height")
        .unwrap_or_else(|| size.list_row_h(theme));
    Px(base.0.max(0.0))
}

fn resolve_row_padding_x(theme: &Theme) -> Px {
    // Prefer component-level Tailwind-like tokens; fall back to baseline metrics to avoid drift.
    MetricRef::space(Space::N2p5).resolve(theme)
}

fn resolve_row_padding_y(theme: &Theme) -> Px {
    MetricRef::space(Space::N1p5).resolve(theme)
}

fn resolve_indent(theme: &Theme) -> Px {
    MetricRef::space(Space::N4).resolve(theme)
}

struct DefaultTreeRowRenderer;

impl<H: UiHost> TreeRowRenderer<H> for DefaultTreeRowRenderer {
    fn render_row(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        entry: &TreeEntry,
        _state: TreeRowState,
    ) -> Elements {
        vec![cx.text(entry.label.as_ref())].into()
    }
}

#[derive(Default)]
struct TreeRowsState {
    last_items_revision: Option<u64>,
    last_state_revision: Option<u64>,
    entries: Vec<TreeEntry>,
    index_by_id: HashMap<TreeItemId, usize>,
    scroll: VirtualListScrollHandle,
}

fn rebuild_entries(
    items: Vec<crate::TreeItem>,
    expanded: &std::collections::HashSet<TreeItemId>,
) -> (Vec<TreeEntry>, HashMap<TreeItemId, usize>) {
    let entries = flatten_tree(&items, expanded);
    let index_by_id: HashMap<TreeItemId, usize> =
        entries.iter().enumerate().map(|(i, e)| (e.id, i)).collect();
    (entries, index_by_id)
}

/// Declarative tree view helper (virtualized, component-friendly).
///
/// This is intentionally minimal:
/// - selection/expand policies live in the parent `TreeView` widget,
/// - this function only renders rows and dispatches `tree.select.<id>` / `tree.toggle.<id>` commands.
pub fn tree_view<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    items: Model<Vec<crate::TreeItem>>,
    state: Model<TreeState>,
    size: Size,
) -> AnyElement {
    let mut renderer = DefaultTreeRowRenderer;
    tree_view_with_renderer(cx, items, state, size, &mut renderer)
}

/// A retained-host variant of [`tree_view`] that enables composable rows under cache-root reuse.
///
/// This uses the virt-003 retained VirtualList host path, so overscan boundary updates can attach/detach
/// item subtrees without forcing a parent cache-root rerender.
///
/// Notes:
/// - v1 only supports fixed row height (no measured mode).
/// - `debug_row_test_id_prefix` is intended for scripted UI Gallery harnesses.
pub fn tree_view_retained<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    items: Model<Vec<crate::TreeItem>>,
    state: Model<TreeState>,
    size: Size,
    debug_row_test_id_prefix: Option<Arc<str>>,
) -> AnyElement
where
    H: 'static,
{
    tree_view_retained_impl(cx, items, state, size, debug_row_test_id_prefix)
}

pub fn tree_view_with_renderer<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    items: Model<Vec<crate::TreeItem>>,
    state: Model<TreeState>,
    size: Size,
    renderer: &mut impl TreeRowRenderer<H>,
) -> AnyElement {
    let items_revision = cx.app.models().revision(&items).unwrap_or(0);
    let state_revision = cx.app.models().revision(&state).unwrap_or(0);

    let TreeState { selected, expanded } =
        cx.watch_model(&state).paint().cloned().unwrap_or_default();

    let items_value = cx.watch_model(&items).layout().cloned().unwrap_or_default();

    let theme = Theme::global(&*cx.app);
    let (list_bg, border, row_hover, row_active) = resolve_list_colors(theme);
    let radius = theme.metric_required("metric.radius.md");

    let row_h = resolve_row_height(theme, size);
    let row_px = resolve_row_padding_x(theme);
    let row_py = resolve_row_padding_y(theme);
    let indent = resolve_indent(theme);

    let (entries, index_by_id, scroll) = cx.with_state(TreeRowsState::default, |rows_state| {
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
            rows_state.scroll.clone(),
        )
    });

    if let Some(id) = selected
        && let Some(idx) = index_by_id.get(&id).copied()
    {
        scroll.scroll_to_item(idx, ScrollStrategy::Nearest);
    }

    let len = entries.len();
    let items_revision = items_revision ^ state_revision.rotate_left(17);

    let mut options = fret_ui::element::VirtualListOptions::new(row_h, 2);
    options.items_revision = items_revision;

    let mut fill_layout = LayoutStyle::default();
    fill_layout.size.width = Length::Fill;
    fill_layout.size.height = Length::Fill;
    fill_layout.flex.grow = 1.0;
    fill_layout.flex.basis = Length::Px(Px(0.0));

    cx.container(
        ContainerProps {
            layout: fill_layout,
            background: Some(list_bg),
            border: Edges::all(Px(1.0)),
            border_color: Some(border),
            corner_radii: Corners::all(radius),
            ..Default::default()
        },
        |cx| {
            let entries_for_key = Arc::clone(&entries);
            let entries_for_row = Arc::clone(&entries);
            let expanded = expanded.clone();

            vec![cx.virtual_list_keyed_with_layout(
                fill_layout,
                len,
                options,
                &scroll,
                move |i| entries_for_key.get(i).map(|e| e.id).unwrap_or_default(),
                |cx, i| {
                    let Some(entry) = entries_for_row.get(i).cloned() else {
                        return cx.text("");
                    };

                    let is_selected = selected == Some(entry.id);
                    let is_expanded = entry.has_children && expanded.contains(&entry.id);
                    let row_state = TreeRowState {
                        selected: is_selected,
                        expanded: is_expanded,
                        disabled: entry.disabled,
                        depth: entry.depth,
                        has_children: entry.has_children,
                    };

                    let bg = if is_selected { Some(row_active) } else { None };
                    let enabled = !entry.disabled;

                    let pad_left = Px(row_px.0 + indent.0 * (entry.depth as f32).max(0.0));
                    let toggle_cmd = entry
                        .has_children
                        .then(|| CommandId::new(format!("tree.toggle.{}", entry.id)));
                    let select_cmd =
                        enabled.then(|| CommandId::new(format!("tree.select.{}", entry.id)));

                    cx.pressable(
                        PressableProps {
                            enabled,
                            a11y: PressableA11y {
                                role: Some(SemanticsRole::TreeItem),
                                label: Some(entry.label.clone()),
                                selected: is_selected,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        |cx, st| {
                            cx.pressable_dispatch_command_if_enabled_opt(select_cmd);
                            let row_bg = if bg.is_some() {
                                bg
                            } else if enabled && st.pressed {
                                Some(row_active)
                            } else if enabled && st.hovered {
                                Some(row_hover)
                            } else {
                                None
                            };

                            vec![cx.container(
                                ContainerProps {
                                    padding: Edges {
                                        top: row_py,
                                        right: row_px,
                                        bottom: row_py,
                                        left: pad_left,
                                    },
                                    background: row_bg,
                                    ..Default::default()
                                },
                                |cx| {
                                    vec![stack::hstack(
                                        cx,
                                        stack::HStackProps::default()
                                            .gap_x(Space::N2)
                                            .justify(Justify::Start)
                                            .items(Items::Center),
                                        |cx| {
                                            let mut out = Vec::new();

                                            if entry.has_children {
                                                // Toggle button (kept separate so click target is predictable).
                                                let glyph: Arc<str> =
                                                    Arc::from(if is_expanded { "v" } else { ">" });
                                                out.push(cx.pressable(
                                                    PressableProps {
                                                        enabled: toggle_cmd.is_some(),
                                                        a11y: PressableA11y {
                                                            role: Some(SemanticsRole::Button),
                                                            label: Some(Arc::from("Toggle")),
                                                            selected: false,
                                                            ..Default::default()
                                                        },
                                                        ..Default::default()
                                                    },
                                                    |cx, _st| {
                                                        cx.pressable_dispatch_command_if_enabled_opt(toggle_cmd);
                                                        vec![cx.text(glyph.as_ref())]
                                                    },
                                                ));
                                            } else {
                                                out.push(cx.spacer(SpacerProps {
                                                    min: Px(14.0),
                                                    ..Default::default()
                                                }));
                                            }

                                            out.extend(renderer.render_row(cx, &entry, row_state));
                                            out.push(cx.spacer(SpacerProps::default()));
                                            out.extend(
                                                renderer.render_trailing(cx, &entry, row_state),
                                            );
                                            out
                                        },
                                    )]
                                },
                            )]
                        },
                    )
                },
            )]
        },
    )
}

fn tree_view_retained_impl<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    items: Model<Vec<crate::TreeItem>>,
    state: Model<TreeState>,
    size: Size,
    debug_row_test_id_prefix: Option<Arc<str>>,
) -> AnyElement
where
    H: 'static,
{
    let items_revision = cx.app.models().revision(&items).unwrap_or(0);
    let state_revision = cx.app.models().revision(&state).unwrap_or(0);

    let TreeState { selected, expanded } =
        cx.watch_model(&state).paint().cloned().unwrap_or_default();

    let items_value = cx.watch_model(&items).layout().cloned().unwrap_or_default();

    let theme = Theme::global(&*cx.app);
    let (list_bg, border, row_hover, row_active) = resolve_list_colors(theme);
    let radius = theme.metric_required("metric.radius.md");

    let row_h = resolve_row_height(theme, size);
    let row_px = resolve_row_padding_x(theme);
    let row_py = resolve_row_padding_y(theme);
    let indent = resolve_indent(theme);

    let (entries, index_by_id, scroll) = cx.with_state(TreeRowsState::default, |rows_state| {
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
            rows_state.scroll.clone(),
        )
    });

    if let Some(id) = selected
        && let Some(idx) = index_by_id.get(&id).copied()
    {
        scroll.scroll_to_item(idx, ScrollStrategy::Nearest);
    }

    let len = entries.len();
    let items_revision = items_revision ^ state_revision.rotate_left(17);

    let mut options = fret_ui::element::VirtualListOptions::new(row_h, 2);
    options.items_revision = items_revision;
    options.measure_mode = fret_ui::element::VirtualListMeasureMode::Fixed;
    options.key_cache = fret_ui::element::VirtualListKeyCacheMode::VisibleOnly;

    let mut fill_layout = LayoutStyle::default();
    fill_layout.size.width = Length::Fill;
    fill_layout.size.height = Length::Fill;
    fill_layout.flex.grow = 1.0;
    fill_layout.flex.basis = Length::Px(Px(0.0));

    cx.container(
        ContainerProps {
            layout: fill_layout,
            background: Some(list_bg),
            border: Edges::all(Px(1.0)),
            border_color: Some(border),
            corner_radii: Corners::all(radius),
            ..Default::default()
        },
        |cx| {
            let entries_for_key = Arc::clone(&entries);
            let entries_for_row = Arc::clone(&entries);
            let expanded = expanded.clone();

            let key_at: Arc<dyn Fn(usize) -> fret_ui::ItemKey> = Arc::new(move |i| {
                entries_for_key
                    .get(i)
                    .map(|e: &TreeEntry| e.id)
                    .unwrap_or_default()
            });
            let row_test_id_prefix = debug_row_test_id_prefix.clone();

            let row: Arc<dyn for<'a> Fn(&mut ElementContext<'a, H>, usize) -> AnyElement> =
                Arc::new(move |cx: &mut ElementContext<'_, H>, i| {
                    let Some(entry) = entries_for_row.get(i).cloned() else {
                        return cx.text("");
                    };

                    let is_selected = selected == Some(entry.id);
                    let is_expanded = entry.has_children && expanded.contains(&entry.id);
                    let row_state = TreeRowState {
                        selected: is_selected,
                        expanded: is_expanded,
                        disabled: entry.disabled,
                        depth: entry.depth,
                        has_children: entry.has_children,
                    };

                    let bg = if is_selected { Some(row_active) } else { None };
                    let enabled = !entry.disabled;

                    let pad_left = Px(row_px.0 + indent.0 * (entry.depth as f32).max(0.0));
                    let toggle_cmd = entry
                        .has_children
                        .then(|| CommandId::new(format!("tree.toggle.{}", entry.id)));
                    let select_cmd =
                        enabled.then(|| CommandId::new(format!("tree.select.{}", entry.id)));

                    let debug_test_id: Option<Arc<str>> = row_test_id_prefix
                        .as_ref()
                        .map(|prefix| Arc::from(format!("{prefix}-{}", entry.id)));
                    let debug_toggle_test_id: Option<Arc<str>> = debug_test_id
                        .as_ref()
                        .map(|id| Arc::from(format!("{id}-toggle")));

                    let row_el = cx.pressable(
                        PressableProps {
                            enabled,
                            a11y: PressableA11y {
                                role: Some(SemanticsRole::TreeItem),
                                label: Some(entry.label.clone()),
                                selected: is_selected,
                                test_id: debug_test_id.clone(),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        |cx, st| {
                            cx.pressable_dispatch_command_if_enabled_opt(select_cmd);
                            let row_bg = if bg.is_some() {
                                bg
                            } else if enabled && st.pressed {
                                Some(row_active)
                            } else if enabled && st.hovered {
                                Some(row_hover)
                            } else {
                                None
                            };

                            let mut renderer = DefaultTreeRowRenderer;
                            vec![cx.container(
                                ContainerProps {
                                    padding: Edges {
                                        top: row_py,
                                        right: row_px,
                                        bottom: row_py,
                                        left: pad_left,
                                    },
                                    background: row_bg,
                                    ..Default::default()
                                },
                                |cx| {
                                    vec![stack::hstack(
                                        cx,
                                        stack::HStackProps::default()
                                            .gap_x(Space::N2)
                                            .justify(Justify::Start)
                                            .items(Items::Center),
                                        |cx| {
                                            let mut out = Vec::new();

                                            if entry.has_children {
                                                let glyph: Arc<str> =
                                                    Arc::from(if is_expanded { "v" } else { ">" });
                                                out.push(cx.pressable(
                                                PressableProps {
                                                    enabled: toggle_cmd.is_some(),
                                                    a11y: PressableA11y {
                                                        role: Some(SemanticsRole::Button),
                                                        label: Some(Arc::from("Toggle")),
                                                        selected: false,
                                                        test_id: debug_toggle_test_id.clone(),
                                                        ..Default::default()
                                                    },
                                                    ..Default::default()
                                                },
                                                |cx, _st| {
                                                    cx.pressable_dispatch_command_if_enabled_opt(
                                                        toggle_cmd,
                                                    );
                                                    vec![cx.text(glyph.as_ref())]
                                                },
                                            ));
                                            } else {
                                                out.push(cx.spacer(SpacerProps {
                                                    min: Px(14.0),
                                                    ..Default::default()
                                                }));
                                            }

                                            out.extend(renderer.render_row(cx, &entry, row_state));
                                            out.push(cx.spacer(SpacerProps::default()));
                                            out.extend(
                                                renderer.render_trailing(cx, &entry, row_state),
                                            );
                                            out
                                        },
                                    )]
                                },
                            )]
                        },
                    );

                    row_el
                });

            vec![cx.virtual_list_keyed_retained_with_layout(
                fill_layout,
                len,
                options,
                &scroll,
                key_at,
                row,
            )]
        },
    )
}
