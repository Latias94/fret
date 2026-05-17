use std::collections::HashMap;
use std::sync::Arc;

use fret_core::{Color, Corners, Edges, Px, SemanticsRole};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, Elements, LayoutStyle, Length, Overflow, PressableA11y,
    PressableProps, SpacerProps, VirtualListMeasureMode,
};
use fret_ui::scroll::{ScrollStrategy, VirtualListScrollHandle};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::declarative::action_hooks::ActionHooksExt;
use crate::declarative::model_watch::ModelWatchExt as _;
use crate::ui;
use crate::{
    MetricRef, Size, Space, TreeEntry, TreeItemId, TreeRowRenderer, TreeRowState, TreeState,
    flatten_tree,
};

type RetainedTreeRowFn<H> = dyn for<'a> Fn(&mut ElementContext<'a, H>, usize) -> AnyElement;

fn resolve_list_colors(theme: &Theme) -> (Color, Color, Color, Color) {
    let list_bg = theme
        .color_by_key("list.background")
        .or_else(|| theme.color_by_key("card"))
        .unwrap_or_else(|| theme.color_token("card"));
    let border = theme
        .color_by_key("border")
        .or_else(|| theme.color_by_key("list.border"))
        .unwrap_or_else(|| theme.color_token("border"));
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

fn default_tree_row_label<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: impl Into<Arc<str>>,
) -> AnyElement {
    crate::declarative::text::text_list_row_label(cx, label)
}

fn tree_toggle_glyph<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    glyph: impl Into<Arc<str>>,
) -> AnyElement {
    crate::declarative::text::text_chrome_glyph(cx, glyph)
}

fn tree_missing_virtual_row_placeholder<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    cx.spacer(SpacerProps::default())
}

fn retained_tree_row_layout(row_h: Px, measure_mode: VirtualListMeasureMode) -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    if matches!(
        measure_mode,
        VirtualListMeasureMode::Fixed | VirtualListMeasureMode::Known
    ) {
        layout.size.height = Length::Px(row_h);
        layout.overflow = Overflow::Clip;
    }
    layout
}

