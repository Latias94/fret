use std::sync::Arc;

use fret_runtime::{CommandRegistry, MenuBar};
use fret_ui::element::{AnyElement, ContainerProps, FlexProps, LayoutStyle, Length};
use fret_ui::{ElementContext, UiHost};

use crate::workspace::layout::WorkspacePaneLayout;
use crate::workspace::{
    WorkspaceFrame, WorkspaceTabStrip, WorkspaceTopBar, workspace_pane_tree_element,
};

use crate::workspace_menu::{MenubarFromRuntimeOptions, menubar_from_runtime};

fn fill_layout() -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Fill;
    layout
}

/// Compose an editor-style workspace shell:
/// - optional menubar (rendered via shadcn recipes),
/// - pane tree (multi-pane layout),
/// - per-pane tab strip.
///
/// Apps remain responsible for:
/// - providing a stable doc-id -> title mapping,
/// - rendering each pane's main content.
pub fn workspace_shell<H: UiHost, FTitle, FPane>(
    cx: &mut ElementContext<'_, H>,
    menu_bar: Option<&MenuBar>,
    commands: Option<&CommandRegistry>,
    window: &crate::workspace::layout::WorkspaceWindowLayout,
    tab_title: FTitle,
    mut render_pane_content: FPane,
) -> AnyElement
where
    FTitle: Fn(&str) -> Arc<str> + Clone,
    FPane: FnMut(&mut ElementContext<'_, H>, &WorkspacePaneLayout, bool) -> AnyElement,
{
    let top = menu_bar
        .map(|bar| menubar_from_runtime(cx, bar, commands, MenubarFromRuntimeOptions::default()));

    let active_pane = window.active_pane_id().map(|id| id.as_ref());
    let center = workspace_pane_tree_element(
        cx,
        &window.pane_tree,
        active_pane,
        &mut |cx, pane, is_active| {
            let tab_title = tab_title.clone();
            let tab_strip =
                WorkspaceTabStrip::from_workspace_tabs(&pane.tabs, tab_title).into_element(cx);
            let content = render_pane_content(cx, pane, is_active);

            cx.flex(
                FlexProps {
                    layout: fill_layout(),
                    direction: fret_core::Axis::Vertical,
                    ..Default::default()
                },
                |_cx| vec![tab_strip, content],
            )
        },
    );

    let mut frame = WorkspaceFrame::new(center);
    if let Some(menu) = top {
        frame = frame.top(
            WorkspaceTopBar::new()
                .left([cx.container(
                    ContainerProps {
                        layout: fill_layout(),
                        ..Default::default()
                    },
                    |_cx| vec![menu],
                )])
                .into_element(cx),
        );
    }

    frame.into_element(cx)
}
