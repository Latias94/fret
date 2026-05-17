use fret_app::App;
use fret_core::SemanticsRole;
use fret_ui::element::AnyElement;
use fret_ui::element::SemanticsProps;
use fret_ui::{ElementContext, Invalidation};
use fret_ui_kit::declarative::text as decl_text;
use fret_workspace::WorkspaceStatusBar;
use std::sync::Arc;

use crate::ui;

pub(super) type InspectorStatus = (Arc<str>, Arc<str>, Arc<str>, Arc<str>);

fn status_bar_readout_text(cx: &mut ElementContext<'_, App>, text: impl Into<Arc<str>>) -> AnyElement {
    decl_text::text_control_readout(cx, text)
}

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

        let mut right_items: Vec<AnyElement> = Vec::new();
        right_items.push(status_bar_readout_text(cx, format!(
            "theme={} view_cache={} layout_us={} paint_us={}",
            status_theme.as_ref(),
            status_view_cache as u8,
            layout_time_us,
            paint_time_us
        )));
        if inspector_status.is_some() {
            right_items.push(status_bar_readout_text(cx, "inspector=on"));
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
            move |cx| vec![status_bar_readout_text(cx, status_last_action_text.clone())],
        );

        let status_bar = WorkspaceStatusBar::new()
            .left(vec![status_last_action_item])
            .right(right_items)
            .into_element(cx);

        cx.semantics(
            SemanticsProps {
                role: SemanticsRole::Group,
                test_id: Some(Arc::from("ui-gallery-status-bar")),
                ..Default::default()
            },
            |_cx| [status_bar],
        )
    })
}
