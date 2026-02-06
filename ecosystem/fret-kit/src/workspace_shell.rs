use std::sync::Arc;

use fret_runtime::{CommandId, InputContext, MenuBar, Model, Platform, WindowMenuBarFocusService};
use fret_ui::element::{AnyElement, ContainerProps, FlexProps, LayoutStyle, Length, StackProps};
use fret_ui::{ElementContext, GlobalElementId, PendingShortcutOverlayState, UiHost};

use crate::workspace::layout::WorkspacePaneLayout;
use crate::workspace::{
    WorkspaceFrame, WorkspaceTabStrip, WorkspaceTopBar, workspace_pane_tree_element_with_resize,
};

use crate::pending_shortcut_overlay::pending_shortcut_hint_overlay;
use crate::workspace_menu::{
    InWindowMenubarFocusHandle, MenubarFromRuntimeOptions, menubar_from_runtime_with_focus_handle,
};
use fret_ui_kit::primitives::menubar::trigger_row as menubar_trigger_row;

fn fill_layout() -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Fill;
    layout
}

/// Model-driven workspace shell.
///
/// This enables in-place UI mutations (e.g. split drag-to-resize) without requiring apps to route
/// those interactions through commands.
pub fn workspace_shell_model<H: UiHost, FTitle, FPane>(
    cx: &mut ElementContext<'_, H>,
    menu_bar: Option<&MenuBar>,
    window: Model<crate::workspace::layout::WorkspaceWindowLayout>,
    tab_title: FTitle,
    mut render_pane_content: FPane,
) -> AnyElement
where
    FTitle: Fn(&str) -> Arc<str> + Clone,
    FPane: FnMut(&mut ElementContext<'_, H>, &WorkspacePaneLayout, bool) -> AnyElement,
{
    cx.app
        .with_global_mut(WindowMenuBarFocusService::default, |svc, _app| {
            svc.set_present(cx.window, menu_bar.is_some());
        });

    cx.keyed("workspace_shell.command_scope", |cx| {
        let shell_root = cx.root_id();
        let menubar_handle: std::cell::RefCell<Option<InWindowMenubarFocusHandle>> =
            std::cell::RefCell::new(None);

        let top = menu_bar.map(|bar| {
            cx.keyed("workspace_shell.menubar", |cx| {
                let (menu, handle) = menubar_from_runtime_with_focus_handle(
                    cx,
                    bar,
                    MenubarFromRuntimeOptions::default(),
                );
                *menubar_handle.borrow_mut() = Some(handle);
                menu
            })
        });

        if let Some(handle) = menubar_handle.borrow().clone() {
            let group_active = handle.group_active.clone();
            let trigger_registry = handle.trigger_registry.clone();
            let last_focus_before_menubar = handle.last_focus_before_menubar.clone();
            let focus_is_trigger = handle.focus_is_trigger.clone();
            let group_active_for_command = group_active.clone();
            let trigger_registry_for_command = trigger_registry.clone();
            let last_focus_for_command = last_focus_before_menubar.clone();
            cx.command_add_on_command_for(
                shell_root,
                Arc::new(move |host, acx, command| {
                    if command.as_str() != fret_app::core_commands::FOCUS_MENU_BAR {
                        return false;
                    }

                    let active = host
                        .models_mut()
                        .get_cloned(&group_active_for_command)
                        .flatten();
                    if let Some(active) = active {
                        let _ = host.models_mut().update(&active.open, |v| *v = false);
                        let _ = host
                            .models_mut()
                            .update(&group_active_for_command, |v| *v = None);
                        let restore = host
                            .models_mut()
                            .get_cloned(&last_focus_for_command)
                            .flatten();
                        host.request_focus(restore.unwrap_or(active.trigger));
                        host.request_redraw(acx.window);
                        return true;
                    }

                    let entries = host
                        .models_mut()
                        .get_cloned(&trigger_registry_for_command)
                        .unwrap_or_default();
                    let target = entries.iter().find(|e| e.enabled).cloned();
                    let Some(target) = target else {
                        return false;
                    };

                    let open_for_state = target.open.clone();
                    let _ = host.models_mut().update(&group_active_for_command, |v| {
                        *v = Some(menubar_trigger_row::MenubarActiveTrigger {
                            trigger: target.trigger,
                            open: open_for_state,
                        });
                    });

                    host.request_focus(target.trigger);
                    host.request_redraw(acx.window);
                    true
                }),
            );

            cx.key_add_on_key_down_for(
                shell_root,
                menubar_trigger_row::open_on_alt_mnemonic(
                    group_active.clone(),
                    trigger_registry.clone(),
                ),
            );
            cx.key_add_on_key_down_for(
                shell_root,
                menubar_trigger_row::open_on_mnemonic_when_active(
                    group_active.clone(),
                    trigger_registry.clone(),
                    focus_is_trigger.clone(),
                ),
            );
            cx.key_add_on_key_down_for(
                shell_root,
                menubar_trigger_row::exit_active_on_escape_when_closed(
                    group_active.clone(),
                    last_focus_before_menubar.clone(),
                    focus_is_trigger.clone(),
                ),
            );
        }

        let mut topbar_anchor_id: Option<GlobalElementId> = None;

        let center = workspace_pane_tree_element_with_resize(
            cx,
            window.clone(),
            &mut |cx, pane, is_active, tab_drag| {
                let tab_title = tab_title.clone();
                let tab_strip = WorkspaceTabStrip::from_workspace_tabs(&pane.tabs, tab_title)
                    .pane_id(pane.id.clone())
                    .tab_drag_model(tab_drag.clone())
                    .into_element(cx);
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
            let topbar = cx.keyed("workspace_shell.topbar_anchor", |cx| {
                topbar_anchor_id = Some(cx.root_id());

                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Fill;

                cx.container(
                    ContainerProps {
                        layout,
                        ..Default::default()
                    },
                    |cx| {
                        vec![
                            WorkspaceTopBar::new()
                                .left([cx.container(
                                    ContainerProps {
                                        layout: fill_layout(),
                                        ..Default::default()
                                    },
                                    |_cx| vec![menu],
                                )])
                                .into_element(cx),
                        ]
                    },
                )
            });

            frame = frame.top(topbar);
        }

        let (pending_input_ctx, pending_sequence, pending_continuations) = cx
            .app
            .global::<PendingShortcutOverlayState>()
            .and_then(|s| {
                s.snapshot_for_window(cx.window)
                    .map(|(ctx, seq, cont)| (ctx.clone(), seq.to_vec(), cont.to_vec()))
            })
            .unwrap_or_else(|| (InputContext::default(), Vec::new(), Vec::new()));

        let top_inset = topbar_anchor_id
            .and_then(|id| cx.last_bounds_for_element(id))
            .map(|bounds| bounds.origin.y + bounds.size.height + fret_core::Px(8.0))
            .unwrap_or(fret_core::Px(40.0));

        let frame = frame.into_element(cx);
        let overlay = pending_shortcut_hint_overlay(
            cx,
            top_inset,
            &pending_input_ctx,
            &pending_sequence,
            &pending_continuations,
        );
        if let Some(overlay) = overlay {
            return cx.stack_props(
                StackProps {
                    layout: fill_layout(),
                },
                |_cx| vec![frame, overlay],
            );
        }

        frame
    })
}

/// A `workspace_shell_model` convenience wrapper that renders the default editor-style menu bar
/// provided by `fret-workspace`.
pub fn workspace_shell_model_default_menu<H: UiHost, FTitle, FPane>(
    cx: &mut ElementContext<'_, H>,
    window: Model<crate::workspace::layout::WorkspaceWindowLayout>,
    tab_title: FTitle,
    render_pane_content: FPane,
) -> AnyElement
where
    FTitle: Fn(&str) -> Arc<str> + Clone,
    FPane: FnMut(&mut ElementContext<'_, H>, &WorkspacePaneLayout, bool) -> AnyElement,
{
    let mut cmds = crate::workspace::menu::WorkspaceMenuCommands::default();
    if Platform::current() == Platform::Macos {
        cmds.app_menu_title = cx
            .app
            .global::<fret_app::AppDisplayName>()
            .map(|name| name.0.clone())
            .or(Some(Arc::from("App")));
        cmds.include_services_menu = true;
        cmds.about = Some(CommandId::new(fret_app::core_commands::APP_ABOUT));
        cmds.preferences = Some(CommandId::new(fret_app::core_commands::APP_PREFERENCES));
        cmds.hide = Some(CommandId::new(fret_app::core_commands::APP_HIDE));
        cmds.hide_others = Some(CommandId::new(fret_app::core_commands::APP_HIDE_OTHERS));
        cmds.show_all = Some(CommandId::new(fret_app::core_commands::APP_SHOW_ALL));
        cmds.quit_app = Some(CommandId::new(fret_app::core_commands::APP_QUIT));
    }
    let menu_bar = crate::workspace::menu::workspace_default_menu_bar(cmds);

    workspace_shell_model(cx, Some(&menu_bar), window, tab_title, render_pane_content)
}
