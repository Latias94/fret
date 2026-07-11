use fret_app::{App, Model};
use fret_core::SemanticsRole;
use fret_ui::element::AnyElement;
use fret_ui::element::SemanticsProps;
use fret_ui::{ElementContext, Invalidation};
use fret_ui_shadcn::facade as shadcn;
use fret_workspace::layout::WorkspaceWindowLayout;
use fret_workspace::{WorkspaceTabStrip, WorkspaceTopBar};
use std::sync::Arc;

use super::text_roles;
use crate::spec::{CMD_APP_SETTINGS, page_meta};

pub(super) fn tab_strip_view(
    cx: &mut ElementContext<'_, App>,
    disabled: bool,
    workspace_window_layout: &Model<WorkspaceWindowLayout>,
) -> AnyElement {
    cx.keyed("ui_gallery.tab_strip", |cx| {
        if disabled {
            return text_roles::chrome_readout_text(cx, "Tabs (disabled)");
        }

        let workspace_layout = cx
            .get_model_cloned(workspace_window_layout, Invalidation::Layout)
            .unwrap_or_else(|| {
                WorkspaceWindowLayout::new(
                    super::UI_GALLERY_WORKSPACE_WINDOW_LAYOUT_ID,
                    super::UI_GALLERY_WORKSPACE_PANE_ID,
                )
            });
        let tab_strip = workspace_layout
            .pane_tree
            .find_pane(super::UI_GALLERY_WORKSPACE_PANE_ID)
            .map(|pane| {
                WorkspaceTabStrip::from_workspace_tabs(&pane.tabs, |tab_id| {
                    Arc::<str>::from(page_meta(tab_id).0)
                })
            })
            .unwrap_or_else(|| WorkspaceTabStrip::new_optional(None))
            .pane_id(super::UI_GALLERY_WORKSPACE_PANE_ID)
            .test_id_root("ui-gallery-workspace-tabstrip")
            .tab_test_id_prefix("ui-gallery-workspace-tab")
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
