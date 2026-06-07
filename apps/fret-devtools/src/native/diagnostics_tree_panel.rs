use std::hash::{Hash, Hasher};
use std::sync::Arc;

use fret_app::App;
use fret_core::Px;
use fret_ui::element::{AnyElement, LayoutStyle, Length, VirtualListOptions};
use fret_ui::scroll::ScrollStrategy;
use fret_ui::scroll::VirtualListScrollHandle;
use fret_ui::ElementContext;
use fret_ui_kit::ui;
use fret_ui_shadcn::facade as shadcn;

use super::{semantics, State};

#[derive(Debug, Clone, Copy)]
enum InspectTreeMode {
    Semantics,
    Layout,
    Elements,
}

impl InspectTreeMode {
    fn search_label(self) -> &'static str {
        match self {
            Self::Semantics => "Semantics search",
            Self::Layout => "Layout tree search",
            Self::Elements => "Element tree search",
        }
    }

    fn search_placeholder(self) -> &'static str {
        match self {
            Self::Semantics => "Search role/test_id/label/value...",
            Self::Layout => "Search role/test_id/bounds/parent...",
            Self::Elements => "Search role/test_id/id/relationships...",
        }
    }

    fn empty_text(self) -> &'static str {
        match self {
            Self::Semantics => {
                "No semantics yet. Use 'Dump Bundle' or run a script that dumps a bundle."
            }
            Self::Layout => {
                "No layout-bounds tree yet. Use 'Dump Bundle' or run a script that dumps semantics bounds."
            }
            Self::Elements => {
                "No element-identity tree yet. Use 'Dump Bundle' or run a script that dumps semantics identity."
            }
        }
    }

    fn error_prefix(self) -> &'static str {
        match self {
            Self::Semantics => "semantics error",
            Self::Layout => "layout tree error",
            Self::Elements => "element tree error",
        }
    }

    fn stats_prefix(self) -> &'static str {
        match self {
            Self::Semantics => "semantics",
            Self::Layout => "layout-derived",
            Self::Elements => "element-derived",
        }
    }

    fn cache_discriminant(self) -> u8 {
        match self {
            Self::Semantics => 0,
            Self::Layout => 1,
            Self::Elements => 2,
        }
    }
}

pub(super) fn semantics_panel(cx: &mut ElementContext<'_, App>, st: &State) -> AnyElement {
    diagnostics_tree_panel(cx, st, InspectTreeMode::Semantics)
}

pub(super) fn layout_tree_panel(cx: &mut ElementContext<'_, App>, st: &State) -> AnyElement {
    diagnostics_tree_panel(cx, st, InspectTreeMode::Layout)
}

pub(super) fn element_tree_panel(cx: &mut ElementContext<'_, App>, st: &State) -> AnyElement {
    diagnostics_tree_panel(cx, st, InspectTreeMode::Elements)
}

