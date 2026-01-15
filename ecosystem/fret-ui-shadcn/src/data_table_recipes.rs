use std::collections::HashMap;
use std::sync::Arc;

use fret_core::{TextOverflow, TextWrap};
use fret_runtime::Model;
use fret_ui::action::OnActivate;
use fret_ui::element::{AnyElement, LayoutStyle, TextProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_headless::table::{ColumnDef, ColumnId, TableState};
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::stack::{HStackProps, hstack};
use fret_ui_kit::declarative::table::TableViewOutput;
use fret_ui_kit::{LayoutRefinement, Space};

use crate::button::{Button, ButtonSize, ButtonVariant};
use crate::dropdown_menu::{
    DropdownMenu, DropdownMenuCheckboxItem, DropdownMenuEntry, DropdownMenuRadioGroup,
    DropdownMenuRadioItemSpec,
};
use crate::input::Input;

fn is_column_visible(state: &TableState, id: &ColumnId) -> bool {
    state.column_visibility.get(id).copied().unwrap_or(true)
}

fn sync_global_filter<H: UiHost>(app: &mut H, state: &Model<TableState>, value: &str) {
    let next = value.trim();
    let next: Option<Arc<str>> = if next.is_empty() {
        None
    } else {
        Some(Arc::from(next.to_string()))
    };

    let _ = app.models_mut().update(state, |st| {
        if st.global_filter != next {
            st.global_filter = next;
            st.pagination.page_index = 0;
        }
    });
}

fn sync_column_visibility(
    app: &mut impl UiHost,
    state: &Model<TableState>,
    desired: &HashMap<ColumnId, bool>,
) {
    let _ = app.models_mut().update(state, |st| {
        let mut changed = false;
        for (id, visible) in desired {
            let current = is_column_visible(st, id);
            if current == *visible {
                continue;
            }
            changed = true;
            if *visible {
                st.column_visibility.remove(id);
            } else {
                st.column_visibility.insert(id.clone(), false);
            }
        }
        if changed {
            st.pagination.page_index = 0;
        }
    });
}

struct ColumnVisibilityBinding {
    id: ColumnId,
    model: Model<bool>,
}

impl Clone for ColumnVisibilityBinding {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            model: self.model.clone(),
        }
    }
}

#[derive(Default)]
struct DataTableToolbarState {
    filter_model: Option<Model<String>>,
    columns_open: Option<Model<bool>>,
    column_visibility: Vec<ColumnVisibilityBinding>,
}

/// shadcn/ui `DataTable` toolbar (recipe).
///
/// This is a v1 convenience surface that wires common controls to `TableState`:
/// - global filter input (`TableState.global_filter`)
/// - column visibility dropdown (`TableState.column_visibility`)
/// - selected row count (`TableState.row_selection`)
#[derive(Clone)]
pub struct DataTableToolbar<TData> {
    state: Model<TableState>,
    columns: Arc<[ColumnDef<TData>]>,
    column_label: Arc<dyn Fn(&ColumnDef<TData>) -> Arc<str>>,
    filter_placeholder: Arc<str>,
}

impl<TData> std::fmt::Debug for DataTableToolbar<TData> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataTableToolbar")
            .field("columns_len", &self.columns.len())
            .finish_non_exhaustive()
    }
}

impl<TData> DataTableToolbar<TData> {
    pub fn new(
        state: Model<TableState>,
        columns: impl Into<Arc<[ColumnDef<TData>]>>,
        column_label: impl Fn(&ColumnDef<TData>) -> Arc<str> + 'static,
    ) -> Self {
        Self {
            state,
            columns: columns.into(),
            column_label: Arc::new(column_label),
            filter_placeholder: Arc::from("Filter..."),
        }
    }

