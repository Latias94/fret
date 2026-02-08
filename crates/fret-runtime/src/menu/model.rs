use std::sync::Arc;

use crate::{CommandId, WhenExpr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MenuRole {
    App,
    File,
    Edit,
    View,
    Window,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SystemMenuType {
    Services,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MenuItemToggleKind {
    Checkbox,
    Radio,
}

/// Optional checked/radio semantics for command-backed menu items.
///
/// This is intentionally data-only so OS menubars and in-window surfaces can share one
/// source-of-truth for toggle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MenuItemToggle {
    pub kind: MenuItemToggleKind,
    pub checked: bool,
}

/// A minimal, data-only menu model intended to power:
/// - future menubar rendering,
/// - context menus,
/// - command palette “breadcrumbs”.
///
/// This keeps menu structures derived from commands (ADR 0023) and avoids duplicating enablement
/// logic in widget code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuBar {
    pub menus: Vec<Menu>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Menu {
    pub title: Arc<str>,
    pub role: Option<MenuRole>,
    /// Optional mnemonic/access key for Alt+Key activation on platforms that support it.
    ///
    /// This is intentionally separate from `title` so titles can remain localization-friendly
    /// and so OS vs in-window surfaces can share one source of truth.
    pub mnemonic: Option<char>,
    pub items: Vec<MenuItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuItem {
    Command {
        command: CommandId,
        when: Option<WhenExpr>,
        toggle: Option<MenuItemToggle>,
    },
    /// A non-interactive, disabled menu entry with a custom label.
    ///
    /// This is intended for placeholder and dynamic menu content where a `CommandId` is not
    /// available (yet). Menu surfaces should render this as disabled text.
    Label {
        title: Arc<str>,
    },
    Separator,
    Submenu {
        title: Arc<str>,
        when: Option<WhenExpr>,
        items: Vec<MenuItem>,
    },
    /// A menu managed by the OS (e.g. the macOS Services menu).
    ///
    /// In-window menu surfaces should ignore or render a disabled placeholder for these entries.
    SystemMenu {
        title: Arc<str>,
        menu_type: SystemMenuType,
    },
}

impl MenuBar {
    pub fn empty() -> Self {
        Self { menus: Vec::new() }
    }
}