impl<H: UiHost> TreeRowRenderer<H> for DefaultTreeRowRenderer {
    fn render_row(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        entry: &TreeEntry,
        _state: TreeRowState,
    ) -> Elements {
        vec![default_tree_row_label(cx, entry.label.clone())].into()
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

fn tree_item_a11y(
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

fn select_tree_item(state: &mut TreeState, id: TreeItemId) {
    state.selected = Some(id);
}

fn toggle_tree_item_expanded(state: &mut TreeState, id: TreeItemId) {
    if !state.expanded.insert(id) {
        state.expanded.remove(&id);
    }
}

/// Declarative tree view helper (virtualized, component-friendly).
///
/// This is intentionally minimal:
/// - selection/expand policies live in this UI-kit helper because it owns the `TreeState` model,
/// - consumers can still use a custom row renderer for content policy.
#[track_caller]
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
/// - This defaults to fixed row height for predictable perf. If you need variable-height rows,
///   use [`tree_view_retained_with_measure_mode`] with `VirtualListMeasureMode::Measured`.
/// - `debug_row_test_id_prefix` is intended for scripted UI Gallery harnesses.
#[track_caller]
pub fn tree_view_retained<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    items: Model<Vec<crate::TreeItem>>,
    state: Model<TreeState>,
    size: Size,
    debug_row_test_id_prefix: Option<Arc<str>>,
) -> AnyElement {
    tree_view_retained_with_measure_mode(
        cx,
        items,
        state,
        size,
        fret_ui::element::VirtualListMeasureMode::Fixed,
        debug_row_test_id_prefix,
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
            id: 42,
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

    fn render_retained_tree(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut FakeServices,
        window: AppWindowId,
        bounds: Rect,
        measure_mode: VirtualListMeasureMode,
    ) {
        let items = app.models_mut().insert(vec![crate::TreeItem::new(
            7,
            "Long retained tree row that must not paint outside a fixed row",
        )]);
        let state = app.models_mut().insert(TreeState::default());

        for _ in 0..3 {
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "retained-tree-row-clip-test",
                |cx| {
                    vec![tree_view_retained_with_measure_mode(
                        cx,
                        items.clone(),
                        state.clone(),
                        Size::Medium,
                        measure_mode,
                        Some(Arc::from("retained-tree-row")),
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

    #[test]
    fn tree_item_a11y_sets_level_and_expanded_for_parent_rows() {
        let a11y = tree_item_a11y(&entry(true, 1), true, false, Some(Arc::from("row")));

        assert_eq!(a11y.role, Some(SemanticsRole::TreeItem));
        assert_eq!(a11y.level, Some(2));
        assert!(a11y.selected);
        assert_eq!(a11y.expanded, Some(false));
        assert_eq!(a11y.test_id.as_deref(), Some("row"));
    }

    #[test]
    fn tree_item_a11y_omits_expanded_for_leaf_rows() {
        let a11y = tree_item_a11y(&entry(false, 2), false, false, None);

        assert_eq!(a11y.level, Some(3));
        assert_eq!(a11y.expanded, None);
    }

    #[test]
    fn select_tree_item_updates_state_selection() {
        let mut state = TreeState::default();

        select_tree_item(&mut state, 42);

        assert_eq!(state.selected, Some(42));
    }

    #[test]
    fn toggle_tree_item_expanded_flips_state_membership() {
        let mut state = TreeState::default();

        toggle_tree_item_expanded(&mut state, 42);
        assert!(state.expanded.contains(&42));

        toggle_tree_item_expanded(&mut state, 42);
        assert!(!state.expanded.contains(&42));
    }

    #[test]
    fn default_tree_row_label_uses_shared_list_row_text_role() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let element =
            fret_ui::elements::with_element_cx(&mut app, window, test_bounds(), "test", |cx| {
                default_tree_row_label(cx, "Long tree row label that should not wrap")
            });

        let ElementKind::Text(props) = &element.kind else {
            panic!("default tree row label should be text");
        };
        assert_eq!(props.layout.size.width, Length::Fill);
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.layout.flex.shrink, 1.0);
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
    }

    #[test]
    fn tree_toggle_glyph_uses_shared_chrome_glyph_text_role() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let element =
            fret_ui::elements::with_element_cx(&mut app, window, test_bounds(), "test", |cx| {
                tree_toggle_glyph(cx, ">")
            });

        let ElementKind::Text(props) = &element.kind else {
            panic!("tree toggle glyph should be text");
        };
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.layout.flex.shrink, 1.0);
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Clip);
    }

    #[test]
    fn missing_tree_virtual_row_placeholder_is_not_text() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let element =
            fret_ui::elements::with_element_cx(&mut app, window, test_bounds(), "test", |cx| {
                tree_missing_virtual_row_placeholder(cx)
            });

        let ElementKind::Spacer(props) = &element.kind else {
            panic!("missing tree virtual row placeholder should be a spacer");
        };
        assert_eq!(props.min, Px(0.0));
    }

    #[test]
    fn retained_tree_fixed_rows_clip_to_row_height() {
        let layout = retained_tree_row_layout(Px(24.0), VirtualListMeasureMode::Fixed);

        assert_eq!(layout.size.width, Length::Fill);
        assert_eq!(layout.size.height, Length::Px(Px(24.0)));
        assert_eq!(layout.overflow, Overflow::Clip);
    }

    #[test]
    fn retained_tree_known_rows_clip_to_row_height() {
        let layout = retained_tree_row_layout(Px(32.0), VirtualListMeasureMode::Known);

        assert_eq!(layout.size.width, Length::Fill);
        assert_eq!(layout.size.height, Length::Px(Px(32.0)));
        assert_eq!(layout.overflow, Overflow::Clip);
    }

    #[test]
    fn retained_tree_measured_rows_keep_overflow_visible_for_measurement() {
        let layout = retained_tree_row_layout(Px(24.0), VirtualListMeasureMode::Measured);

        assert_eq!(layout.size.width, Length::Fill);
        assert_eq!(layout.size.height, Length::Auto);
        assert_eq!(layout.overflow, LayoutStyle::default().overflow);
    }

    #[test]
    fn retained_tree_fixed_rows_mount_as_clip_boundaries() {
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

        render_retained_tree(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            VirtualListMeasureMode::Fixed,
        );

        let row_node = semantics_node_id_by_test_id(&ui, "retained-tree-row-7");
        let row_bounds = ui.debug_node_bounds(row_node).expect("row bounds");

        assert_eq!(
            ui.debug_declarative_instance_kind(&mut app, window, row_node),
            Some("Pressable")
        );
        assert!(
            row_bounds.size.height.0 <= 32.5,
            "fixed retained tree row should keep medium row height, got {row_bounds:?}"
        );
        assert_eq!(
            ui.debug_node_clips_hit_test(row_node),
            Some(true),
            "fixed retained tree row should clip oversized/wrapping row content"
        );
    }

    #[test]
    fn retained_tree_measured_rows_do_not_force_row_clip() {
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

        render_retained_tree(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            VirtualListMeasureMode::Measured,
        );

        let row_node = semantics_node_id_by_test_id(&ui, "retained-tree-row-7");

        assert_eq!(
            ui.debug_declarative_instance_kind(&mut app, window, row_node),
            Some("Pressable")
        );
        assert_eq!(
            ui.debug_node_clips_hit_test(row_node),
            Some(false),
            "measured retained tree row should keep overflow visible for runtime measurement"
        );
    }
}

/// A variant of [`tree_view_retained`] that allows opting into measured (variable-height) rows.
#[track_caller]
pub fn tree_view_retained_with_measure_mode<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    items: Model<Vec<crate::TreeItem>>,
    state: Model<TreeState>,
    size: Size,
    measure_mode: fret_ui::element::VirtualListMeasureMode,
    debug_row_test_id_prefix: Option<Arc<str>>,
) -> AnyElement {
    tree_view_retained_impl(
        cx,
        items,
        state,
        size,
        measure_mode,
        debug_row_test_id_prefix,
    )
}

#[track_caller]
pub fn tree_view_with_renderer<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    items: Model<Vec<crate::TreeItem>>,
    state: Model<TreeState>,
    size: Size,
    renderer: &mut impl TreeRowRenderer<H>,
) -> AnyElement {
    let items_revision = cx.app.models().revision(&items).unwrap_or(0);
    let state_revision = cx.app.models().revision(&state).unwrap_or(0);

    let TreeState { selected, expanded } = cx.watch_model(&state).paint().cloned_or_default();

    let items_value = cx.watch_model(&items).layout().cloned_or_default();

    let theme = Theme::global(&*cx.app);
    let (list_bg, border, row_hover, row_active) = resolve_list_colors(theme);
    let radius = theme.metric_token("metric.radius.md");

    let row_h = resolve_row_height(theme, size);
    let row_px = resolve_row_padding_x(theme);
    let row_py = resolve_row_padding_y(theme);
    let indent = resolve_indent(theme);

    let (entries, index_by_id, scroll) = cx.slot_state(TreeRowsState::default, |rows_state| {
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
                        return tree_missing_virtual_row_placeholder(cx);
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
                    cx.pressable(
                        PressableProps {
                            enabled,
                            a11y: tree_item_a11y(&entry, is_selected, is_expanded, None),
                            ..Default::default()
                        },
                        |cx, st| {
                            if enabled {
                                let row_id = entry.id;
                                cx.pressable_update_model(&state, move |st| {
                                    select_tree_item(st, row_id);
                                });
                            }
                            let row_bg = if bg.is_some() {
                                bg
                            } else if enabled && st.pressed {
                                Some(row_active)
                            } else if enabled && st.hovered {
                                Some(row_hover)
                            } else {
                                None
                            };

                            vec![
                                cx.container(
                                    ContainerProps {
                                        padding: Edges {
                                            top: row_py,
                                            right: row_px,
                                            bottom: row_py,
                                            left: pad_left,
                                        }
                                        .into(),
                                        background: row_bg,
                                        ..Default::default()
                                    },
                                    |cx| {
                                        vec![
                                            ui::h_row(|cx| {
                                                let mut out = Vec::new();

                                                if entry.has_children {
                                                    // Toggle button (kept separate so click target is predictable).
                                                    let glyph: Arc<str> =
                                                        Arc::from(if is_expanded {
                                                            "v"
                                                        } else {
                                                            ">"
                                                        });
                                                    out.push(cx.pressable(
                                                        PressableProps {
                                                            enabled: entry.has_children,
                                                            a11y: PressableA11y {
                                                                role: Some(SemanticsRole::Button),
                                                                label: Some(Arc::from("Toggle")),
                                                                expanded: Some(is_expanded),
                                                                ..Default::default()
                                                            },
                                                            ..Default::default()
                                                        },
                                                        |cx, _st| {
                                                            let row_id = entry.id;
                                                            cx.pressable_update_model(
                                                                &state,
                                                                move |st| {
                                                                    toggle_tree_item_expanded(
                                                                        st, row_id,
                                                                    );
                                                                },
                                                            );
                                                            vec![tree_toggle_glyph(
                                                                cx,
                                                                glyph.clone(),
                                                            )]
                                                        },
                                                    ));
                                                } else {
                                                    out.push(cx.spacer(SpacerProps {
                                                        min: Px(14.0),
                                                        ..Default::default()
                                                    }));
                                                }

                                                out.extend(
                                                    renderer.render_row(cx, &entry, row_state),
                                                );
                                                out.push(cx.spacer(SpacerProps::default()));
                                                out.extend(
                                                    renderer.render_trailing(cx, &entry, row_state),
                                                );
                                                out
                                            })
                                            .gap(Space::N2)
                                            .justify_start()
                                            .items_center()
                                            .into_element(cx),
                                        ]
                                    },
                                ),
                            ]
                        },
                    )
                },
            )]
        },
    )
}