    pub fn filter_placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.filter_placeholder = placeholder.into();
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement
    where
        TData: 'static,
    {
        let state_value = cx
            .watch_model(&self.state)
            .layout()
            .cloned()
            .unwrap_or_default();

        let filter_model =
            cx.with_state(DataTableToolbarState::default, |st| st.filter_model.clone());
        let filter_model = match filter_model {
            Some(m) => m,
            None => {
                let initial = state_value
                    .global_filter
                    .clone()
                    .unwrap_or_else(|| Arc::from(""))
                    .to_string();
                let m = cx.app.models_mut().insert(initial);
                let m_for_state = m.clone();
                cx.with_state(DataTableToolbarState::default, move |st| {
                    st.filter_model = Some(m_for_state);
                });
                m
            }
        };

        let columns_open =
            cx.with_state(DataTableToolbarState::default, |st| st.columns_open.clone());
        let columns_open = match columns_open {
            Some(m) => m,
            None => {
                let m = cx.app.models_mut().insert(false);
                let m_for_state = m.clone();
                cx.with_state(DataTableToolbarState::default, move |st| {
                    st.columns_open = Some(m_for_state);
                });
                m
            }
        };

        let mut bindings = cx.with_state(DataTableToolbarState::default, |st| {
            st.column_visibility.clone()
        });
        if bindings.is_empty() {
            bindings = self
                .columns
                .iter()
                .filter(|c| c.enable_hiding)
                .map(|c| ColumnVisibilityBinding {
                    id: c.id.clone(),
                    model: cx
                        .app
                        .models_mut()
                        .insert(is_column_visible(&state_value, &c.id)),
                })
                .collect();
            let next = bindings.clone();
            cx.with_state(DataTableToolbarState::default, |st| {
                st.column_visibility = next
            });
        }

        let filter_value = cx
            .watch_model(&filter_model)
            .layout()
            .cloned()
            .unwrap_or_default();
        sync_global_filter(&mut *cx.app, &self.state, &filter_value);

        let desired_visibility: HashMap<ColumnId, bool> = bindings
            .iter()
            .map(|b| {
                (
                    b.id.clone(),
                    cx.watch_model(&b.model).layout().copied().unwrap_or(true),
                )
            })
            .collect();
        sync_column_visibility(&mut *cx.app, &self.state, &desired_visibility);

        let selected_count = state_value.row_selection.len();
        let theme = Theme::global(&*cx.app).clone();

        let column_label = Arc::clone(&self.column_label);
        let columns = Arc::clone(&self.columns);
        let visibility_items: Vec<DropdownMenuEntry> = bindings
            .iter()
            .filter_map(|b| {
                let col = columns.iter().find(|c| c.id.as_ref() == b.id.as_ref())?;
                let label = (column_label)(col);
                Some(DropdownMenuEntry::CheckboxItem(
                    DropdownMenuCheckboxItem::new(b.model.clone(), label),
                ))
            })
            .collect();

        let cols_menu = DropdownMenu::new(columns_open).into_element(
            cx,
            |cx| {
                Button::new("Columns")
                    .variant(ButtonVariant::Outline)
                    .size(ButtonSize::Sm)
                    .into_element(cx)
            },
            move |_cx| visibility_items.clone(),
        );

        let filter = Input::new(filter_model)
            .placeholder(self.filter_placeholder.clone())
            .into_element(cx);

        let selected_text: Option<AnyElement> = (selected_count > 0).then(|| {
            cx.text_props(TextProps {
                layout: LayoutStyle::default(),
                text: Arc::from(format!("Selected: {selected_count}")),
                style: None,
                color: theme.color_by_key("muted-foreground"),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
            })
        });

        hstack(
            cx,
            HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .items_center()
                .gap_x(Space::N2),
            move |_cx| {
                let mut children = vec![filter, cols_menu];
                if let Some(sel) = selected_text.clone() {
                    children.push(sel);
                }
                children
            },
        )
    }
}

#[derive(Default)]
struct DataTablePaginationState {
    page_size_open: Option<Model<bool>>,
    page_size_value: Option<Model<Option<Arc<str>>>>,
    last_synced_page_size: Option<usize>,
}

