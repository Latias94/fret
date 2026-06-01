//! Gradient editor spike (v1).
//!
//! Goal: validate that editor primitives (DragValue, ColorEdit, PropertyGrid) are sufficient to
//! build an editor-grade gradient stop editor without adding new runtime contracts.
//!
//! This is intentionally a policy/composition surface:
//! - callers own stop models and mutations (add/remove/reorder),
//! - the editor crate provides consistent layout, chrome, and a compact preview.

use std::panic::Location;
use std::sync::{Arc, Mutex};

use fret_core::{Axis, Color, Edges, Px};
use fret_runtime::Model;
use fret_ui::element::{AnyElement, CrossAlign, FlexProps, MainAlign, SpacingLength};
use fret_ui::{ElementContext, Invalidation, UiHost};

use crate::primitives::input_group::derived_test_id;

mod angle;
mod options;
mod preview;
mod stops;
mod stops_group;
#[cfg(test)]
mod tests;

use angle::gradient_angle_row;
pub use options::{
    GradientEditorOptions, GradientStopBinding, OnGradientAction, OnGradientStopAction,
};
use preview::{GradientPreviewState, PreviewStop, gradient_preview_canvas};
use stops_group::{GradientStopsGroupOptions, gradient_stops_group};

#[derive(Clone)]
pub struct GradientEditor {
    pub angle_degrees: Option<Model<f64>>,
    pub stops: Arc<[GradientStopBinding]>,
    pub on_add_stop: Option<OnGradientAction>,
    pub options: GradientEditorOptions,
}

impl GradientEditor {
    pub fn new(stops: Arc<[GradientStopBinding]>) -> Self {
        Self {
            angle_degrees: None,
            stops,
            on_add_stop: None,
            options: GradientEditorOptions::default(),
        }
    }

    pub fn angle_degrees(mut self, angle: Option<Model<f64>>) -> Self {
        self.angle_degrees = angle;
        self
    }

    pub fn on_add_stop(mut self, on_add: Option<OnGradientAction>) -> Self {
        self.on_add_stop = on_add;
        self
    }

    pub fn options(mut self, options: GradientEditorOptions) -> Self {
        self.options = options;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let loc = Location::caller();
        let callsite = (loc.file(), loc.line(), loc.column());
        let id_source = self.options.id_source.clone();

        if let Some(id_source) = id_source.as_deref() {
            cx.keyed(("fret-ui-editor.gradient_editor", id_source), |cx| {
                self.into_element_keyed(cx)
            })
        } else {
            cx.keyed(("fret-ui-editor.gradient_editor", callsite), |cx| {
                self.into_element_keyed(cx)
            })
        }
    }

    fn into_element_keyed<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let GradientEditor {
            angle_degrees,
            stops,
            on_add_stop,
            options,
        } = self;

        let state_id = cx.named("gradient_editor.preview_state", |cx| cx.root_id());
        let preview_state: Arc<Mutex<GradientPreviewState>> = cx.state_for(
            state_id,
            || Arc::new(Mutex::new(GradientPreviewState::default())),
            |s| s.clone(),
        );

        let angle = angle_degrees
            .as_ref()
            .and_then(|m| cx.get_model_copied(m, Invalidation::Paint))
            .unwrap_or(0.0);
        let preview_test_id = options
            .preview_test_id
            .clone()
            .or_else(|| derived_test_id(options.test_id.as_ref(), "preview"));
        let stops_test_id = options
            .stops_test_id
            .clone()
            .or_else(|| derived_test_id(options.test_id.as_ref(), "stops"));
        let add_stop_test_id = options
            .add_stop_test_id
            .clone()
            .or_else(|| derived_test_id(options.test_id.as_ref(), "add-stop"));
        let angle_test_id = derived_test_id(options.test_id.as_ref(), "angle");

        let mut preview_stops: Vec<PreviewStop> = Vec::new();
        let mut stop_rows: Vec<(f64, GradientStopBinding)> = Vec::new();
        for stop in stops.iter() {
            let pos = cx
                .get_model_copied(&stop.position, Invalidation::Paint)
                .unwrap_or(0.0);
            let color = cx
                .get_model_copied(&stop.color, Invalidation::Paint)
                .unwrap_or(Color::TRANSPARENT);
            preview_stops.push(PreviewStop {
                id: stop.id,
                position: (pos as f32).clamp(0.0, 1.0),
                color,
            });
            stop_rows.push((pos, stop.clone()));
        }
        preview_stops.sort_by(|a, b| a.position.total_cmp(&b.position));
        stop_rows.sort_by(|a, b| a.0.total_cmp(&b.0));

        let preview_h = Px(options.preview_height.0.max(1.0));
        let active_stop = preview_state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .active_stop;
        let mut preview = gradient_preview_canvas(
            cx,
            options.enabled && options.enable_preview_drag,
            angle,
            preview_stops.clone(),
            preview_h,
            active_stop,
            preview_state.clone(),
            stops
                .iter()
                .map(|s| (s.id, s.position.clone()))
                .collect::<Vec<_>>()
                .into(),
        );
        if let Some(test_id) = preview_test_id.as_ref() {
            preview = preview.test_id(test_id.clone());
        }

        let angle_row = (options.show_angle)
            .then_some(angle_degrees.clone())
            .flatten()
            .map(|m| gradient_angle_row(cx, m, angle_test_id.clone()));

        let stops_group = gradient_stops_group(
            cx,
            GradientStopsGroupOptions {
                enabled: options.enabled,
                stops_len: stops.len(),
                stops_test_id,
                add_stop_test_id,
                on_add_stop,
                stop_rows,
            },
        );

        let mut content = Vec::new();
        content.push(preview);
        if let Some(angle_row) = angle_row {
            content.push(angle_row);
        }
        content.push(stops_group);

        let mut root = cx.flex(
            FlexProps {
                layout: options.layout,
                direction: Axis::Vertical,
                gap: SpacingLength::Px(Px(8.0)),
                padding: Edges::all(Px(0.0)).into(),
                justify: MainAlign::Start,
                align: CrossAlign::Stretch,
                wrap: false,
            },
            move |_cx| content,
        );

        if let Some(test_id) = options.test_id.as_ref() {
            root = root.test_id(test_id.clone());
        }
        root
    }
}