#[track_caller]
fn tree_view_retained_impl<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    items: Model<Vec<crate::TreeItem>>,
    state: Model<TreeState>,
    size: Size,
    measure_mode: fret_ui::element::VirtualListMeasureMode,
    debug_row_test_id_prefix: Option<Arc<str>>,
) -> AnyElement {
    let items_revision = cx.app.models().revision(&items).unwrap_or(0);
    let state_revision = cx.app.models().revision(&state).unwrap_or(0);

    let TreeState { selected, expanded } = cx.watch_model(&state).paint().cloned_or_default();

    let items_value = cx.watch_model(&items).layout().cloned_or_default();

    let theme = Theme::global(&*cx.app);
    let (list_bg, border, row_hover, row_active) = resolve_list_colors(theme);
    let radius = theme.metric_token("metric.radius.md");

    let row_h = resolve_row_height(theme, size);
    let row_px = resolve_row_padding_x(theme);
    let row_py = resolve_row_padding_y(theme);
    let indent = resolve_indent(theme);

    let (entries, index_by_id, scroll) = cx.slot_state(TreeRowsState::default, |rows_state| {
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
    options.measure_mode = measure_mode;
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

            let row: Arc<RetainedTreeRowFn<H>> =
                Arc::new(move |cx: &mut ElementContext<'_, H>, i| {
                    let Some(entry) = entries_for_row.get(i).cloned() else {
                        return tree_missing_virtual_row_placeholder(cx);
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
                    let debug_test_id: Option<Arc<str>> = row_test_id_prefix
                        .as_ref()
                        .map(|prefix| Arc::from(format!("{prefix}-{}", entry.id)));
                    let debug_toggle_test_id: Option<Arc<str>> = debug_test_id
                        .as_ref()
                        .map(|id| Arc::from(format!("{id}-toggle")));

                    cx.pressable(
                        PressableProps {
                            layout: retained_tree_row_layout(row_h, measure_mode),
                            enabled,
                            a11y: tree_item_a11y(
                                &entry,
                                is_selected,
                                is_expanded,
                                debug_test_id.clone(),
                            ),
                            ..Default::default()
                        },
                        |cx, st| {
                            if enabled {
                                let row_id = entry.id;
                                cx.pressable_update_model(&state, move |st| {
                                    select_tree_item(st, row_id);
                                });
                            }
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
                            vec![
                                cx.container(
                                    ContainerProps {
                                        padding: Edges {
                                            top: row_py,
                                            right: row_px,
                                            bottom: row_py,
                                            left: pad_left,
                                        }
                                        .into(),
                                        background: row_bg,
                                        ..Default::default()
                                    },
                                    |cx| {
                                        vec![
                                            ui::h_row(|cx| {
                                                let mut out = Vec::new();

                                                if entry.has_children {
                                                    let glyph: Arc<str> =
                                                        Arc::from(if is_expanded {
                                                            "v"
                                                        } else {
                                                            ">"
                                                        });
                                                    out.push(cx.pressable(
                                                        PressableProps {
                                                            enabled: entry.has_children,
                                                            a11y: PressableA11y {
                                                                role: Some(SemanticsRole::Button),
                                                                label: Some(Arc::from("Toggle")),
                                                                expanded: Some(is_expanded),
                                                                test_id:
                                                                    debug_toggle_test_id.clone(),
                                                                ..Default::default()
                                                            },
                                                            ..Default::default()
                                                        },
                                                        |cx, _st| {
                                                            let row_id = entry.id;
                                                            cx.pressable_update_model(
                                                                &state,
                                                                move |st| {
                                                                    toggle_tree_item_expanded(
                                                                        st, row_id,
                                                                    );
                                                                },
                                                            );
                                                            vec![tree_toggle_glyph(
                                                                cx,
                                                                glyph.clone(),
                                                            )]
                                                        },
                                                    ));
                                                } else {
                                                    out.push(cx.spacer(SpacerProps {
                                                        min: Px(14.0),
                                                        ..Default::default()
                                                    }));
                                                }

                                                out.extend(
                                                    renderer.render_row(cx, &entry, row_state),
                                                );
                                                out.push(cx.spacer(SpacerProps::default()));
                                                out.extend(
                                                    renderer.render_trailing(cx, &entry, row_state),
                                                );
                                                out
                                            })
                                            .gap(Space::N2)
                                            .justify_start()
                                            .items_center()
                                            .into_element(cx),
                                        ]
                                    },
                                ),
                            ]
                        },
                    )
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
