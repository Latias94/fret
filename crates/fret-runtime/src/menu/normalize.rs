use super::{MenuBar, MenuItem};

impl MenuBar {
    /// Normalize the menu structure for display across menu surfaces.
    ///
    /// This is a best-effort "shape cleanup" pass intended to prevent drift between:
    /// - OS menubars (runner mappings),
    /// - in-window menubars (overlay renderers),
    /// - other menu-like surfaces that derive from `MenuBar`.
    ///
    /// Current normalization rules:
    /// - remove leading separators,
    /// - collapse duplicate separators,
    /// - remove trailing separators,
    /// - recursively drop empty submenus (after normalizing their children).
    ///
    /// Note: this does **not** apply enable/disable gating; that is handled by
    /// `WindowCommandGatingSnapshot` and surface-specific policies.
    pub fn normalize(&mut self) {
        for menu in &mut self.menus {
            normalize_menu_items(&mut menu.items);
        }
    }

    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }
}

fn normalize_menu_items(items: &mut Vec<MenuItem>) {
    let mut out: Vec<MenuItem> = Vec::with_capacity(items.len());
    let mut last_was_separator = false;

    for item in std::mem::take(items) {
        match item {
            MenuItem::Separator => {
                if out.is_empty() || last_was_separator {
                    continue;
                }
                out.push(MenuItem::Separator);
                last_was_separator = true;
            }
            MenuItem::Submenu {
                title,
                when,
                mut items,
            } => {
                normalize_menu_items(&mut items);
                if items.is_empty() {
                    continue;
                }
                out.push(MenuItem::Submenu { title, when, items });
                last_was_separator = false;
            }
            other => {
                out.push(other);
                last_was_separator = false;
            }
        }
    }

    while matches!(out.last(), Some(MenuItem::Separator)) {
        out.pop();
    }

    *items = out;
}
