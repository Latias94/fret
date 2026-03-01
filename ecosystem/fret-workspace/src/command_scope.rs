use std::sync::Arc;

use fret_core::Px;
use fret_runtime::Model;
use fret_ui::element::{AnyElement, ContainerProps, LayoutStyle, Length};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

use crate::commands::CMD_WORKSPACE_PANE_FOCUS_TAB_STRIP;
use crate::focus_registry::{WorkspaceTabElementKey, workspace_tab_element_registry_model};
use crate::layout::WorkspaceWindowLayout;

fn fill_layout() -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Fill;
    layout.size.min_width = Some(Length::Px(Px(0.0)));
    layout.size.min_height = Some(Length::Px(Px(0.0)));
    layout
}

/// Workspace-shell command routing scope.
///
/// This is intended for editor-like shells where pointer interactions should not steal focus from
/// the content surface, but keyboard users still need deterministic focus transfer commands.
///
/// In particular, it handles `workspace.pane.focus_tab_strip` by focusing the active tab in the
/// active pane's `WorkspaceTabStrip` (best-effort, gated by unit tests).
#[derive(Debug)]
pub struct WorkspaceCommandScope {
    window_layout: Model<WorkspaceWindowLayout>,
    child: AnyElement,
}

impl WorkspaceCommandScope {
    pub fn new(window_layout: Model<WorkspaceWindowLayout>, child: AnyElement) -> Self {
        Self {
            window_layout,
            child,
        }
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let window_layout = self.window_layout;
        let child = self.child;
        let tab_element_registry = workspace_tab_element_registry_model(cx);

        let root = cx.container(
            ContainerProps {
                layout: fill_layout(),
                ..Default::default()
            },
            move |_cx| vec![child],
        );

        let window_layout_for_command = window_layout.clone();
        let tab_element_registry_for_command = tab_element_registry.clone();
        cx.command_on_command_for(
            root.id,
            Arc::new(move |host, acx, command| {
                if command.as_str() != CMD_WORKSPACE_PANE_FOCUS_TAB_STRIP {
                    return false;
                }

                let active = host.models_mut().read(&window_layout_for_command, |w| {
                    let pane_id = w.active_pane_id().cloned()?;
                    let pane = w.pane_tree.find_pane(pane_id.as_ref())?;
                    let tab_id = pane.tabs.active().cloned()?;
                    Some((pane_id, tab_id))
                });
                let Some((pane_id, tab_id)) = active.ok().flatten() else {
                    return false;
                };

                let key = WorkspaceTabElementKey {
                    window: acx.window,
                    pane_id: Some(pane_id),
                    tab_id,
                };

                let target: Option<GlobalElementId> = host
                    .models_mut()
                    .read(&tab_element_registry_for_command, |reg| reg.get(&key))
                    .ok()
                    .flatten();
                let Some(target) = target else {
                    return false;
                };

                host.request_focus(target);
                host.request_redraw(acx.window);
                true
            }),
        );

        root
    }
}
