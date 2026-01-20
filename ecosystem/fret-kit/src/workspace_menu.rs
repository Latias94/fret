use std::sync::Arc;

use fret_runtime::{
    CommandId, InputContext, InputDispatchPhase, KeymapService, MenuBar, MenuItem, Platform,
    PlatformCapabilities, WhenExpr, WindowInputContextService, format_sequence,
};
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use crate::shadcn::menubar::{
    Menubar, MenubarEntry, MenubarItem, MenubarMenu, MenubarMenuEntries, MenubarShortcut,
};

#[derive(Debug, Clone)]
pub struct MenubarFromRuntimeOptions {
    pub platform: Platform,
    pub include_shortcuts: bool,
}

impl MenubarFromRuntimeOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn platform(mut self, platform: Platform) -> Self {
        self.platform = platform;
        self
    }

    pub fn include_shortcuts(mut self, include: bool) -> Self {
        self.include_shortcuts = include;
        self
    }
}

/// Render a shadcn menubar from the data-only `fret-runtime` [`MenuBar`].
///
/// This is a convenience bridge so apps can keep menu structure derived from commands (ADR 0023)
/// while still rendering editor-style chrome using shadcn recipes.
pub fn menubar_from_runtime<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    menu_bar: &MenuBar,
    opts: MenubarFromRuntimeOptions,
) -> AnyElement {
    let menus: Vec<MenubarMenuEntries> = menu_bar
        .menus
        .iter()
        .map(|menu| {
            let entries = menu
                .items
                .iter()
                .flat_map(|item| menu_entries(cx, item, &opts))
                .collect();
            MenubarMenu::new(menu.title.clone()).entries(entries)
        })
        .collect();

    Menubar::new(menus).into_element(cx)
}

fn menu_entries<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    item: &MenuItem,
    opts: &MenubarFromRuntimeOptions,
) -> Vec<MenubarEntry> {
    match item {
        MenuItem::Separator => vec![MenubarEntry::Separator],
        MenuItem::SystemMenu { .. } => Vec::new(),
        MenuItem::Command { command, when } => {
            vec![MenubarEntry::Item(command_entry(
                cx,
                command,
                when.as_ref(),
                opts,
            ))]
        }
        MenuItem::Submenu { title, when, items } => {
            let base_ctx = menu_shortcut_input_context(cx, opts.platform);
            let disabled = when.as_ref().is_some_and(|w| !w.eval(&base_ctx));
            let trigger = MenubarItem::new(title.clone()).disabled(disabled);
            let mut out = Vec::new();
            for item in items {
                out.extend(menu_entries(cx, item, opts));
            }
            vec![MenubarEntry::Submenu(trigger.submenu(out))]
        }
    }
}

fn command_entry<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    command: &CommandId,
    item_when: Option<&WhenExpr>,
    opts: &MenubarFromRuntimeOptions,
) -> MenubarItem {
    let base_ctx = menu_shortcut_input_context(cx, opts.platform);

    let (label, shortcut, meta_disabled) = match cx.app.commands().get(command.clone()) {
        Some(meta) => {
            let label = meta.title.clone();
            let shortcut = if opts.include_shortcuts {
                cx.app
                    .global::<KeymapService>()
                    .and_then(|svc| {
                        svc.keymap
                            .display_shortcut_for_command_sequence(&base_ctx, command)
                    })
                    .or_else(|| {
                        meta.default_keybindings
                            .iter()
                            .find(|kb| kb.platform.matches(opts.platform))
                            .map(|kb| kb.sequence.clone())
                    })
                    .map(|seq| Arc::<str>::from(format_sequence(opts.platform, &seq)))
            } else {
                None
            };
            let meta_disabled = meta.when.as_ref().is_some_and(|w| !w.eval(&base_ctx));
            (label, shortcut, meta_disabled)
        }
        None => (Arc::<str>::from(command.as_str()), None, false),
    };

    let item_disabled = item_when.is_some_and(|w| !w.eval(&base_ctx));
    let disabled = meta_disabled || item_disabled;

    let mut item = MenubarItem::new(label)
        .value(command.as_str())
        .disabled(disabled);
    item.command = Some(command.clone());
    if let Some(shortcut) = shortcut {
        item.trailing = Some(MenubarShortcut::new(shortcut).into_element(cx));
    }
    item
}

fn menu_shortcut_input_context<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    platform: Platform,
) -> InputContext {
    let caps = cx
        .app
        .global::<PlatformCapabilities>()
        .cloned()
        .unwrap_or_default();

    let snapshot = cx
        .app
        .global::<WindowInputContextService>()
        .and_then(|svc| svc.snapshot(cx.window))
        .cloned();

    let mut ctx = snapshot.unwrap_or(InputContext {
        platform,
        caps,
        ui_has_modal: false,
        focus_is_text_input: false,
        edit_can_undo: true,
        edit_can_redo: true,
        dispatch_phase: InputDispatchPhase::Normal,
    });

    // Menubar shortcut labels should be stable and platform-scoped; treat the snapshot as an input
    // for enablement/gating, but keep platform formatting consistent with the caller's intent.
    ctx.platform = platform;
    ctx.dispatch_phase = InputDispatchPhase::Normal;
    ctx
}

impl Default for MenubarFromRuntimeOptions {
    fn default() -> Self {
        Self {
            platform: Platform::current(),
            include_shortcuts: true,
        }
    }
}
