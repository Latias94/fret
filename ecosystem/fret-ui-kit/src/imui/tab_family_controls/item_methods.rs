use std::sync::Arc;

use fret_ui::UiHost;

use super::super::containers::build_imui_children_with_focus;
use super::super::label_identity::parse_label_identity;
use super::super::{ImUiFacade, TabItemOptions};
use super::ImUiTabBar;
use super::items::BuiltTabItem;

impl<'cx, 'a, H: UiHost> ImUiTabBar<'cx, 'a, H> {
    pub fn tab_item(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        self.tab_item_with_options(id, label, TabItemOptions::default(), f);
    }

    pub fn tab_item_with_options(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        options: TabItemOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let id = Arc::<str>::from(id);
        let raw_label = label.into();
        let parts = parse_label_identity(raw_label.as_ref());
        let label = Arc::<str>::from(parts.visible);
        let test_id = options.test_id.clone();
        let panel_test_id = options.panel_test_id.or_else(|| {
            test_id
                .as_ref()
                .map(|test_id| Arc::from(format!("{test_id}.panel")))
        });
        let build_focus = self.build_focus.clone();
        let panel_children = self.cx.keyed(id.clone(), |cx| {
            let mut out = Vec::new();
            build_imui_children_with_focus(cx, &mut out, build_focus, f);
            out
        });
        self.items.push(BuiltTabItem {
            id,
            label,
            enabled: options.enabled,
            default_selected: options.default_selected,
            test_id,
            panel_test_id,
            activate_shortcut: options.activate_shortcut,
            shortcut_repeat: options.shortcut_repeat,
            panel_children,
        });
    }

    pub fn begin_tab_item(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        self.begin_tab_item_with_options(id, label, TabItemOptions::default(), f);
    }

    pub fn begin_tab_item_with_options(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        options: TabItemOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        self.tab_item_with_options(id, label, options, f);
    }
}
