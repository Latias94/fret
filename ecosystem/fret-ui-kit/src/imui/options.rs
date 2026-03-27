//! Immediate-mode option structs and canonical defaults.

use std::sync::Arc;

use fret_core::{Px, SemanticsRole, Size};

use crate::primitives::popper;

#[derive(Debug, Clone, Copy)]
pub struct PopupMenuOptions {
    pub placement: popper::PopperContentPlacement,
    pub estimated_size: Size,
    pub modal: bool,
}

impl Default for PopupMenuOptions {
    fn default() -> Self {
        Self {
            placement: popper::PopperContentPlacement::new(
                popper::LayoutDirection::Ltr,
                popper::Side::Bottom,
                popper::Align::Start,
                Px(4.0),
            ),
            estimated_size: Size::new(Px(160.0), Px(120.0)),
            modal: true,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PopupModalOptions {
    pub size: Size,
    pub close_on_outside_press: bool,
}

impl Default for PopupModalOptions {
    fn default() -> Self {
        Self {
            size: Size::new(Px(320.0), Px(200.0)),
            close_on_outside_press: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MenuItemOptions {
    pub enabled: bool,
    pub close_popup: Option<fret_runtime::Model<bool>>,
    pub test_id: Option<Arc<str>>,
}

impl Default for MenuItemOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            close_popup: None,
            test_id: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ButtonOptions {
    pub enabled: bool,
    pub focusable: bool,
    pub a11y_label: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
}

impl Default for ButtonOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            focusable: true,
            a11y_label: None,
            test_id: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct InputTextOptions {
    pub enabled: bool,
    pub focusable: bool,
    pub a11y_label: Option<Arc<str>>,
    pub a11y_role: Option<SemanticsRole>,
    pub placeholder: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    pub submit_command: Option<fret_runtime::CommandId>,
    pub cancel_command: Option<fret_runtime::CommandId>,
}

impl Default for InputTextOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            focusable: true,
            a11y_label: None,
            a11y_role: Some(SemanticsRole::TextField),
            placeholder: None,
            test_id: None,
            submit_command: None,
            cancel_command: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextAreaOptions {
    pub enabled: bool,
    pub focusable: bool,
    pub a11y_label: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    pub min_height: Px,
    /// If true, opt into a stable multiline line-box policy suitable for UI/form text areas.
    ///
    /// This is expected to reduce baseline jitter across mixed-script / emoji lines, at the cost
    /// of potentially clipping glyph ink that exceeds the chosen line box.
    pub stable_line_boxes: bool,
}

impl Default for TextAreaOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            focusable: true,
            a11y_label: None,
            test_id: None,
            min_height: Px(80.0),
            stable_line_boxes: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SwitchOptions {
    pub enabled: bool,
    pub focusable: bool,
    pub a11y_label: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
}

impl Default for SwitchOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            focusable: true,
            a11y_label: None,
            test_id: None,
        }
    }
}

pub type ToggleOptions = SwitchOptions;

#[derive(Debug, Clone)]
pub struct SliderOptions {
    pub enabled: bool,
    pub focusable: bool,
    pub a11y_label: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    pub min: f32,
    pub max: f32,
    pub step: f32,
}

impl Default for SliderOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            focusable: true,
            a11y_label: None,
            test_id: None,
            min: 0.0,
            max: 100.0,
            step: 1.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SelectOptions {
    pub enabled: bool,
    pub focusable: bool,
    pub a11y_label: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    /// Optional stable popup scope id override.
    ///
    /// When set, `select_model_with_options` will use this id for its internal popup scope instead of
    /// deriving one from `test_id`/`label`. This is useful to avoid accidental collisions (e.g.
    /// multiple selects with the same label) and to keep popup store growth bounded when call sites
    /// generate dynamic labels.
    pub popup_scope_id: Option<Arc<str>>,
    pub placeholder: Option<Arc<str>>,
    pub popup: PopupMenuOptions,
}

impl Default for SelectOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            focusable: true,
            a11y_label: None,
            test_id: None,
            popup_scope_id: None,
            placeholder: Some(Arc::from("Select?")),
            popup: PopupMenuOptions::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HorizontalOptions {
    pub gap: crate::MetricRef,
    pub justify: crate::Justify,
    pub items: crate::Items,
    pub wrap: bool,
}

impl Default for HorizontalOptions {
    fn default() -> Self {
        Self {
            gap: crate::MetricRef::space(crate::Space::N0),
            justify: crate::Justify::Start,
            items: crate::Items::Center,
            wrap: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VerticalOptions {
    pub gap: crate::MetricRef,
    pub justify: crate::Justify,
    pub items: crate::Items,
    pub wrap: bool,
}

impl Default for VerticalOptions {
    fn default() -> Self {
        Self {
            gap: crate::MetricRef::space(crate::Space::N0),
            justify: crate::Justify::Start,
            items: crate::Items::Stretch,
            wrap: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GridOptions {
    pub columns: usize,
    pub column_gap: crate::MetricRef,
    pub row_gap: crate::MetricRef,
    pub row_justify: crate::Justify,
    pub row_items: crate::Items,
}

impl Default for GridOptions {
    fn default() -> Self {
        Self {
            columns: 1,
            column_gap: crate::MetricRef::space(crate::Space::N0),
            row_gap: crate::MetricRef::space(crate::Space::N0),
            row_justify: crate::Justify::Start,
            row_items: crate::Items::Center,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScrollOptions {
    pub axis: fret_ui::element::ScrollAxis,
    pub show_scrollbar_x: bool,
    pub show_scrollbar_y: bool,
    pub handle: Option<fret_ui::scroll::ScrollHandle>,
}

impl Default for ScrollOptions {
    fn default() -> Self {
        Self {
            axis: fret_ui::element::ScrollAxis::Y,
            show_scrollbar_x: false,
            show_scrollbar_y: true,
            handle: None,
        }
    }
}