/// shadcn/ui `DataTable` pagination (recipe).
///
/// This is a v1 surface wired to `TableState.pagination`.
#[derive(Clone)]
pub struct DataTablePagination {
    state: Model<TableState>,
    output: Model<TableViewOutput>,
    page_sizes: Arc<[usize]>,
}

impl std::fmt::Debug for DataTablePagination {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataTablePagination")
            .field("page_sizes_len", &self.page_sizes.len())
            .finish_non_exhaustive()
    }
}

impl DataTablePagination {
    pub fn new(state: Model<TableState>, output: Model<TableViewOutput>) -> Self {
        Self {
            state,
            output,
            page_sizes: Arc::from([10usize, 20, 50, 100]),
        }
    }

    pub fn page_sizes(mut self, sizes: impl Into<Arc<[usize]>>) -> Self {
        self.page_sizes = sizes.into();
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let state_value = cx
            .watch_model(&self.state)
            .layout()
            .cloned()
            .unwrap_or_default();
        let output_value = cx
            .watch_model(&self.output)
            .layout()
            .cloned()
            .unwrap_or_default();

        let page_size_open = cx.with_state(DataTablePaginationState::default, |st| {
            st.page_size_open.clone()
        });
        let page_size_open = match page_size_open {
            Some(m) => m,
            None => {
                let m = cx.app.models_mut().insert(false);
                let m_for_state = m.clone();
                cx.with_state(DataTablePaginationState::default, move |st| {
                    st.page_size_open = Some(m_for_state);
                });
                m
            }
        };

        let page_size_value = cx.with_state(DataTablePaginationState::default, |st| {
            st.page_size_value.clone()
        });
        let page_size_value = match page_size_value {
            Some(m) => m,
            None => {
                let m = cx.app.models_mut().insert(None::<Arc<str>>);
                let m_for_state = m.clone();
                cx.with_state(DataTablePaginationState::default, move |st| {
                    st.page_size_value = Some(m_for_state);
                });
                m
            }
        };

        let current_size = state_value.pagination.page_size;
        let current_size_str: Arc<str> = Arc::from(current_size.to_string());

        let selected_value = cx
            .watch_model(&page_size_value)
            .layout()
            .cloned()
            .unwrap_or(None);

        let last_synced_page_size = cx.with_state(DataTablePaginationState::default, |st| {
            st.last_synced_page_size
        });

        // Treat `TableState.pagination.page_size` as the source of truth. The dropdown's internal
        // model must follow external updates (e.g. programmatic page size changes) and only drive
        // `TableState` when the user makes a new selection.
        let should_sync_to_state =
            selected_value.is_none() || last_synced_page_size != Some(current_size);
        if should_sync_to_state {
            let _ = cx
                .app
                .models_mut()
                .update(&page_size_value, |v| *v = Some(current_size_str.clone()));
            cx.with_state(DataTablePaginationState::default, |st| {
                st.last_synced_page_size = Some(current_size);
            });
        } else if let Some(sel) = selected_value {
            match sel.as_ref().parse::<usize>() {
                Ok(next) if next != current_size => {
                    let state = self.state.clone();
                    let _ = cx.app.models_mut().update(&state, |st| {
                        st.pagination.page_size = next;
                        st.pagination.page_index = 0;
                    });
                    cx.with_state(DataTablePaginationState::default, |st| {
                        st.last_synced_page_size = Some(next);
                    });
                }
                Ok(_) => {}
                Err(_) => {
                    let _ = cx
                        .app
                        .models_mut()
                        .update(&page_size_value, |v| *v = Some(current_size_str.clone()));
                    cx.with_state(DataTablePaginationState::default, |st| {
                        st.last_synced_page_size = Some(current_size);
                    });
                }
            }
        }

        let prev_enabled = output_value.pagination.can_prev;
        let next_enabled = output_value.pagination.can_next;
        let prev_on_activate: OnActivate = {
            let state = self.state.clone();
            Arc::new(move |host, _acx, _reason| {
                let _ = host.models_mut().update(&state, |st| {
                    st.pagination.page_index = st.pagination.page_index.saturating_sub(1);
                });
            })
        };
        let next_on_activate: OnActivate = {
            let state = self.state.clone();
            Arc::new(move |host, _acx, _reason| {
                let _ = host.models_mut().update(&state, |st| {
                    st.pagination.page_index = st.pagination.page_index.saturating_add(1);
                });
            })
        };

        let page_label: Arc<str> = if output_value.pagination.page_count == 0 {
            Arc::from("Page 0 / 0")
        } else {
            Arc::from(format!(
                "Page {} / {}",
                output_value.pagination.page_index + 1,
                output_value.pagination.page_count
            ))
        };

        let page_sizes = Arc::clone(&self.page_sizes);
        let page_size_menu = DropdownMenu::new(page_size_open).into_element(
            cx,
            |cx| {
                Button::new(Arc::from(format!("Rows: {current_size}")))
                    .variant(ButtonVariant::Outline)
                    .size(ButtonSize::Sm)
                    .into_element(cx)
            },
            move |_cx| {
                vec![DropdownMenuEntry::RadioGroup({
                    let mut group = DropdownMenuRadioGroup::new(page_size_value);
                    for size in page_sizes.iter().copied() {
                        let value: Arc<str> = Arc::from(size.to_string());
                        group = group.item(DropdownMenuRadioItemSpec::new(value.clone(), value));
                    }
                    group
                })]
            },
        );

        hstack(
            cx,
            HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .items_center()
                .gap_x(Space::N2),
            move |cx| {
                vec![
                    Button::new("Prev")
                        .variant(ButtonVariant::Outline)
                        .size(ButtonSize::Sm)
                        .disabled(!prev_enabled)
                        .on_activate(prev_on_activate.clone())
                        .into_element(cx),
                    Button::new(page_label.clone())
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Sm)
                        .into_element(cx),
                    Button::new("Next")
                        .variant(ButtonVariant::Outline)
                        .size(ButtonSize::Sm)
                        .disabled(!next_enabled)
                        .on_activate(next_on_activate.clone())
                        .into_element(cx),
                    page_size_menu,
                ]
            },
        )
    }
}

