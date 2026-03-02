use std::collections::HashMap;
use std::sync::Arc;

use fret_core::{Color, Px, Rect, TextStyle};
use fret_ui::{ElementContext, UiHost};
use fret_ui_headless::tab_strip_overflow::compute_overflowed_tab_indices;
use fret_ui_shadcn::{DropdownMenuEntry, DropdownMenuItem};

use super::WorkspaceTab;
use crate::tab_drag::WorkspaceTabHitRect;

pub(crate) fn compute_overflowed_tab_ids(
    tabs: &[WorkspaceTab],
    tab_rects: &[WorkspaceTabHitRect],
    viewport: Rect,
    margin: Px,
) -> Vec<Arc<str>> {
    let by_id: HashMap<&str, Rect> = tab_rects.iter().map(|r| (r.id.as_ref(), r.rect)).collect();
    compute_overflowed_tab_indices(
        tabs,
        |tab| by_id.get(tab.id.as_ref()).copied(),
        viewport,
        margin,
    )
    .into_iter()
    .filter_map(|ix| tabs.get(ix).map(|tab| tab.id.clone()))
    .collect()
}

pub(crate) fn compute_overflow_menu_entries<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    root_test_id: Option<&Arc<str>>,
    tabs: &[WorkspaceTab],
    tab_rects: &[WorkspaceTabHitRect],
    viewport: Option<Rect>,
    is_overflowing: bool,
    text_style: TextStyle,
    inactive_fg: Color,
) -> (Option<Arc<str>>, Vec<DropdownMenuEntry>) {
    let button_test_id = root_test_id.map(|id| Arc::<str>::from(format!("{id}.overflow_button")));

    let overflowed = viewport
        .map(|viewport| compute_overflowed_tab_ids(tabs, tab_rects, viewport, Px(2.0)))
        .unwrap_or_default();
    let overflowed = if is_overflowing && overflowed.is_empty() {
        tabs.iter().map(|tab| tab.id.clone()).collect()
    } else {
        overflowed
    };

    let entries = overflowed
        .iter()
        .filter_map(|id| {
            let tab_ix = tabs.iter().position(|t| t.id.as_ref() == id.as_ref())?;
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
                .on_select(tab.command.clone());
            if let Some(id) = test_id {
                item = item.test_id(id);
            }
            if let Some(close_cmd) = tab.close_command.clone() {
                item = item.trailing_on_select(close_cmd);
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

        let overflowed = compute_overflowed_tab_ids(&tabs, &rects, viewport, Px(0.0));
        assert_eq!(
            overflowed,
            vec![Arc::from("d")],
            "expected right-clipped tab to be considered overflowed"
        );

        let overflowed = compute_overflowed_tab_ids(&tabs, &rects, viewport, Px(10.0));
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

        let overflowed = compute_overflowed_tab_ids(&tabs, &rects, viewport, Px(0.0));
        assert_eq!(
            overflowed,
            vec![Arc::from("a"), Arc::from("b"), Arc::from("d")],
            "expected left-clipped and right-clipped tabs to be overflowed"
        );
    }
}
