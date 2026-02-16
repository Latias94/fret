use std::{collections::HashMap, sync::Arc};

use crate::{CommandId, DefaultKeybinding, WhenExpr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// A command that should be forwarded to the host OS when supported (e.g. menu integration).
pub enum OsAction {
    Cut,
    Copy,
    Paste,
    SelectAll,
    Undo,
    Redo,
}

#[derive(Default)]
/// Registry of known commands and their metadata.
///
/// Apps typically populate this during startup and use it to drive keybinding resolution,
/// command palettes, and menu surfaces.
pub struct CommandRegistry {
    commands: HashMap<CommandId, CommandMeta>,
}

#[derive(Debug, Clone)]
/// Metadata describing a command (title, keybindings, visibility, routing scope).
pub struct CommandMeta {
    pub title: Arc<str>,
    pub description: Option<Arc<str>>,
    pub category: Option<Arc<str>>,
    pub keywords: Vec<Arc<str>>,
    pub default_keybindings: Vec<DefaultKeybinding>,
    pub when: Option<WhenExpr>,
    pub os_action: Option<OsAction>,
    pub scope: CommandScope,
    pub hidden: bool,
    pub repeatable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Routing scope for a command.
pub enum CommandScope {
    /// Routed through the focused UI node (with bubbling), per ADR 0020.
    Widget,
    /// Handled at the window/app driver boundary (e.g. create/close windows, toggle overlays).
    Window,
    /// Global, cross-window command handled by the app.
    App,
}

impl CommandMeta {
    pub fn new(title: impl Into<Arc<str>>) -> Self {
        Self {
            title: title.into(),
            description: None,
            category: None,
            keywords: Vec::new(),
            default_keybindings: Vec::new(),
            when: None,
            os_action: None,
            scope: CommandScope::Window,
            hidden: false,
            repeatable: false,
        }
    }

    pub fn with_description(mut self, description: impl Into<Arc<str>>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_category(mut self, category: impl Into<Arc<str>>) -> Self {
        self.category = Some(category.into());
        self
    }

    pub fn with_keywords(
        mut self,
        keywords: impl IntoIterator<Item = impl Into<Arc<str>>>,
    ) -> Self {
        self.keywords = keywords.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_default_keybindings(
        mut self,
        bindings: impl IntoIterator<Item = DefaultKeybinding>,
    ) -> Self {
        self.default_keybindings = bindings.into_iter().collect();
        self
    }

    pub fn with_when(mut self, when: WhenExpr) -> Self {
        self.when = Some(when);
        self
    }

    pub fn with_os_action(mut self, os_action: OsAction) -> Self {
        self.os_action = Some(os_action);
        self
    }

    pub fn with_scope(mut self, scope: CommandScope) -> Self {
        self.scope = scope;
        self
    }

    pub fn hidden(mut self) -> Self {
        self.hidden = true;
        self
    }

    pub fn repeatable(mut self) -> Self {
        self.repeatable = true;
        self
    }
}

impl CommandRegistry {
    pub fn register(&mut self, id: CommandId, meta: CommandMeta) {
        self.commands.insert(id, meta);
    }

    pub fn get(&self, id: CommandId) -> Option<&CommandMeta> {
        self.commands.get(&id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&CommandId, &CommandMeta)> {
        self.commands.iter()
    }
}