#[cfg(test)]
mod tests {
    #[derive(Debug, Clone, PartialEq, Eq)]
    enum PageSizeAction {
        None,
        SyncToState,
        SetToUserSelection(usize),
    }

    fn reconcile_page_size(
        current_size: usize,
        selected_value: Option<&str>,
        last_synced: Option<usize>,
    ) -> PageSizeAction {
        if selected_value.is_none() || last_synced != Some(current_size) {
            return PageSizeAction::SyncToState;
        }

        let Some(sel) = selected_value else {
            return PageSizeAction::SyncToState;
        };

        match sel.parse::<usize>() {
            Ok(next) if next != current_size => PageSizeAction::SetToUserSelection(next),
            Ok(_) => PageSizeAction::None,
            Err(_) => PageSizeAction::SyncToState,
        }
    }

    #[test]
    fn pagination_page_size_is_controlled_by_state() {
        assert_eq!(
            reconcile_page_size(20, None, None),
            PageSizeAction::SyncToState
        );
        assert_eq!(
            reconcile_page_size(50, Some("10"), Some(10)),
            PageSizeAction::SyncToState,
            "external page_size change must win over stale dropdown model"
        );
    }

    #[test]
    fn pagination_page_size_accepts_user_selection() {
        assert_eq!(
            reconcile_page_size(20, Some("50"), Some(20)),
            PageSizeAction::SetToUserSelection(50)
        );
        assert_eq!(
            reconcile_page_size(20, Some("abc"), Some(20)),
            PageSizeAction::SyncToState
        );
    }
}
