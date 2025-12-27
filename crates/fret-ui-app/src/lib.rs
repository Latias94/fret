//! Integration convenience layer: `fret-ui` bound to `fret-app::App`.
//!
//! This crate exists to keep app/demo/editor code ergonomic (`impl Widget for ...`)
//! while allowing `fret-ui` to remain host-generic and independent from `fret-app`.

pub use fret_app::App;

pub use fret_ui::declarative;
pub use fret_ui::dock;
pub use fret_ui::element;
pub use fret_ui::elements;
pub use fret_ui::primitives;
pub use fret_ui::theme;
pub use fret_ui::tree;

pub use fret_ui::{
    DockManager, DockPanel, DockPanelContentService, DockSpace, GlobalElementId, Invalidation,
    PaintCachePolicy, Theme, ThemeConfig, ThemeSnapshot, UiDebugFrameStats, UiDebugHitTest,
    UiDebugLayerInfo, UiHost, UiLayerId, ViewportPanel,
};

pub use fret_ui::primitives::{
    BoundTextArea, BoundTextInput, Clip, Column, Image, Path, ResizableSplit, Row, Scroll, Split,
    Stack, Text, TextArea, TextAreaStyle, TextInput, TextInputStyle,
};

pub use fret_ui::{
    CommandCx as GenericCommandCx, EventCx as GenericEventCx, LayoutCx as GenericLayoutCx,
    PaintCx as GenericPaintCx, UiTree as GenericUiTree, Widget as GenericWidget,
};

pub type UiTree = fret_ui::UiTree<App>;
pub type EventCx<'a> = fret_ui::EventCx<'a, App>;
pub type CommandCx<'a> = fret_ui::CommandCx<'a, App>;
pub type LayoutCx<'a> = fret_ui::LayoutCx<'a, App>;
pub type PaintCx<'a> = fret_ui::PaintCx<'a, App>;

pub mod accessibility_actions;
