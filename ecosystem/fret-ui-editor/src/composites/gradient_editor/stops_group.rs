//! GradientEditor stops group owner.

use std::sync::Arc;

use fret_core::Px;
use fret_core::scene::MAX_STOPS;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Theme, UiHost};

use super::stops::gradient_stop_row;
use super::{GradientStopBinding, OnGradientAction};
use crate::composites::{PropertyGrid, PropertyGroup};
use crate::controls::{IconButton, IconButtonOptions, OnIconButtonActivate};
use crate::primitives::EditorDensity;
use crate::primitives::readout::editor_empty_state_text_props;

pub(super) struct GradientStopsGroupOptions {
    pub(super) enabled: bool,
    pub(super) stops_len: usize,
    pub(super) stops_test_id: Option<Arc<str>>,
    pub(super) add_stop_test_id: Option<Arc<str>>,
    pub(super) on_add_stop: Option<OnGradientAction>,
    pub(super) stop_rows: Vec<(f64, GradientStopBinding)>,
}

pub(super) fn gradient_stops_group<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    options: GradientStopsGroupOptions,
) -> AnyElement {
    let GradientStopsGroupOptions {
        enabled,
        stops_len,
        stops_test_id,
        add_stop_test_id,
        on_add_stop,
        stop_rows,
    } = options;

    let density = EditorDensity::resolve(Theme::global(&*cx.app));
    let can_add_stop = enabled && (stops_len < MAX_STOPS);
    let stops_test_id_for_rows = stops_test_id.clone();

    let mut stops_group = PropertyGroup::new("Stops").into_element(
        cx,
        move |cx| {
            let on_add_stop = on_add_stop.clone()?;
            let on_activate: OnIconButtonActivate = Arc::new(move |host, action_cx| {
                on_add_stop(host, action_cx);
            });
            Some(
                IconButton::new(fret_icons::ids::ui::PLUS, on_activate)
                    .options(IconButtonOptions {
                        enabled: can_add_stop,
                        focusable: false,
                        icon_size: Some(Px(12.0)),
                        a11y_label: Some(Arc::from("Add stop")),
                        test_id: add_stop_test_id.clone(),
                        ..Default::default()
                    })
                    .into_element(cx),
            )
        },
        move |cx| {
            let stops_test_id = stops_test_id_for_rows.clone();
            vec![PropertyGrid::new().into_element(cx, move |cx, row_cx| {
                let mut rows = Vec::new();
                for (_pos, stop) in stop_rows.iter().cloned() {
                    let stops_test_id = stops_test_id.clone();
                    rows.push(cx.keyed(("gradient_stop_row", stop.id), |cx| {
                        gradient_stop_row(
                            cx,
                            density,
                            enabled,
                            stops_test_id.clone(),
                            stop,
                            row_cx.row_options(),
                        )
                    }));
                }
                if rows.is_empty() {
                    rows.push(gradient_editor_empty_state_text(cx, "No stops"));
                }
                rows
            })]
        },
    );

    if let Some(test_id) = stops_test_id.as_ref() {
        stops_group = stops_group.test_id(test_id.clone());
    }

    stops_group
}

pub(super) fn gradient_editor_empty_state_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    let (color, row_height) = {
        let theme = Theme::global(&*cx.app);
        let density = EditorDensity::resolve(theme);
        (
            crate::primitives::colors::editor_muted_foreground(theme),
            density.row_height,
        )
    };

    cx.text_props(editor_empty_state_text_props(
        text.into(),
        color,
        row_height,
    ))
}
