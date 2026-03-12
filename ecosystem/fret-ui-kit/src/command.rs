use std::collections::BTreeMap;
use std::sync::Arc;

use fret_runtime::{
    ActionId, CommandId, CommandMeta, InputContext, InputDispatchPhase, KeymapService, Platform,
    PlatformCapabilities, WindowCommandGatingSnapshot, format_sequence,
};
use fret_ui::{ElementContext, UiHost};

pub fn default_fallback_input_context<H: UiHost>(app: &H) -> InputContext {
    let caps = app
        .global::<PlatformCapabilities>()
        .cloned()
        .unwrap_or_default();
    InputContext::fallback(Platform::current(), caps)
}

/// Best-effort input context for global command discovery surfaces such as command palettes.
pub fn command_palette_input_context<H: UiHost>(app: &H) -> InputContext {
    let caps = app
        .global::<PlatformCapabilities>()
        .cloned()
        .unwrap_or_default();
    InputContext {
        platform: Platform::current(),
        caps,
        ui_has_modal: true,
        window_arbitration: None,
        focus_is_text_input: false,
        text_boundary_mode: fret_runtime::TextBoundaryMode::UnicodeWord,
        edit_can_undo: true,
        edit_can_redo: true,
        router_can_back: false,
        router_can_forward: false,
        dispatch_phase: InputDispatchPhase::Bubble,
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CommandCatalogOptions {
    /// When `true`, commands that fail their `when` gating are excluded instead of being rendered
    /// as disabled rows.
    pub hide_disabled: bool,
}

/// Data-only command item derived from host command metadata and current window gating state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandCatalogItem {
    pub label: Arc<str>,
    pub value: Arc<str>,
    pub disabled: bool,
    pub keywords: Vec<Arc<str>>,
    pub shortcut: Option<Arc<str>>,
    pub command: CommandId,
}

impl CommandCatalogItem {
    pub fn new(label: impl Into<Arc<str>>, command: impl Into<CommandId>) -> Self {
        let label = label.into();
        Self {
            label,
            value: Arc::from(""),
            disabled: false,
            keywords: Vec::new(),
            shortcut: None,
            command: command.into(),
        }
    }

    pub fn value(mut self, value: impl Into<Arc<str>>) -> Self {
        self.value = trimmed_arc(value.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn keywords<I, S>(mut self, keywords: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<Arc<str>>,
    {
        self.keywords = keywords
            .into_iter()
            .map(|keyword| trimmed_arc(keyword.into()))
            .collect();
        self
    }

    pub fn shortcut(mut self, shortcut: impl Into<Arc<str>>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }
}

/// Data-only command catalog group. Group ownership belongs to component-policy layers, not to a
/// specific recipe crate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandCatalogGroup {
    pub heading: Arc<str>,
    pub items: Vec<CommandCatalogItem>,
}

impl CommandCatalogGroup {
    pub fn new(
        heading: impl Into<Arc<str>>,
        items: impl IntoIterator<Item = CommandCatalogItem>,
    ) -> Self {
        Self {
            heading: heading.into(),
            items: items.into_iter().collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandCatalogEntry {
    Item(CommandCatalogItem),
    Group(CommandCatalogGroup),
}

pub fn command_catalog_entries_from_host_commands<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Vec<CommandCatalogEntry> {
    command_catalog_entries_from_host_commands_with_options(cx, CommandCatalogOptions::default())
}

pub fn command_catalog_entries_from_host_commands_with_options<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    options: CommandCatalogOptions,
) -> Vec<CommandCatalogEntry> {
    let fallback_input_ctx = command_palette_input_context(&*cx.app);
    let snapshot = fret_runtime::best_effort_snapshot_for_window_with_input_ctx_fallback(
        &*cx.app,
        cx.window,
        fallback_input_ctx,
    );

    let mut input_ctx = snapshot.input_ctx().clone();
    input_ctx.ui_has_modal = true;
    input_ctx.focus_is_text_input = false;
    input_ctx.dispatch_phase = InputDispatchPhase::Bubble;

    let gating = snapshot.with_input_ctx(input_ctx);
    command_catalog_entries_from_host_commands_with_gating_snapshot(cx, options, &gating)
}

pub fn command_catalog_entries_from_host_commands_with_gating_snapshot<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    options: CommandCatalogOptions,
    gating: &WindowCommandGatingSnapshot,
) -> Vec<CommandCatalogEntry> {
    let mut commands: Vec<(CommandId, CommandMeta)> = cx
        .app
        .commands()
        .iter()
        .filter_map(|(id, meta)| (!meta.hidden).then_some((id.clone(), meta.clone())))
        .collect();

    commands.sort_by(|(a_id, a_meta), (b_id, b_meta)| {
        match (&a_meta.category, &b_meta.category) {
            (None, Some(_)) => std::cmp::Ordering::Less,
            (Some(_), None) => std::cmp::Ordering::Greater,
            (Some(a), Some(b)) => a.as_ref().cmp(b.as_ref()),
            (None, None) => std::cmp::Ordering::Equal,
        }
        .then_with(|| a_meta.title.as_ref().cmp(b_meta.title.as_ref()))
        .then_with(|| a_id.as_str().cmp(b_id.as_str()))
    });

    let mut root_items: Vec<CommandCatalogItem> = Vec::new();
    let mut groups: BTreeMap<Arc<str>, Vec<CommandCatalogItem>> = BTreeMap::new();

    for (id, meta) in &commands {
        let disabled = !gating.is_enabled_for_command(id, meta);
        if disabled && options.hide_disabled {
            continue;
        }

        let item = command_catalog_item_from_meta_with_gating(cx, gating, id, meta);
        if let Some(category) = meta.category.clone() {
            groups.entry(category).or_default().push(item);
        } else {
            root_items.push(item);
        }
    }

    let mut entries: Vec<CommandCatalogEntry> = Vec::new();
    entries.extend(root_items.into_iter().map(CommandCatalogEntry::Item));
    entries.extend(groups.into_iter().map(|(heading, items)| {
        CommandCatalogEntry::Group(CommandCatalogGroup::new(heading, items))
    }));
    entries
}

pub trait ElementCommandGatingExt {
    fn command_is_enabled(&self, command: &CommandId) -> bool;
    fn command_is_enabled_with_fallback_input_context(
        &self,
        command: &CommandId,
        fallback_input_ctx: InputContext,
    ) -> bool;

    fn dispatch_command_if_enabled(&mut self, command: CommandId) -> bool;
    fn dispatch_command_if_enabled_with_fallback_input_context(
        &mut self,
        command: CommandId,
        fallback_input_ctx: InputContext,
    ) -> bool;

    /// Action-first naming parity: `ActionId` uses the same ID strings as `CommandId` in v1.
    fn action_is_enabled(&self, action: &ActionId) -> bool;

    /// Action-first naming parity: dispatch an `ActionId` if enabled.
    fn dispatch_action_if_enabled(&mut self, action: ActionId) -> bool;
}

impl<H: UiHost> ElementCommandGatingExt for ElementContext<'_, H> {
    fn command_is_enabled(&self, command: &CommandId) -> bool {
        let fallback_input_ctx = default_fallback_input_context(&*self.app);
        fret_runtime::command_is_enabled_for_window_with_input_ctx_fallback(
            &*self.app,
            self.window,
            command,
            fallback_input_ctx,
        )
    }

    fn command_is_enabled_with_fallback_input_context(
        &self,
        command: &CommandId,
        fallback_input_ctx: InputContext,
    ) -> bool {
        fret_runtime::command_is_enabled_for_window_with_input_ctx_fallback(
            &*self.app,
            self.window,
            command,
            fallback_input_ctx,
        )
    }

    fn dispatch_command_if_enabled(&mut self, command: CommandId) -> bool {
        let fallback_input_ctx = default_fallback_input_context(&*self.app);
        self.dispatch_command_if_enabled_with_fallback_input_context(command, fallback_input_ctx)
    }

    fn dispatch_command_if_enabled_with_fallback_input_context(
        &mut self,
        command: CommandId,
        fallback_input_ctx: InputContext,
    ) -> bool {
        if !fret_runtime::command_is_enabled_for_window_with_input_ctx_fallback(
            &*self.app,
            self.window,
            &command,
            fallback_input_ctx,
        ) {
            return false;
        }
        self.app.push_effect(fret_runtime::Effect::Command {
            window: Some(self.window),
            command,
        });
        true
    }

    fn action_is_enabled(&self, action: &ActionId) -> bool {
        self.command_is_enabled(action)
    }

    fn dispatch_action_if_enabled(&mut self, action: ActionId) -> bool {
        self.dispatch_command_if_enabled(action)
    }
}

fn command_catalog_item_from_meta_with_gating<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    gating: &WindowCommandGatingSnapshot,
    id: &CommandId,
    meta: &CommandMeta,
) -> CommandCatalogItem {
    let input_ctx = gating.input_ctx();

    let mut keywords: Vec<Arc<str>> = meta.keywords.clone();
    keywords.push(Arc::from(id.as_str()));
    if let Some(category) = meta.category.as_ref() {
        keywords.push(category.clone());
    }
    if let Some(description) = meta.description.as_ref() {
        keywords.push(description.clone());
    }

    let shortcut = cx
        .app
        .global::<KeymapService>()
        .and_then(|svc| {
            svc.keymap
                .display_shortcut_for_command_sequence_with_key_contexts(
                    input_ctx,
                    gating.key_contexts(),
                    id,
                )
        })
        .map(|seq| Arc::from(format_sequence(input_ctx.platform, &seq)));

    let mut item = CommandCatalogItem::new(meta.title.clone(), id.clone())
        .value(Arc::from(id.as_str()))
        .keywords(keywords)
        .disabled(!gating.is_enabled_for_command(id, meta));
    if let Some(shortcut) = shortcut {
        item = item.shortcut(shortcut);
    }
    item
}

fn trimmed_arc(value: Arc<str>) -> Arc<str> {
    let trimmed = value.trim();
    if trimmed == value.as_ref() {
        value
    } else {
        Arc::<str>::from(trimmed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashMap;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size};
    use fret_runtime::{
        CommandScope, WindowCommandActionAvailabilityService, WindowCommandEnabledService,
    };

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        )
    }

    fn find_item<'a>(
        entries: &'a [CommandCatalogEntry],
        command: &CommandId,
    ) -> Option<&'a CommandCatalogItem> {
        entries.iter().find_map(|entry| match entry {
            CommandCatalogEntry::Item(item) if &item.command == command => Some(item),
            CommandCatalogEntry::Group(group) => {
                group.items.iter().find(|item| &item.command == command)
            }
            _ => None,
        })
    }

    #[test]
    fn host_command_entries_respect_window_command_enabled_overrides() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let cmd = CommandId::from("test.disabled-command");
        app.commands_mut()
            .register(cmd.clone(), CommandMeta::new("Disabled Command"));
        app.set_global(WindowCommandEnabledService::default());
        app.with_global_mut(WindowCommandEnabledService::default, |svc, _app| {
            svc.set_enabled(window, cmd.clone(), false);
        });

        let entries =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "cmdk", |cx| {
                command_catalog_entries_from_host_commands(cx)
            });
        let item = find_item(&entries, &cmd).expect("catalog item");
        assert!(
            item.disabled,
            "expected the command entry to be disabled via WindowCommandEnabledService"
        );
    }

    #[test]
    fn host_command_entries_respect_widget_action_availability_snapshot() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let cmd = CommandId::from("test.widget-action");
        app.commands_mut().register(
            cmd.clone(),
            CommandMeta::new("Widget Action").with_scope(CommandScope::Widget),
        );

        app.set_global(WindowCommandActionAvailabilityService::default());
        app.with_global_mut(
            WindowCommandActionAvailabilityService::default,
            |svc, _app| {
                let mut snapshot: HashMap<CommandId, bool> = HashMap::new();
                snapshot.insert(cmd.clone(), false);
                svc.set_snapshot(window, snapshot);
            },
        );

        let entries =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "cmdk", |cx| {
                command_catalog_entries_from_host_commands(cx)
            });
        let item = find_item(&entries, &cmd).expect("catalog item");
        assert!(
            item.disabled,
            "expected the command entry to be disabled via WindowCommandActionAvailabilityService"
        );
    }

    #[test]
    fn host_command_entries_prefer_window_command_gating_snapshot_when_present() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let cmd = CommandId::from("test.widget-action");
        app.commands_mut().register(
            cmd.clone(),
            CommandMeta::new("Widget Action").with_scope(CommandScope::Widget),
        );

        app.set_global(WindowCommandActionAvailabilityService::default());
        app.with_global_mut(
            WindowCommandActionAvailabilityService::default,
            |svc, _app| {
                let mut snapshot: HashMap<CommandId, bool> = HashMap::new();
                snapshot.insert(cmd.clone(), true);
                svc.set_snapshot(window, snapshot);
            },
        );

        app.set_global(fret_runtime::WindowCommandGatingService::default());
        app.with_global_mut(
            fret_runtime::WindowCommandGatingService::default,
            |svc, app| {
                let input_ctx = command_palette_input_context(app);
                let enabled_overrides: HashMap<CommandId, bool> = HashMap::new();
                let mut availability: HashMap<CommandId, bool> = HashMap::new();
                availability.insert(cmd.clone(), false);
                svc.set_snapshot(
                    window,
                    WindowCommandGatingSnapshot::new(input_ctx, enabled_overrides)
                        .with_action_availability(Some(Arc::new(availability))),
                );
            },
        );

        let entries =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "cmdk", |cx| {
                command_catalog_entries_from_host_commands(cx)
            });
        let item = find_item(&entries, &cmd).expect("catalog item");
        assert!(
            item.disabled,
            "expected the command entry to be disabled via WindowCommandGatingService snapshot"
        );
    }
}
