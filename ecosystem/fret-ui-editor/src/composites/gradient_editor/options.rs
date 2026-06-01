//! GradientEditor public options and binding records.

use std::sync::Arc;

use fret_core::{Color, Px};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, UiActionHost};
use fret_ui::element::{LayoutStyle, Length, SizeStyle};

pub type OnGradientStopAction =
    Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, fret_ui::ItemKey) + 'static>;

pub type OnGradientAction = Arc<dyn Fn(&mut dyn UiActionHost, ActionCx) + 'static>;

#[derive(Debug, Clone)]
pub struct GradientEditorOptions {
    pub layout: LayoutStyle,
    pub enabled: bool,
    pub preview_height: Px,
    pub show_angle: bool,
    pub enable_preview_drag: bool,
    pub a11y_label: Option<Arc<str>>,
    pub id_source: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    pub preview_test_id: Option<Arc<str>>,
    pub stops_test_id: Option<Arc<str>>,
    pub add_stop_test_id: Option<Arc<str>>,
}

impl Default for GradientEditorOptions {
    fn default() -> Self {
        Self {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            enabled: true,
            preview_height: Px(22.0),
            show_angle: true,
            enable_preview_drag: true,
            a11y_label: None,
            id_source: None,
            test_id: None,
            preview_test_id: None,
            stops_test_id: None,
            add_stop_test_id: None,
        }
    }
}

#[derive(Clone)]
pub struct GradientStopBinding {
    pub id: fret_ui::ItemKey,
    pub position: Model<f64>,
    pub color: Model<Color>,
    pub remove: Option<OnGradientStopAction>,
}
