use fret_app::{App, CommandId, Model};
use fret_core::SemanticsRole;
use fret_ui::element::AnyElement;
use fret_ui::element::SemanticsProps;
use fret_ui::{ElementContext, Invalidation};
use fret_ui_shadcn::facade as shadcn;
use fret_workspace::commands::CMD_WORKSPACE_TAB_CLOSE_PREFIX;
use fret_workspace::{WorkspaceTab, WorkspaceTabStrip, WorkspaceTopBar};
use std::sync::Arc;

use crate::spec::{CMD_APP_SETTINGS, PAGE_INTRO, page_meta, page_spec};

pub(super) fn tab_strip_view(
    cx: &mut ElementContext<'_, App>,
    disabled: bool,
    selected_page: &Model<Arc<str>>,
    workspace_tabs: &Model<Vec<Arc<str>>>,
    workspace_dirty_tabs: &Model<Vec<Arc<str>>>,
) -> AnyElement {
    cx.keyed("ui_gallery.tab_strip", |cx| {
        if disabled {
            return cx.text("Tabs (disabled)");
        }

        let selected = cx
            .get_model_cloned(selected_page, Invalidation::Layout)
            .unwrap_or_else(|| Arc::<str>::from(PAGE_INTRO));
        let workspace_tab_ids = cx
            .get_model_cloned(workspace_tabs, Invalidation::Layout)
            .unwrap_or_default();
        let workspace_dirty_ids = cx
            .get_model_cloned(workspace_dirty_tabs, Invalidation::Layout)
            .unwrap_or_default();

        let tab_strip = WorkspaceTabStrip::new(selected.clone())
            .pane_id(super::UI_GALLERY_WORKSPACE_PANE_ID)
            .test_id_root("ui-gallery-workspace-tabstrip")
            .tab_test_id_prefix("ui-gallery-workspace-tab")
            .tabs(workspace_tab_ids.iter().map(|tab_id| {
                let (title, _origin) = page_meta(tab_id.as_ref());
                let dirty = workspace_dirty_ids
                    .iter()
                    .any(|d| d.as_ref() == tab_id.as_ref());
                WorkspaceTab::new(
                    tab_id.clone(),
                    title,
                    page_spec(tab_id.as_ref())
                        .map(|spec| CommandId::from(spec.command))
                        .unwrap_or_else(|| {
                            CommandId::new(format!("ui_gallery.nav.select.{}", tab_id.as_ref()))
                        }),
                )
                .close_command(CommandId::new(format!(
                    "{}{}",
                    CMD_WORKSPACE_TAB_CLOSE_PREFIX,
                    tab_id.as_ref()
                )))
                .dirty(dirty)
            }))
            .into_element(cx);
        tab_strip
    })
}

pub(super) fn top_bar_view(
    cx: &mut ElementContext<'_, App>,
    left: Vec<AnyElement>,
    tab_strip: Option<AnyElement>,
) -> AnyElement {
    let top_bar = WorkspaceTopBar::new()
        .left(left)
        .center(tab_strip.into_iter())
        .right(vec![
            shadcn::Button::new("Settings")
                .test_id("ui-gallery-settings-open")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .action(CMD_APP_SETTINGS)
                .into_element(cx),
            shadcn::Button::new("Command palette")
                .test_id("ui-gallery-command-palette")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .action(fret_app::core_commands::COMMAND_PALETTE)
                .into_element(cx),
        ])
        .into_element(cx);

    cx.semantics(
        SemanticsProps {
            role: SemanticsRole::Toolbar,
            test_id: Some(Arc::from("ui-gallery-top-bar")),
            ..Default::default()
        },
        |_cx| [top_bar],
    )
}
