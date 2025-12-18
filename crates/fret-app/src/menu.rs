use std::sync::Arc;

use crate::{CommandId, when_expr::WhenExpr};

/// A minimal, data-only menu model intended to power:
/// - future menubar rendering,
/// - context menus,
/// - command palette “breadcrumbs”.
///
/// This keeps menu structures derived from commands (ADR 0023) and avoids duplicating enablement
/// logic in widget code.
#[derive(Debug, Clone)]
pub struct MenuBar {
    pub menus: Vec<Menu>,
}

#[derive(Debug, Clone)]
pub struct Menu {
    pub title: Arc<str>,
    pub items: Vec<MenuItem>,
}

#[derive(Debug, Clone)]
pub enum MenuItem {
    Command {
        command: CommandId,
        when: Option<WhenExpr>,
    },
    Separator,
    Submenu {
        title: Arc<str>,
        when: Option<WhenExpr>,
        items: Vec<MenuItem>,
    },
}

