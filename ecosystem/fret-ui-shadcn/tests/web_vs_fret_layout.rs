pub(crate) use fret_app::App;
pub(crate) use fret_core::{
    AppWindowId, Edges, Event, FrameId, ImageId, Modifiers, MouseButtons, NodeId, Point,
    PointerEvent, PointerId, PointerType, Px, Rect, Scene, SceneOp, SemanticsRole,
    Size as CoreSize, TextOverflow, TextWrap, Transform2D,
};
pub(crate) use fret_icons::IconId;
pub(crate) use fret_runtime::Model;
pub(crate) use fret_ui::Theme;
pub(crate) use fret_ui::element::{
    AnyElement, ColumnProps, ContainerProps, CrossAlign, FlexProps, GridProps, LayoutStyle, Length,
    MainAlign, PressableProps, RovingFlexProps, RowProps, SizeStyle, TextProps,
};
pub(crate) use fret_ui::scroll::ScrollHandle;
pub(crate) use fret_ui::tree::UiTree;
pub(crate) use fret_ui_kit::declarative::icon as decl_icon;
pub(crate) use fret_ui_kit::declarative::style as decl_style;
pub(crate) use fret_ui_kit::declarative::text as decl_text;
pub(crate) use fret_ui_kit::primitives::radio_group as radio_group_prim;
pub(crate) use fret_ui_kit::{
    ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius, Space, ui,
};
pub(crate) use fret_ui_shadcn::button_group::ButtonGroupText;
pub(crate) use fret_ui_shadcn::empty::{
    EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyMediaVariant, EmptyTitle,
};
pub(crate) use fret_ui_shadcn::sidebar::SidebarMenuButtonSize;
pub(crate) use serde::Deserialize;
pub(crate) use std::cell::Cell;
pub(crate) use std::rc::Rc;
pub(crate) use std::sync::Arc;

mod css_color;
pub(crate) use css_color::{Rgba, color_to_rgba, parse_css_color};
mod chart_test_data;
pub(crate) use chart_test_data::{CHART_INTERACTIVE_DESKTOP, CHART_INTERACTIVE_MOBILE};

#[path = "web_vs_fret_layout/support.rs"]
mod support;
pub(crate) use support::*;

#[path = "web_vs_fret_layout/web.rs"]
mod web;
pub(crate) use web::*;

#[path = "web_vs_fret_layout/harness.rs"]
mod harness;
pub(crate) use harness::*;

#[path = "web_vs_fret_layout/insets.rs"]
mod insets;
pub(crate) use insets::*;

#[path = "web_vs_fret_layout/accordion.rs"]
mod accordion;
#[path = "web_vs_fret_layout/avatar.rs"]
mod avatar;
#[path = "web_vs_fret_layout/badge.rs"]
mod badge;
#[path = "web_vs_fret_layout/basic.rs"]
mod basic;
#[path = "web_vs_fret_layout/breadcrumb.rs"]
mod breadcrumb;
#[path = "web_vs_fret_layout/button.rs"]
mod button;
#[path = "web_vs_fret_layout/calendar.rs"]
mod calendar;
pub(crate) use calendar::{
    parse_calendar_cell_size_px, parse_calendar_day_aria_label, parse_calendar_title_label,
    parse_calendar_weekday_label,
};
#[path = "web_vs_fret_layout/card.rs"]
mod card;
#[path = "web_vs_fret_layout/carousel.rs"]
mod carousel;
#[path = "web_vs_fret_layout/chart.rs"]
mod chart;
#[path = "web_vs_fret_layout/collapsible.rs"]
mod collapsible;
#[path = "web_vs_fret_layout/dashboard.rs"]
mod dashboard;
#[path = "web_vs_fret_layout/empty.rs"]
mod empty;
#[path = "web_vs_fret_layout/item.rs"]
mod item;
#[path = "web_vs_fret_layout/kbd.rs"]
mod kbd;
#[path = "web_vs_fret_layout/chart_scaffold.rs"]
mod layout_chart_scaffold_fixtures;
#[path = "web_vs_fret_layout/field.rs"]
mod layout_field_fixtures;
#[path = "web_vs_fret_layout/form.rs"]
mod layout_form_fixtures;
#[path = "web_vs_fret_layout/input.rs"]
mod layout_input_fixtures;
#[path = "web_vs_fret_layout/scroll.rs"]
mod layout_scroll_fixtures;
#[path = "web_vs_fret_layout/typography.rs"]
mod layout_typography_fixtures;
#[path = "web_vs_fret_layout/native_select.rs"]
mod native_select;
#[path = "web_vs_fret_layout/pagination.rs"]
mod pagination;
#[path = "web_vs_fret_layout/progress.rs"]
mod progress;
#[path = "web_vs_fret_layout/radio_group.rs"]
mod radio_group;
#[path = "web_vs_fret_layout/resizable.rs"]
mod resizable;
#[path = "web_vs_fret_layout/select.rs"]
mod select;
#[path = "web_vs_fret_layout/separator.rs"]
mod separator;
#[path = "web_vs_fret_layout/shell.rs"]
mod shell;
#[path = "web_vs_fret_layout/sidebar.rs"]
mod sidebar;
#[path = "web_vs_fret_layout/skeleton.rs"]
mod skeleton;
#[path = "web_vs_fret_layout/sonner.rs"]
mod sonner;
#[path = "web_vs_fret_layout/spinner.rs"]
mod spinner;
#[path = "web_vs_fret_layout/switch.rs"]
mod switch;
#[path = "web_vs_fret_layout/table.rs"]
mod table;
#[path = "web_vs_fret_layout/tabs.rs"]
mod tabs;
#[path = "web_vs_fret_layout/textarea.rs"]
mod textarea;
#[path = "web_vs_fret_layout/triggers.rs"]
mod triggers;

#[derive(Debug, Clone, Deserialize)]
struct FixtureSuite<T> {
    schema_version: u32,
    cases: Vec<T>,
}