fn diagnostics_tree_panel(
    cx: &mut ElementContext<'_, App>,
    st: &State,
    mode: InspectTreeMode,
) -> AnyElement {
    let index = cx
        .app
        .models()
        .read(&st.semantics_cache, |v| v.clone())
        .ok()
        .flatten();
    let error = cx
        .app
        .models()
        .read(&st.semantics_error, |v| v.clone())
        .ok()
        .flatten();
    let search = cx
        .app
        .models()
        .read(&st.semantics_search, |v| v.clone())
        .unwrap_or_default();
    let expanded = cx
        .app
        .models()
        .read(&st.semantics_expanded, |v| v.clone())
        .unwrap_or_default();
    let selected_id = cx
        .app
        .models()
        .read(&st.semantics_selected_id, |v| *v)
        .ok()
        .flatten();
    let source_hash = cx
        .app
        .models()
        .read(&st.semantics_source_hash, |v| *v)
        .ok()
        .flatten()
        .unwrap_or(0);

    let search_input = shadcn::Input::new(st.semantics_search.clone())
        .a11y_label(mode.search_label())
        .placeholder(mode.search_placeholder())
        .into_element(cx);

    let header = ui::h_row(|_cx| [search_input])
        .gap(fret_ui_kit::Space::N2)
        .items_center()
        .into_element(cx);

    let content: AnyElement = match (index, error) {
        (_index, Some(err)) => cx.text(format!("{}: {err}", mode.error_prefix())),
        (None, None) => cx.text(mode.empty_text()),
        (Some(index), None) => {
            #[derive(Debug, Default)]
            struct RowsCache {
                key: u64,
                rows: Arc<Vec<semantics::SemanticsRow>>,
            }

            #[derive(Debug, Default)]
            struct SelectionScrollSync {
                last: Option<(u64, u64)>,
            }

            let rows_key = {
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                mode.cache_discriminant().hash(&mut hasher);
                source_hash.hash(&mut hasher);
                search.trim().to_lowercase().hash(&mut hasher);
                let mut expanded_sorted: Vec<u64> = expanded.iter().copied().collect();
                expanded_sorted.sort_unstable();
                expanded_sorted.hash(&mut hasher);
                hasher.finish()
            };

            let rows = cx.slot_state(RowsCache::default, |cache| {
                if cache.key != rows_key {
                    let next = semantics::compute_rows(&index, &expanded, &search);
                    cache.key = rows_key;
                    cache.rows = Arc::new(next);
                }
                Arc::clone(&cache.rows)
            });

            let scroll_handle = cx.slot_state(VirtualListScrollHandle::new, |h| h.clone());

            if let Some(sel) = selected_id {
                let rows_for_scroll = Arc::clone(&rows);
                let handle_for_scroll = scroll_handle.clone();
                cx.slot_state(SelectionScrollSync::default, |sync| {
                    let next = (rows_key, sel);
                    if sync.last == Some(next) {
                        return;
                    }
                    sync.last = Some(next);

                    if let Some(idx) = rows_for_scroll.iter().position(|r| r.id == sel) {
                        handle_for_scroll.scroll_to_item(idx, ScrollStrategy::Nearest);
                    }
                });
            } else {
                cx.slot_state(SelectionScrollSync::default, |sync| sync.last = None);
            }

            let mut layout = LayoutStyle::default();
            layout.size.width = Length::Fill;
            layout.size.height = Length::Fill;
            layout.flex.grow = 1.0;

            let mut options = VirtualListOptions::fixed(Px(28.0), 8).keep_alive(16);
            options.items_revision = rows_key;

            let stats = cx.text(format!(
                "{} window={} roots={} nodes={} rows={}",
                mode.stats_prefix(),
                index.window,
                index.roots.len(),
                index.nodes_by_id.len(),
                rows.len()
            ));

            let rows_for_key = Arc::clone(&rows);
            let rows_for_row = Arc::clone(&rows);
            let index_for_list = Arc::clone(&index);
            let selected_id_for_list = selected_id;
            let has_search = !search.trim().is_empty();

            let list = cx.virtual_list_keyed_with_layout(
                layout,
                rows_for_key.len(),
                options,
                &scroll_handle,
                |i| rows_for_key[i].id,
                move |cx, i| {
                    let row = &rows_for_row[i];
                    let id = row.id;

                    let variant = if selected_id_for_list == Some(id) {
                        shadcn::ButtonVariant::Secondary
                    } else {
                        shadcn::ButtonVariant::Ghost
                    };

                    let toggle: AnyElement = if row.has_children {
                        let glyph = if row.is_expanded { "v" } else { ">" };
                        if has_search {
                            cx.text(glyph.to_string())
                        } else {
                            let expanded_model = st.semantics_expanded.clone();
                            let on_toggle: fret_ui::action::OnActivate =
                                Arc::new(move |host, action_cx, _reason| {
                                    let _ = host.models_mut().update(&expanded_model, |set| {
                                        if set.contains(&id) {
                                            set.remove(&id);
                                        } else {
                                            set.insert(id);
                                        }
                                    });
                                    host.request_redraw(action_cx.window);
                                });
                            shadcn::Button::new(glyph)
                                .variant(shadcn::ButtonVariant::Ghost)
                                .size(shadcn::ButtonSize::Sm)
                                .on_activate(on_toggle)
                                .into_element(cx)
                        }
                    } else {
                        cx.text(" ")
                    };

                    let label = index_for_list
                        .node(id)
                        .map(|node| match mode {
                            InspectTreeMode::Semantics => semantics::node_label(node),
                            InspectTreeMode::Layout => semantics::layout_node_label(node),
                            InspectTreeMode::Elements => semantics::element_node_label(node),
                        })
                        .unwrap_or_else(|| format!("<missing semantics node id={id}>"));

                    let selected_id_model = st.semantics_selected_id.clone();
                    let selected_json_model = st.semantics_selected_node_json.clone();
                    let selected_live_json_model = st.semantics_selected_node_live_json.clone();
                    let selected_live_status_model = st.semantics_selected_node_live_status.clone();
                    let selected_live_updated_model =
                        st.semantics_selected_node_live_updated_unix_ms.clone();
                    let selected_live_children_model =
                        st.semantics_selected_node_live_children.clone();
                    let selected_hit_test_explain_json_model =
                        st.semantics_selected_hit_test_explain_json.clone();
                    let selected_hit_test_explain_summary_model =
                        st.semantics_selected_hit_test_explain_summary.clone();
                    let selected_hit_test_explain_status_model =
                        st.semantics_selected_hit_test_explain_status.clone();
                    let selected_hit_test_explain_updated_model =
                        st.semantics_selected_hit_test_explain_updated_unix_ms.clone();
                    let index_for_select = Arc::clone(&index_for_list);
                    let on_select: fret_ui::action::OnActivate =
                        Arc::new(move |host, action_cx, _reason| {
                            let _ = host.models_mut().update(&selected_id_model, |v| {
                                *v = Some(id);
                            });
                            let text =
                                semantics::selected_node_json(index_for_select.as_ref(), Some(id));
                            let _ = host.models_mut().update(&selected_json_model, |v| {
                                *v = text;
                            });
                            let _ = host
                                .models_mut()
                                .update(&selected_live_json_model, |v| v.clear());
                            let _ = host.models_mut().update(&selected_live_status_model, |v| {
                                *v = None;
                            });
                            let _ = host.models_mut().update(&selected_live_updated_model, |v| {
                                *v = None;
                            });
                            let _ = host
                                .models_mut()
                                .update(&selected_live_children_model, |v| v.clear());
                            let _ = host
                                .models_mut()
                                .update(&selected_hit_test_explain_json_model, |v| v.clear());
                            let _ = host
                                .models_mut()
                                .update(&selected_hit_test_explain_summary_model, |v| v.clear());
                            let _ =
                                host.models_mut()
                                    .update(&selected_hit_test_explain_status_model, |v| {
                                        *v = None;
                                    });
                            let _ =
                                host.models_mut().update(
                                    &selected_hit_test_explain_updated_model,
                                    |v| *v = None,
                                );
                            host.request_redraw(action_cx.window);
                        });

                    let row_button = shadcn::Button::new(label)
                        .variant(variant)
                        .size(shadcn::ButtonSize::Sm)
                        .on_activate(on_select)
                        .refine_layout(
                            fret_ui_kit::LayoutRefinement::default()
                                .flex_1()
                                .min_w_0()
                                .ml_px(Px(12.0 * row.depth as f32)),
                        )
                        .into_element(cx);

                    ui::h_row(|_cx| [toggle, row_button])
                        .gap(fret_ui_kit::Space::N1)
                        .items_center()
                        .layout(fret_ui_kit::LayoutRefinement::default().w_full())
                        .into_element(cx)
                },
            );

            ui::v_stack(|_cx| [stats, list])
                .gap(fret_ui_kit::Space::N1)
                .layout(fret_ui_kit::LayoutRefinement::default().w_full().h_full())
                .into_element(cx)
        }
    };

    ui::v_stack(|_cx| [header, content])
        .gap(fret_ui_kit::Space::N2)
        .layout(fret_ui_kit::LayoutRefinement::default().w_full().h_full())
        .into_element(cx)
}
