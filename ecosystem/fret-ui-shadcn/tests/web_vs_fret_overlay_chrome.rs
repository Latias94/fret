#![cfg(feature = "web-goldens")]
// Heavy, web-golden-backed conformance. Enable via:
//   cargo nextest run -p fret-ui-shadcn --features web-goldens

use fret_app::App;
use fret_core::{
    AppWindowId, Color, Event, FrameId, KeyCode, Modifiers, MouseButton, MouseButtons, Paint,
    Point, PointerEvent, PointerType, Px, Rect, Scene, SceneOp, SemanticsRole, Size as CoreSize,
    Transform2D,
};
use fret_runtime::Model;
use fret_ui::ElementContext;
use fret_ui::element::AnyElement;
use fret_ui::elements::{GlobalElementId, bounds_for_element, with_element_cx};
use fret_ui::tree::UiTree;
use fret_ui_kit::OverlayController;
use serde::Deserialize;
use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

mod css_color;
use css_color::{color_to_rgba, parse_css_color};

#[path = "web_vs_fret_overlay_chrome/web.rs"]
mod web;
use web::*;

#[path = "web_vs_fret_overlay_chrome/support.rs"]
mod support;
use support::*;

#[path = "web_vs_fret_overlay_chrome/alert_dialog.rs"]
mod alert_dialog;
#[path = "web_vs_fret_overlay_chrome/button_group.rs"]
mod button_group;
#[path = "web_vs_fret_overlay_chrome/calendar.rs"]
mod calendar;
#[path = "web_vs_fret_overlay_chrome/combobox.rs"]
mod combobox;
#[path = "web_vs_fret_overlay_chrome/command_dialog.rs"]
mod command_dialog;
#[path = "web_vs_fret_overlay_chrome/context_menu.rs"]
mod context_menu;
#[path = "web_vs_fret_overlay_chrome/date_picker.rs"]
mod date_picker;
#[path = "web_vs_fret_overlay_chrome/dialog.rs"]
mod dialog;
#[path = "web_vs_fret_overlay_chrome/drawer.rs"]
mod drawer;
#[path = "web_vs_fret_overlay_chrome/dropdown_menu.rs"]
mod dropdown_menu;
#[path = "web_vs_fret_overlay_chrome/hover_card.rs"]
mod hover_card;
#[path = "web_vs_fret_overlay_chrome/menubar.rs"]
mod menubar;
#[path = "web_vs_fret_overlay_chrome/navigation_menu.rs"]
mod navigation_menu;
#[path = "web_vs_fret_overlay_chrome/popover.rs"]
mod popover;
#[path = "web_vs_fret_overlay_chrome/select.rs"]
mod select;
#[path = "web_vs_fret_overlay_chrome/sheet.rs"]
mod sheet;
#[path = "web_vs_fret_overlay_chrome/tooltip.rs"]
mod tooltip;

#[derive(Debug, Clone, Deserialize)]
struct FixtureSuite<T> {
    schema_version: u32,
    cases: Vec<T>,
}
