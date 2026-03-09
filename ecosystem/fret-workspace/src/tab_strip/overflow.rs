use std::collections::HashMap;
use std::sync::Arc;

use fret_core::{Color, Px, Rect, TextStyle};
use fret_runtime::Model;
use fret_ui::{ElementContext, UiHost};
use fret_ui_headless::tab_strip_overflow::compute_overflowed_tab_indices;
use fret_ui_headless::tab_strip_overflow_menu::{
    OverflowMenuActivePolicy, OverflowMenuEmptyOverflowedPolicy, compute_overflow_menu_item_indices,
};
use fret_ui_shadcn::{DropdownMenuEntry, DropdownMenuItem};

use super::WorkspaceTab;
use super::state::WorkspaceTabStripRevealHint;
use crate::tab_drag::WorkspaceTabHitRect;

fn compute_overflowed_tab_indices_for_tabs(
    tabs: &[WorkspaceTab],
    tab_rects: &[WorkspaceTabHitRect],
    viewport: Rect,
    margin: Px,
) -> Vec<usize> {
    let by_id: HashMap<&str, Rect> = tab_rects.iter().map(|r| (r.id.as_ref(), r.rect)).collect();
    compute_overflowed_tab_indices(
        tabs,
        |tab| by_id.get(tab.id.as_ref()).copied(),
        viewport,
        margin,
    )
}

