use fret_app::App;
use fret_core::SemanticsRole;
use fret_ui::element::AnyElement;
use fret_ui::element::SemanticsProps;
use fret_ui::{ElementContext, Invalidation};
use fret_workspace::WorkspaceStatusBar;
use std::sync::Arc;

use crate::ui;

pub(super) type InspectorStatus = (Arc<str>, Arc<str>, Arc<str>, Arc<str>);

pub(super) fn status_bar_view(
    cx: &mut ElementContext<'_, App>,
    models: &ui::UiGalleryModels,
    inspector_status: Option<&InspectorStatus>,
    layout_time_us: u128,
    paint_time_us: u128,
) -> AnyElement {
    cx.keyed("ui_gallery.status_bar", |cx| {
        let status_last_action = cx
            .get_model_cloned(&models.last_action, Invalidation::Layout)
            .unwrap_or_else(|| Arc::<str>::from("<none>"));
        let status_theme = cx
            .get_model_cloned(&models.theme_preset, Invalidation::Layout)
            .flatten()
            .unwrap_or_else(|| Arc::<str>::from("<default>"));
        let status_view_cache = cx
            .get_model_copied(&models.view_cache_enabled, Invalidation::Layout)
            .unwrap_or(false);
        let status_cache_shell = cx
            .get_model_copied(&models.view_cache_cache_shell, Invalidation::Layout)
            .unwrap_or(false);

        let mut right_items: Vec<AnyElement> = vec![cx.text(format!(
            "theme: {} view_cache={} shell_cache={} layout_us={} paint_us={}",
            status_theme.as_ref(),
            status_view_cache as u8,
            status_cache_shell as u8,
            layout_time_us,
            paint_time_us
        ))];
        if let Some((cursor, hit, focus, text)) = inspector_status {
            right_items.push(cx.text(format!("inspect: {}", cursor.as_ref())));
            right_items.push(cx.text(format!("inspect: {}", hit.as_ref())));
            right_items.push(cx.text(format!("inspect: {}", focus.as_ref())));
            right_items.push(cx.text(format!("inspect: {}", text.as_ref())));
        }

        let status_last_action_label =
            Arc::<str>::from(format!("last action: {}", status_last_action.as_ref()));
        let status_last_action_text = status_last_action_label.clone();
        let status_last_action_item = cx.semantics(
            SemanticsProps {
                role: SemanticsRole::Text,
                label: Some(status_last_action_label),
                test_id: Some(Arc::from("ui-gallery-status-last-action")),
                ..Default::default()
            },
            move |cx| vec![cx.text(status_last_action_text.as_ref())],
        );

        WorkspaceStatusBar::new()
            .left(vec![status_last_action_item])
            .right(right_items)
            .into_element(cx)
    })
}
