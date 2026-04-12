//! Immediate-mode option structs and canonical defaults.

use std::sync::Arc;

use fret_core::{Px, SemanticsRole, Size};

use crate::primitives::popper;

#[derive(Debug, Clone, Copy)]
pub struct PopupMenuOptions {
    pub placement: popper::PopperContentPlacement,
    pub estimated_size: Size,
    pub modal: bool,
    pub auto_focus: bool,
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
            auto_focus: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MenuBarOptions {
    pub gap: crate::MetricRef,
    pub test_id: Option<Arc<str>>,
}

impl Default for MenuBarOptions {
    fn default() -> Self {
        Self {
            gap: crate::MetricRef::space(crate::Space::N1),
            test_id: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BeginMenuOptions {
    pub enabled: bool,
    pub test_id: Option<Arc<str>>,
    pub popup: PopupMenuOptions,
}

impl Default for BeginMenuOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            test_id: None,
            popup: PopupMenuOptions::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BeginSubmenuOptions {
    pub enabled: bool,
    pub test_id: Option<Arc<str>>,
    pub popup: PopupMenuOptions,
}

impl Default for BeginSubmenuOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            test_id: None,
            popup: PopupMenuOptions {
                placement: popper::PopperContentPlacement::new(
                    popper::LayoutDirection::Ltr,
                    popper::Side::Right,
                    popper::Align::Start,
                    Px(4.0),
                ),
                estimated_size: Size::new(Px(160.0), Px(120.0)),
                modal: false,
                auto_focus: false,
            },
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
pub struct TooltipOptions {
    pub placement: popper::PopperContentPlacement,
    pub estimated_size: Size,
    pub window_margin: Px,
    pub open_delay_frames_override: Option<u32>,
    pub close_delay_frames_override: Option<u32>,
    pub disable_hoverable_content: Option<bool>,
    pub test_id: Option<Arc<str>>,
}

impl Default for TooltipOptions {
    fn default() -> Self {
        Self {
            placement: popper::PopperContentPlacement::new(
                popper::LayoutDirection::Ltr,
                popper::Side::Top,
                popper::Align::Center,
                Px(6.0),
            )
            .with_shift_cross_axis(true),
            estimated_size: Size::new(Px(180.0), Px(32.0)),
            window_margin: Px(8.0),
            open_delay_frames_override: None,
            close_delay_frames_override: None,
            disable_hoverable_content: None,
            test_id: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DragSourceOptions {
    /// When false, the helper does not publish a payload for the trigger's drag gesture.
    pub enabled: bool,
    /// When true, upgrade the trigger's runtime drag session to cross-window hover routing.
    pub cross_window: bool,
}

impl Default for DragSourceOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            cross_window: false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DropTargetOptions {
    /// When false, the target ignores active drags and never reports preview/delivery.
    pub enabled: bool,
}

impl Default for DropTargetOptions {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Debug, Clone)]
pub struct CollapsingHeaderOptions {
    pub enabled: bool,
    pub open: Option<fret_runtime::Model<bool>>,
    pub default_open: bool,
    pub test_id: Option<Arc<str>>,
    pub header_test_id: Option<Arc<str>>,
    pub content_test_id: Option<Arc<str>>,
}

impl Default for CollapsingHeaderOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            open: None,
            default_open: false,
            test_id: None,
            header_test_id: None,
            content_test_id: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TreeNodeOptions {
    pub enabled: bool,
    pub open: Option<fret_runtime::Model<bool>>,
    pub default_open: bool,
    pub selected: bool,
    pub leaf: bool,
    /// Optional hierarchy level for accessibility semantics (1-based).
    ///
    /// This also drives the default visual indentation for the first-cut immediate tree helper.
    pub level: u32,
    pub pos_in_set: Option<u32>,
    pub set_size: Option<u32>,
    pub test_id: Option<Arc<str>>,
    pub content_test_id: Option<Arc<str>>,
}

impl Default for TreeNodeOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            open: None,
            default_open: false,
            selected: false,
            leaf: false,
            level: 1,
            pos_in_set: None,
            set_size: None,
            test_id: None,
            content_test_id: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MenuItemOptions {
    pub enabled: bool,
    pub close_popup: Option<fret_runtime::Model<bool>>,
    pub shortcut: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    pub shortcut_test_id: Option<Arc<str>>,
    pub submenu: bool,
    pub expanded: Option<bool>,
}

impl Default for MenuItemOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            close_popup: None,
            shortcut: None,
            test_id: None,
            shortcut_test_id: None,
            submenu: false,
            expanded: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SelectableOptions {
    pub enabled: bool,
    pub focusable: bool,
    pub selected: bool,
    pub close_popup: Option<fret_runtime::Model<bool>>,
    pub a11y_label: Option<Arc<str>>,
    pub a11y_role: Option<SemanticsRole>,
    pub test_id: Option<Arc<str>>,
}

impl Default for SelectableOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            focusable: true,
            selected: false,
            close_popup: None,
            a11y_label: None,
            a11y_role: Some(SemanticsRole::ListBoxOption),
            test_id: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ComboOptions {
    pub enabled: bool,
    pub focusable: bool,
    pub a11y_label: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    pub popup: PopupMenuOptions,
}

impl Default for ComboOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            focusable: true,
            a11y_label: None,
            test_id: None,
            popup: PopupMenuOptions::default(),
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
pub struct ComboModelOptions {
    pub enabled: bool,
    pub focusable: bool,
    pub a11y_label: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    pub placeholder: Option<Arc<str>>,
    pub popup: PopupMenuOptions,
}

impl Default for ComboModelOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            focusable: true,
            a11y_label: None,
            test_id: None,
            placeholder: Some(Arc::from("Select...")),
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TableColumnWidth {
    Px(Px),
    Fill(f32),
}

impl TableColumnWidth {
    pub fn px(width: Px) -> Self {
        Self::Px(width)
    }

    pub fn fill(weight: f32) -> Self {
        Self::Fill(weight)
    }
}

#[derive(Debug, Clone)]
pub struct TableColumn {
    pub header: Option<Arc<str>>,
    pub width: TableColumnWidth,
}

impl TableColumn {
    pub fn px(header: impl Into<Arc<str>>, width: Px) -> Self {
        Self {
            header: Some(header.into()),
            width: TableColumnWidth::Px(width),
        }
    }

    pub fn fill(header: impl Into<Arc<str>>) -> Self {
        Self {
            header: Some(header.into()),
            width: TableColumnWidth::Fill(1.0),
        }
    }

    pub fn weighted(header: impl Into<Arc<str>>, weight: f32) -> Self {
        Self {
            header: Some(header.into()),
            width: TableColumnWidth::Fill(weight),
        }
    }

    pub fn unlabeled(width: TableColumnWidth) -> Self {
        Self {
            header: None,
            width,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TableOptions {
    pub show_header: bool,
    pub striped: bool,
    pub clip_cells: bool,
    pub column_gap: crate::MetricRef,
    pub row_gap: crate::MetricRef,
    pub test_id: Option<Arc<str>>,
}

impl Default for TableOptions {
    fn default() -> Self {
        Self {
            show_header: true,
            striped: false,
            clip_cells: true,
            column_gap: crate::MetricRef::space(crate::Space::N0),
            row_gap: crate::MetricRef::space(crate::Space::N0),
            test_id: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TableRowOptions {
    pub test_id: Option<Arc<str>>,
}

#[derive(Clone)]
pub struct VirtualListOptions {
    /// Bounded viewport height for the virtualized list surface.
    pub viewport_height: Px,
    /// Estimated row height used by the runtime virtualizer.
    pub estimate_row_height: Px,
    /// Overscan row count per side.
    pub overscan: usize,
    /// Caller-provided revision bump when item identities or row-height inputs change.
    pub items_revision: u64,
    /// Runtime measure mode.
    pub measure_mode: fret_ui::element::VirtualListMeasureMode,
    /// Runtime key-cache policy.
    pub key_cache: fret_ui::element::VirtualListKeyCacheMode,
    /// Number of off-window rows a retained host may keep alive.
    pub keep_alive: usize,
    /// Inter-row gap owned by the runtime virtualizer.
    pub gap: Px,
    /// Virtualizer scroll-margin offset.
    pub scroll_margin: Px,
    /// Optional known row-height callback used when `measure_mode == Known`.
    pub known_row_height_at: Option<Arc<dyn Fn(usize) -> Px + Send + Sync>>,
    /// Optional external scroll handle.
    pub handle: Option<fret_ui::scroll::VirtualListScrollHandle>,
    pub test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for VirtualListOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VirtualListOptions")
            .field("viewport_height", &self.viewport_height)
            .field("estimate_row_height", &self.estimate_row_height)
            .field("overscan", &self.overscan)
            .field("items_revision", &self.items_revision)
            .field("measure_mode", &self.measure_mode)
            .field("key_cache", &self.key_cache)
            .field("keep_alive", &self.keep_alive)
            .field("gap", &self.gap)
            .field("scroll_margin", &self.scroll_margin)
            .field("known_row_height_at", &self.known_row_height_at.is_some())
            .field("handle", &self.handle.is_some())
            .field("test_id", &self.test_id)
            .finish()
    }
}

impl Default for VirtualListOptions {
    fn default() -> Self {
        Self {
            viewport_height: Px(240.0),
            estimate_row_height: Px(28.0),
            overscan: 6,
            items_revision: 0,
            measure_mode: fret_ui::element::VirtualListMeasureMode::Measured,
            key_cache: fret_ui::element::VirtualListKeyCacheMode::AllKeys,
            keep_alive: 0,
            gap: Px(0.0),
            scroll_margin: Px(0.0),
            known_row_height_at: None,
            handle: None,
            test_id: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SeparatorTextOptions {
    pub test_id: Option<Arc<str>>,
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

#[derive(Debug, Clone, Default)]
pub struct ChildRegionOptions {
    pub scroll: ScrollOptions,
    pub test_id: Option<Arc<str>>,
    pub content_test_id: Option<Arc<str>>,
}