pub(crate) fn compute_overflow_menu_entries<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    root_test_id: Option<&Arc<str>>,
    active: Option<&Arc<str>>,
    active_policy: OverflowMenuActivePolicy,
    tabs: &[WorkspaceTab],
    tab_rects: &[WorkspaceTabHitRect],
    viewport: Option<Rect>,
    is_overflowing: bool,
    reveal_hint_model: Model<WorkspaceTabStripRevealHint>,
    text_style: TextStyle,
    inactive_fg: Color,
) -> (Option<Arc<str>>, Vec<DropdownMenuEntry>) {
    let button_test_id = root_test_id.map(|id| Arc::<str>::from(format!("{id}.overflow_button")));

    let active_index = active.and_then(|active| {
        tabs.iter()
            .position(|tab| tab.id.as_ref() == active.as_ref())
    });

    let overflowed_indices = viewport
        .map(|viewport| compute_overflowed_tab_indices_for_tabs(tabs, tab_rects, viewport, Px(2.0)))
        .unwrap_or_default();
    let item_indices = compute_overflow_menu_item_indices(
        tabs.len(),
        &overflowed_indices,
        active_index,
        active_policy,
        if is_overflowing {
            OverflowMenuEmptyOverflowedPolicy::AllTabs
        } else {
            OverflowMenuEmptyOverflowedPolicy::Empty
        },
    );

    let entries = item_indices
        .into_iter()
        .filter_map(|tab_ix| {
            let tab = tabs.get(tab_ix)?;
            let test_id = root_test_id
                .map(|root| Arc::<str>::from(format!("{root}.overflow_entry.{}", tab.id.as_ref())));
            let close_test_id = root_test_id.map(|root| {
                Arc::<str>::from(format!("{root}.overflow_entry.{}.close", tab.id.as_ref()))
            });

            let close_slot = tab.close_command.as_ref().map(|_cmd| {
                super::widgets::overflow_menu_close_slot(
                    cx,
                    text_style.clone(),
                    inactive_fg,
                    close_test_id,
                )
            });

            let mut item = DropdownMenuItem::new(tab.title.clone())
                .close_on_select(true)
                .on_activate({
                    let reveal_hint_model = reveal_hint_model.clone();
                    let tab_id = tab.id.clone();
                    Arc::new(move |host, _acx, reason| {
                        let _ = host.models_mut().update(&reveal_hint_model, |st| {
                            st.tab_id = Some(tab_id.clone());
                            st.reason = Some(reason);
                        });
                    })
                })
                .action(tab.command.clone());
            if let Some(id) = test_id {
                item = item.test_id(id);
            }
            if let Some(close_cmd) = tab.close_command.clone() {
                item = item.trailing_action(close_cmd);
            }
            if let Some(close_slot) = close_slot {
                item = item.trailing(close_slot);
            }
            Some(DropdownMenuEntry::Item(item))
        })
        .collect::<Vec<_>>();

    (button_test_id, entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::{Point, Size};

    fn rect(x: f32, w: f32) -> Rect {
        Rect::new(Point::new(Px(x), Px(0.0)), Size::new(Px(w), Px(10.0)))
    }

    #[test]
    fn overflow_membership_tracks_viewport_and_margin() {
        let tabs = vec![
            WorkspaceTab::new("a", "A", "cmd.a"),
            WorkspaceTab::new("b", "B", "cmd.b"),
            WorkspaceTab::new("c", "C", "cmd.c"),
            WorkspaceTab::new("d", "D", "cmd.d"),
        ];
        let rects = vec![
            WorkspaceTabHitRect {
                id: Arc::from("a"),
                rect: rect(0.0, 25.0),
            },
            WorkspaceTabHitRect {
                id: Arc::from("b"),
                rect: rect(25.0, 25.0),
            },
            WorkspaceTabHitRect {
                id: Arc::from("c"),
                rect: rect(50.0, 25.0),
            },
            WorkspaceTabHitRect {
                id: Arc::from("d"),
                rect: rect(75.0, 30.0),
            },
        ];
        let viewport = rect(0.0, 100.0);

        let overflowed = compute_overflowed_tab_indices_for_tabs(&tabs, &rects, viewport, Px(0.0))
            .into_iter()
            .filter_map(|ix| tabs.get(ix).map(|t| t.id.clone()))
            .collect::<Vec<_>>();
        assert_eq!(
            overflowed,
            vec![Arc::from("d")],
            "expected right-clipped tab to be considered overflowed"
        );

        let overflowed = compute_overflowed_tab_indices_for_tabs(&tabs, &rects, viewport, Px(10.0))
            .into_iter()
            .filter_map(|ix| tabs.get(ix).map(|t| t.id.clone()))
            .collect::<Vec<_>>();
        assert_eq!(
            overflowed,
            vec![Arc::from("a"), Arc::from("d")],
            "expected margin to treat edge-adjacent tabs as overflowed"
        );
    }

    #[test]
    fn overflow_membership_changes_with_scroll_like_translation() {
        let tabs = vec![
            WorkspaceTab::new("a", "A", "cmd.a"),
            WorkspaceTab::new("b", "B", "cmd.b"),
            WorkspaceTab::new("c", "C", "cmd.c"),
            WorkspaceTab::new("d", "D", "cmd.d"),
        ];

        // Simulate a scroll offset shifting all tabs left by 40px.
        let rects = vec![
            WorkspaceTabHitRect {
                id: Arc::from("a"),
                rect: rect(-40.0, 25.0),
            },
            WorkspaceTabHitRect {
                id: Arc::from("b"),
                rect: rect(-15.0, 25.0),
            },
            WorkspaceTabHitRect {
                id: Arc::from("c"),
                rect: rect(10.0, 25.0),
            },
            WorkspaceTabHitRect {
                id: Arc::from("d"),
                rect: rect(35.0, 30.0),
            },
        ];
        let viewport = rect(0.0, 60.0);

        let overflowed = compute_overflowed_tab_indices_for_tabs(&tabs, &rects, viewport, Px(0.0))
            .into_iter()
            .filter_map(|ix| tabs.get(ix).map(|t| t.id.clone()))
            .collect::<Vec<_>>();
        assert_eq!(
            overflowed,
            vec![Arc::from("a"), Arc::from("b"), Arc::from("d")],
            "expected left-clipped and right-clipped tabs to be overflowed"
        );
    }
}
