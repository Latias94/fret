use crate::popper_arrow::{self, DiamondArrowStyle};
use fret_core::{Color, Corners, Edges, FontId, FontWeight, Point, Px, SemanticsRole, TextStyle};
use fret_icons::ids;
use fret_runtime::Model;
use fret_ui::action::{ActionCx, OnDismissRequest};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, InsetStyle, LayoutStyle, Length, MainAlign,
    Overflow, PointerRegionProps, PositionStyle, PressableA11y, PressableProps, ScrollProps,
    SemanticsProps, SizeStyle, StackProps,
};
use fret_ui::elements::GlobalElementId;
use fret_ui::overlay_placement::{Align, Side};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt;
use fret_ui_kit::declarative::chrome as decl_chrome;
use fret_ui_kit::declarative::collection_semantics::CollectionSemanticsExt as _;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::overlay_motion;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::overlay;
use fret_ui_kit::primitives::active_descendant as active_desc;
use fret_ui_kit::primitives::direction as direction_prim;
use fret_ui_kit::primitives::popper;
use fret_ui_kit::primitives::popper_content;
use fret_ui_kit::primitives::presence as radix_presence;
use fret_ui_kit::primitives::roving_focus_group;
use fret_ui_kit::primitives::select as radix_select;
use fret_ui_kit::recipes::input::{
    InputTokenKeys, input_chrome_container_props, resolve_input_chrome,
};
use fret_ui_kit::theme_tokens;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, OverlayPresence, OverrideSlot, Space,
    WidgetState, WidgetStateProperty, WidgetStates, resolve_override_slot, ui,
};
use std::cell::Cell;
use std::sync::{Arc, Mutex};

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

fn select_list_desired_height(
    item_height: Px,
    item_count: usize,
    max_height: Px,
    outer_height: Px,
) -> Px {
    let outer_height = Px(outer_height.0.max(0.0));
    let max_height = Px(max_height.0.max(0.0).min(outer_height.0));

    // Radix/shadcn: keep the list at least one row tall when possible, but never exceed the
    // computed max height (derived from available space) since the content scrolls internally.
    let min_height = Px(item_height.0.max(0.0).min(max_height.0));
    let content_height = Px(item_height.0.max(0.0) * item_count as f32);

    Px(content_height.0.min(max_height.0).max(min_height.0))
}

fn select_scroll_with_buttons<H: UiHost, C, I>(
    cx: &mut ElementContext<'_, H>,
    theme: Theme,
    item_step: Px,
    scroll_handle: fret_ui::scroll::ScrollHandle,
    initial_scroll_to_y: Option<Px>,
    viewport_id_out: &Cell<Option<GlobalElementId>>,
    active_element_id_out: &Cell<Option<GlobalElementId>>,
    consume_pending_active_scroll_into_view: impl Fn() -> bool + Clone + 'static,
    should_align_active_to_top: impl Fn() -> bool + Clone + 'static,
    on_aligned_active_to_top: impl Fn() + Clone + 'static,
    set_scroll_up_visible: impl Fn(bool) + Clone + 'static,
    should_focus_selected_item: impl Fn() -> bool + Clone + 'static,
    on_focused_selected_item: impl Fn() + Clone + 'static,
    content: C,
) -> AnyElement
where
    C: FnOnce(&mut ElementContext<'_, H>, &Cell<Option<GlobalElementId>>) -> I,
    I: IntoIterator<Item = AnyElement>,
{
    cx.flex(
        FlexProps {
            layout: {
                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Fill;
                layout.size.height = Length::Fill;
                layout
            },
            direction: fret_core::Axis::Vertical,
            gap: Px(0.0),
            padding: Edges::all(Px(0.0)),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
            wrap: false,
        },
        move |cx| {
            let handle = scroll_handle.clone();
            let did_initial_scroll = initial_scroll_to_y.is_some();
            if let Some(y) = initial_scroll_to_y {
                let prev = handle.offset();
                handle.scroll_to_offset(Point::new(prev.x, y));
            }

            let scroll_button_h = theme
                .metric_by_key("component.select.scroll_button_height")
                .unwrap_or(Px(24.0));

            let max = handle.max_offset();
            let offset = handle.offset();
            // Guard against fractional max offsets (layout rounding) causing scroll affordances to
            // appear when content visually fits.
            let scroll_epsilon = Px(0.5);
            let has_scroll = max.y.0 > scroll_epsilon.0;
            let show_up = has_scroll && offset.y.0 > scroll_epsilon.0;
            // Match Radix Select's `Math.ceil(scrollTop) < maxScroll` guard for zoomed UIs.
            let show_down = has_scroll && offset.y.0.ceil() < max.y.0;

            if std::env::var("FRET_DEBUG_SELECT_SCROLLABLE")
                .ok()
                .is_some_and(|v| v == "1")
            {
                eprintln!(
                    "select scroll offset_y={} max_y={} show_up={} show_down={} did_initial_scroll={}",
                    offset.y.0, max.y.0, show_up, show_down, did_initial_scroll
                );
            }

            set_scroll_up_visible(show_up);

            let scroll_button = |cx: &mut ElementContext<'_, H>,
                                 icon: fret_icons::IconId,
                                 test_id: &'static str,
                                 dir: f32,
                                 visible: bool| {
                let handle = handle.clone();
                let theme = theme.clone();
                let pressable = cx.pressable(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Fill;
                            layout.size.height = Length::Px(scroll_button_h);
                            layout
                        },
                        enabled: true,
                        focusable: false,
                        a11y: PressableA11y {
                            hidden: true,
                            test_id: Some(Arc::from(test_id)),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    move |cx, _st| {
                        let on_scroll = Arc::new(move |host: &mut dyn fret_ui::action::UiActionHost,
                                                  action_cx: ActionCx| {
                            let prev = handle.offset();
                            let next = Point::new(prev.x, Px(prev.y.0 + item_step.0 * dir));
                            handle.scroll_to_offset(next);
                            host.request_redraw(action_cx.window);
                        });

                        let on_scroll_for_activate = on_scroll.clone();
                        cx.pressable_add_on_activate(Arc::new(move |host, action_cx, _reason| {
                            on_scroll_for_activate(host, action_cx);
                        }));

                        cx.pressable_add_on_pointer_down(Arc::new(move |host, action_cx, down| {
                            if down.button != fret_core::MouseButton::Left {
                                return fret_ui::action::PressablePointerDownResult::Continue;
                            }
                            on_scroll(host, action_cx);
                            host.prevent_default(fret_runtime::DefaultAction::FocusOnPointerDown);
                            fret_ui::action::PressablePointerDownResult::SkipDefaultAndStopPropagation
                        }));

                        vec![cx.container(
                            ContainerProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Fill;
                                    layout.size.height = Length::Fill;
                                    layout
                                },
                                // new-york-v4: `py-1` and no hover fill.
                                padding: Edges {
                                    top: Px(4.0),
                                    right: Px(0.0),
                                    bottom: Px(4.0),
                                    left: Px(0.0),
                                },
                                background: Some(Color::TRANSPARENT),
                                shadow: None,
                                border: Edges::all(Px(0.0)),
                                border_color: None,
                                corner_radii: Corners::all(Px(0.0)),
                                ..Default::default()
                            },
                            |cx| {
                                vec![cx.flex(
                                    FlexProps {
                                        layout: {
                                            let mut layout = LayoutStyle::default();
                                            layout.size.width = Length::Fill;
                                            layout.size.height = Length::Fill;
                                            layout
                                        },
                                        direction: fret_core::Axis::Horizontal,
                                        gap: Px(0.0),
                                        padding: Edges::all(Px(0.0)),
                                        justify: MainAlign::Center,
                                        align: CrossAlign::Center,
                                        wrap: false,
                                    },
                                    |cx| {
                                        vec![decl_icon::icon_with(
                                            cx,
                                            icon,
                                            Some(Px(16.0)),
                                            Some(ColorRef::Color(
                                                theme
                                                    .color_by_key("muted-foreground")
                                                    .unwrap_or_else(|| {
                                                        theme.color_required("muted-foreground")
                                                    }),
                                            )),
                                        )]
                                    },
                                )]
                            },
                        )]
                    },
                )
                ;

                let gated = cx.interactivity_gate(true, visible, |_cx| vec![pressable]);
                cx.opacity(if visible { 1.0 } else { 0.0 }, |_cx| vec![gated])
            };

            let handle_for_stack = handle.clone();
            let stack = cx.stack_props(
                StackProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout.size.min_height = Some(Px(0.0));
                        layout.flex.grow = 1.0;
                        layout.flex.shrink = 1.0;
                        layout.flex.basis = Length::Px(Px(0.0));
                        layout
                    },
                },
                move |cx| {
                    active_element_id_out.set(None);
                    let active_element_ref = active_element_id_out;

                    let mut scroll_layout = LayoutStyle::default();
                    scroll_layout.size.width = Length::Fill;
                    scroll_layout.size.height = Length::Fill;
                    scroll_layout.overflow = Overflow::Clip;

                    let scroll = cx.scroll(
                        ScrollProps {
                            layout: scroll_layout,
                            scroll_handle: Some(handle_for_stack.clone()),
                            ..Default::default()
                        },
                        move |cx| {
                            vec![cx.container(
                                ContainerProps {
                                    layout: {
                                        let mut layout = LayoutStyle::default();
                                        layout.size.width = Length::Fill;
                                        layout
                                    },
                                    padding: Edges::all(Px(0.0)),
                                    ..Default::default()
                                },
                                move |cx| {
                                    content(cx, active_element_ref)
                                        .into_iter()
                                        .collect::<Vec<_>>()
                                },
                            )]
                        },
                    );
                    viewport_id_out.set(Some(scroll.id));

                    let scroll = cx.semantics(
                        SemanticsProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Fill;
                                layout.size.height = Length::Fill;
                                layout
                            },
                            test_id: Some(Arc::from("select-scroll-viewport")),
                            ..Default::default()
                        },
                        |_cx| vec![scroll],
                    );

                    if let Some(active_element) = active_element_ref.get() {
                        if has_scroll && !did_initial_scroll && should_align_active_to_top() {
                            let did = active_desc::scroll_active_element_align_top_y(
                                cx,
                                &handle_for_stack,
                                scroll.id,
                                active_element,
                            );
                            if did {
                                on_aligned_active_to_top();
                            } else if let (Some(viewport), Some(child)) = (
                                cx.last_bounds_for_element(scroll.id),
                                cx.last_bounds_for_element(active_element),
                            ) {
                                let delta = (child.origin.y.0 - viewport.origin.y.0).abs();
                                if delta <= 0.01 {
                                    on_aligned_active_to_top();
                                }
                            }

                        } else if has_scroll && !did_initial_scroll && should_focus_selected_item() {
                            // Match Radix `focusSelectedItem`'s `scrollIntoView({ block: 'nearest' })`
                            // behavior using scroll-content coordinates (stable even when we don't
                            // have paint-space bounds for scrolled children).
                            if let (Some(viewport), Some(child)) = (
                                cx.last_bounds_for_element(scroll.id),
                                cx.last_bounds_for_element(active_element),
                            ) {
                                let child_top =
                                    Px((child.origin.y.0 - viewport.origin.y.0).max(0.0));
                                let child_h = Px(child.size.height.0.max(0.0));
                                let child_bottom = Px(child_top.0 + child_h.0);
                                let viewport_h = Px(viewport.size.height.0.max(0.0));

                                let prev = handle_for_stack.offset();
                                let view_top = prev.y;
                                let view_bottom = Px(prev.y.0 + viewport_h.0);

                                let target_y = if child_top.0 < view_top.0 {
                                    child_top
                                } else if child_bottom.0 > view_bottom.0 {
                                    Px(child_bottom.0 - viewport_h.0)
                                } else {
                                    view_top
                                };
                                handle_for_stack.set_offset(Point::new(prev.x, target_y));
                            }
                            on_focused_selected_item();
                        } else {
                            // Match Radix Select: only keep the active option in view when the
                            // active row changes via keyboard/typeahead. Do not continuously
                            // "chase" the active row during wheel scrolling.
                            if consume_pending_active_scroll_into_view() {
                                let _ = active_desc::scroll_active_element_into_view_y(
                                    cx,
                                    &handle_for_stack,
                                    scroll.id,
                                    active_element,
                                );
                            }
                        }
                    }

                    vec![scroll]
                },
            );

            if has_scroll {
                vec![
                    scroll_button(
                        cx,
                        ids::ui::CHEVRON_UP,
                        "select-scroll-up-button",
                        -1.0,
                        show_up,
                    ),
                    stack,
                    scroll_button(
                        cx,
                        ids::ui::CHEVRON_DOWN,
                        "select-scroll-down-button",
                        1.0,
                        show_down,
                    ),
                ]
            } else {
                vec![stack]
            }
        },
    )
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SelectAlign {
    #[default]
    Start,
    Center,
    End,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SelectSide {
    Top,
    Right,
    #[default]
    Bottom,
    Left,
}

impl From<SelectAlign> for Align {
    fn from(value: SelectAlign) -> Self {
        match value {
            SelectAlign::Start => Align::Start,
            SelectAlign::Center => Align::Center,
            SelectAlign::End => Align::End,
        }
    }
}

impl From<SelectSide> for Side {
    fn from(value: SelectSide) -> Self {
        match value {
            SelectSide::Top => Side::Top,
            SelectSide::Right => Side::Right,
            SelectSide::Bottom => Side::Bottom,
            SelectSide::Left => Side::Left,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SelectItem {
    pub value: Arc<str>,
    pub label: Arc<str>,
    pub test_id: Option<Arc<str>>,
    pub disabled: bool,
}

impl SelectItem {
    pub fn new(value: impl Into<Arc<str>>, label: impl Into<Arc<str>>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            test_id: None,
            disabled: false,
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// shadcn/ui `SelectLabel` (v4).
#[derive(Debug, Clone)]
pub struct SelectLabel {
    pub text: Arc<str>,
}

impl SelectLabel {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }
}

/// shadcn/ui `SelectGroup` (v4).
///
/// In the upstream DOM implementation this is a structural wrapper. In Fret we render it by
/// flattening its entries into the surrounding listbox.
#[derive(Debug, Clone)]
pub struct SelectGroup {
    pub entries: Vec<SelectEntry>,
}

impl SelectGroup {
    pub fn new(entries: impl IntoIterator<Item = SelectEntry>) -> Self {
        Self {
            entries: entries.into_iter().collect(),
        }
    }
}

/// shadcn/ui `SelectSeparator` (v4).
#[derive(Debug, Clone, Copy, Default)]
pub struct SelectSeparator;

#[derive(Debug, Clone)]
pub enum SelectEntry {
    Item(SelectItem),
    Label(SelectLabel),
    Group(SelectGroup),
    Separator(SelectSeparator),
}

impl From<SelectItem> for SelectEntry {
    fn from(value: SelectItem) -> Self {
        Self::Item(value)
    }
}

impl From<SelectLabel> for SelectEntry {
    fn from(value: SelectLabel) -> Self {
        Self::Label(value)
    }
}

impl From<SelectGroup> for SelectEntry {
    fn from(value: SelectGroup) -> Self {
        Self::Group(value)
    }
}

impl From<SelectSeparator> for SelectEntry {
    fn from(value: SelectSeparator) -> Self {
        Self::Separator(value)
    }
}

/// Matches Radix Select `position`: item-aligned (default upstream) vs popper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SelectPosition {
    #[default]
    ItemAligned,
    Popper,
}

#[derive(Debug, Clone, Copy, Default)]
struct BorderWidthOverride {
    top: Option<Px>,
    right: Option<Px>,
    bottom: Option<Px>,
    left: Option<Px>,
}

#[derive(Debug, Clone, Default)]
pub struct SelectStyle {
    pub trigger_border_color: OverrideSlot<ColorRef>,
    pub option_background: OverrideSlot<ColorRef>,
    pub option_foreground: OverrideSlot<ColorRef>,
}

impl SelectStyle {
    pub fn trigger_border_color(
        mut self,
        trigger_border_color: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.trigger_border_color = Some(trigger_border_color);
        self
    }

    pub fn option_background(
        mut self,
        option_background: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.option_background = Some(option_background);
        self
    }

    pub fn option_foreground(
        mut self,
        option_foreground: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.option_foreground = Some(option_foreground);
        self
    }

    pub fn merged(mut self, other: Self) -> Self {
        if other.trigger_border_color.is_some() {
            self.trigger_border_color = other.trigger_border_color;
        }
        if other.option_background.is_some() {
            self.option_background = other.option_background;
        }
        if other.option_foreground.is_some() {
            self.option_foreground = other.option_foreground;
        }
        self
    }
}

#[derive(Clone)]
pub struct Select {
    model: Model<Option<Arc<str>>>,
    open: Model<bool>,
    on_value_change:
        Option<Arc<dyn Fn(&mut dyn fret_ui::action::UiActionHost, ActionCx, Arc<str>) + 'static>>,
    entries: Vec<SelectEntry>,
    placeholder: Arc<str>,
    disabled: bool,
    trigger_test_id: Option<Arc<str>>,
    a11y_label: Option<Arc<str>>,
    aria_invalid: bool,
    on_dismiss_request: Option<OnDismissRequest>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    style: SelectStyle,
    align: SelectAlign,
    side: SelectSide,
    align_offset: Px,
    side_offset_override: Option<Px>,
    position: SelectPosition,
    loop_navigation: bool,
    arrow: bool,
    arrow_size_override: Option<Px>,
    arrow_padding_override: Option<Px>,
    trigger_border_width_override: BorderWidthOverride,
    trigger_corner_radii_override: Option<Corners>,
}

impl Select {
    pub fn new(model: Model<Option<Arc<str>>>, open: Model<bool>) -> Self {
        Self {
            model,
            open,
            on_value_change: None,
            entries: Vec::new(),
            placeholder: Arc::from("Select..."),
            disabled: false,
            trigger_test_id: None,
            a11y_label: None,
            aria_invalid: false,
            on_dismiss_request: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            style: SelectStyle::default(),
            align: SelectAlign::default(),
            side: SelectSide::default(),
            align_offset: Px(0.0),
            side_offset_override: None,
            position: SelectPosition::default(),
            loop_navigation: true,
            arrow: false,
            arrow_size_override: None,
            arrow_padding_override: None,
            trigger_border_width_override: BorderWidthOverride::default(),
            trigger_corner_radii_override: None,
        }
    }

    /// Creates a Select with controlled/uncontrolled `value` + `open` models (Radix `value` /
    /// `defaultValue` and `open` / `defaultOpen`).
    ///
    /// Notes:
    /// - When a controlled model is `None`, an internal model is created and stored in element state
    ///   at the call site.
    /// - Call this from a stable subtree (key the parent node if you need state to survive
    ///   reordering).
    pub fn new_controllable<H: UiHost, T: Into<Arc<str>>>(
        cx: &mut ElementContext<'_, H>,
        value: Option<Model<Option<Arc<str>>>>,
        default_value: Option<T>,
        open: Option<Model<bool>>,
        default_open: bool,
    ) -> Self {
        let default_value: Option<Arc<str>> = default_value.map(Into::into);
        let model =
            radix_select::select_use_value_model(cx, value, || default_value.clone()).model();

        let open = radix_select::SelectRoot::new()
            .open(open)
            .default_open(default_open)
            .open_model(cx);

        Self::new(model, open)
    }

    /// Called when the user selects a value (Radix `onValueChange`).
    ///
    /// Note: this only fires for user-driven selection events (click/keyboard selection on an
    /// item). Programmatic model updates do not trigger this callback.
    pub fn on_value_change(
        mut self,
        f: impl Fn(&mut dyn fret_ui::action::UiActionHost, ActionCx, Arc<str>) + 'static,
    ) -> Self {
        self.on_value_change = Some(Arc::new(f));
        self
    }

    pub fn item(mut self, item: SelectItem) -> Self {
        self.entries.push(SelectEntry::Item(item));
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = SelectItem>) -> Self {
        self.entries
            .extend(items.into_iter().map(SelectEntry::Item));
        self
    }

    pub fn entry(mut self, entry: impl Into<SelectEntry>) -> Self {
        self.entries.push(entry.into());
        self
    }

    /// Sets a `test_id` on the Select trigger pressable for deterministic automation.
    ///
    /// This is a diagnostics/testing hook and MUST NOT be mapped into platform accessibility label fields.
    pub fn trigger_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.trigger_test_id = Some(id.into());
        self
    }

    pub fn entries(mut self, entries: impl IntoIterator<Item = SelectEntry>) -> Self {
        self.entries.extend(entries);
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn aria_invalid(mut self, aria_invalid: bool) -> Self {
        self.aria_invalid = aria_invalid;
        self
    }

    /// Sets an optional dismiss request handler (Radix `DismissableLayer`).
    ///
    /// When set, Escape/outside-press dismissals route through this handler. To prevent default
    /// dismissal, call `req.prevent_default()`.
    pub fn on_dismiss_request(mut self, on_dismiss_request: Option<OnDismissRequest>) -> Self {
        self.on_dismiss_request = on_dismiss_request;
        self
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn style(mut self, style: SelectStyle) -> Self {
        self.style = self.style.merged(style);
        self
    }

    pub fn align(mut self, align: SelectAlign) -> Self {
        self.align = align;
        self
    }

    pub fn side(mut self, side: SelectSide) -> Self {
        self.side = side;
        self
    }

    pub fn align_offset(mut self, offset: Px) -> Self {
        self.align_offset = offset;
        self
    }

    pub fn side_offset(mut self, offset: Px) -> Self {
        self.side_offset_override = Some(offset);
        self
    }

    pub fn position(mut self, position: SelectPosition) -> Self {
        self.position = position;
        self
    }

    /// When `true` (default), roving navigation loops at the ends (Radix `loop` behavior).
    pub fn loop_navigation(mut self, loop_navigation: bool) -> Self {
        self.loop_navigation = loop_navigation;
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    /// Overrides per-edge border widths (in px) for the Select trigger's chrome.
    ///
    /// This is primarily used by shadcn recipe compositions that merge borders (e.g. input groups).
    pub fn border_left_width_override(mut self, border: Px) -> Self {
        self.trigger_border_width_override.left = Some(border);
        self
    }

    pub fn border_right_width_override(mut self, border: Px) -> Self {
        self.trigger_border_width_override.right = Some(border);
        self
    }

    pub fn border_top_width_override(mut self, border: Px) -> Self {
        self.trigger_border_width_override.top = Some(border);
        self
    }

    pub fn border_bottom_width_override(mut self, border: Px) -> Self {
        self.trigger_border_width_override.bottom = Some(border);
        self
    }

    /// Overrides per-corner radii (in px) for the Select trigger's chrome.
    ///
    /// This is primarily used by shadcn recipe compositions that merge corner radii
    /// (`rounded-l-none`, `rounded-r-none`).
    pub fn corner_radii_override(mut self, corners: Corners) -> Self {
        self.trigger_corner_radii_override = Some(corners);
        self
    }

    /// Enables a Select arrow (Radix `SelectArrow`-style).
    pub fn arrow(mut self, arrow: bool) -> Self {
        self.arrow = arrow;
        self
    }

    pub fn arrow_size(mut self, size: Px) -> Self {
        self.arrow_size_override = Some(size);
        self
    }

    pub fn arrow_padding(mut self, padding: Px) -> Self {
        self.arrow_padding_override = Some(padding);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        select_impl(
            cx,
            self.model,
            self.open,
            self.on_value_change,
            &self.entries,
            self.placeholder,
            self.disabled,
            self.trigger_test_id,
            self.a11y_label,
            self.aria_invalid,
            self.on_dismiss_request,
            self.chrome,
            self.layout,
            self.style,
            self.align,
            self.side,
            self.align_offset,
            self.side_offset_override,
            self.position,
            self.loop_navigation,
            self.arrow,
            self.arrow_size_override,
            self.arrow_padding_override,
            self.trigger_border_width_override,
            self.trigger_corner_radii_override,
        )
    }
}

pub fn select<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<Option<Arc<str>>>,
    open: Model<bool>,
    items: &[SelectItem],
    placeholder: Arc<str>,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    layout: LayoutRefinement,
) -> AnyElement {
    let entries: Vec<SelectEntry> = items.iter().cloned().map(SelectEntry::Item).collect();
    select_impl(
        cx,
        model,
        open,
        None,
        &entries,
        placeholder,
        disabled,
        None,
        a11y_label,
        false,
        None,
        ChromeRefinement::default(),
        layout,
        SelectStyle::default(),
        SelectAlign::default(),
        SelectSide::default(),
        Px(0.0),
        None,
        SelectPosition::default(),
        true,
        false,
        None,
        None,
        BorderWidthOverride::default(),
        None,
    )
}

fn select_impl<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<Option<Arc<str>>>,
    open: Model<bool>,
    on_value_change: Option<
        Arc<dyn Fn(&mut dyn fret_ui::action::UiActionHost, ActionCx, Arc<str>) + 'static>,
    >,
    entries: &[SelectEntry],
    placeholder: Arc<str>,
    disabled: bool,
    trigger_test_id: Option<Arc<str>>,
    a11y_label: Option<Arc<str>>,
    aria_invalid: bool,
    on_dismiss_request: Option<OnDismissRequest>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    style: SelectStyle,
    align: SelectAlign,
    side: SelectSide,
    align_offset: Px,
    side_offset_override: Option<Px>,
    position: SelectPosition,
    loop_navigation: bool,
    arrow: bool,
    arrow_size_override: Option<Px>,
    arrow_padding_override: Option<Px>,
    trigger_border_width_override: BorderWidthOverride,
    trigger_corner_radii_override: Option<Corners>,
) -> AnyElement {
    let chrome = ChromeRefinement::default()
        .pl(Space::N2p5)
        .pr(Space::N2)
        .py(Space::N2)
        .merge(chrome);

    cx.scope(|cx| {
        let trigger_test_id = trigger_test_id.clone();
        fn find_item_label(entries: &[SelectEntry], value: &str) -> Option<Arc<str>> {
            for entry in entries {
                match entry {
                    SelectEntry::Item(it) => {
                        if it.value.as_ref() == value {
                            return Some(it.label.clone());
                        }
                    }
                    SelectEntry::Group(group) => {
                        if let Some(label) = find_item_label(&group.entries, value) {
                            return Some(label);
                        }
                    }
                    SelectEntry::Label(_) | SelectEntry::Separator(_) => {}
                }
            }
            None
        }

        fn count_items(entries: &[SelectEntry]) -> usize {
            let mut count: usize = 0;
            for entry in entries {
                match entry {
                    SelectEntry::Item(_) => count = count.saturating_add(1),
                    SelectEntry::Group(group) => count = count.saturating_add(count_items(&group.entries)),
                    SelectEntry::Label(_) | SelectEntry::Separator(_) => {}
                }
            }
            count
        }

        let theme = Theme::global(&*cx.app).clone();
        let selected = cx.watch_model(&model).cloned().unwrap_or_default();
        let is_open = cx.watch_model(&open).layout().copied().unwrap_or(false);
        let motion = radix_presence::scale_fade_presence_with_durations_and_easing(
            cx,
            is_open,
            overlay_motion::SHADCN_MOTION_TICKS_100,
            overlay_motion::SHADCN_MOTION_TICKS_100,
            0.95,
            1.0,
            overlay_motion::shadcn_ease,
        );
        let overlay_presence = OverlayPresence {
            present: motion.present,
            interactive: is_open,
        };
        let arrow_size = arrow_size_override.unwrap_or_else(|| {
            theme
                .metric_by_key("component.select.arrow_size")
                .or_else(|| theme.metric_by_key("component.popover.arrow_size"))
                .unwrap_or(Px(12.0))
        });
        let arrow_padding = arrow_padding_override.unwrap_or_else(|| {
            theme
                .metric_by_key("component.select.arrow_padding")
                .or_else(|| theme.metric_by_key("component.popover.arrow_padding"))
                .unwrap_or_else(|| theme.metric_required("metric.radius.md"))
        });

        let resolved = resolve_input_chrome(
            &theme,
            fret_ui_kit::Size::default(),
            &chrome,
            InputTokenKeys::none(),
        );

        let radius = resolved.radius;
        let mut ring = decl_style::focus_ring(&theme, radius);

        let label = selected
            .as_ref()
            .and_then(|v| find_item_label(entries, v.as_ref()))
            .unwrap_or(placeholder);

        let text_style = TextStyle {
            font: FontId::default(),
            size: resolved.text_px,
            weight: FontWeight::NORMAL,
            slant: Default::default(),
            line_height: theme
                .metric_by_key("font.line_height")
                .or(Some(theme.metric_required("font.line_height"))),
            letter_spacing_em: None,
        };

        let min_width = theme
            .metric_by_key("component.select.min_width")
            .unwrap_or(Px(128.0));

        let trigger_layout = decl_style::layout_style(
            &theme,
            LayoutRefinement::default()
                .h_px(resolved.min_height)
                .merge(layout),
        );

        let mut border = resolved.border_color;
        let mut border_focus = resolved.border_color_focused;
        let fg = resolved.text_color;
        let fg_muted = theme
            .color_by_key("muted-foreground")
            .unwrap_or_else(|| theme.color_required("muted-foreground"));

        if aria_invalid {
            let border_color = theme.color_required("destructive");
            border = border_color;
            border_focus = border_color;

            let ring_key = if theme.name.contains("/dark") {
                "destructive/40"
            } else {
                "destructive/20"
            };
            ring.color = theme
                .color_by_key(ring_key)
                .or_else(|| theme.color_by_key("destructive/20"))
                .unwrap_or(border_color);
        }

        let style_for_trigger = style.clone();
        let style_for_options = style;

        let enabled = !disabled;
        let item_len = count_items(entries);

        #[derive(Debug)]
        struct SelectTriggerKeyState {
            trigger: radix_select::SelectTriggerKeyState,
            pointer: radix_select::SelectTriggerPointerState,
            content: radix_select::SelectContentKeyState,
            was_open: bool,
            scroll_handle: fret_ui::scroll::ScrollHandle,
            value_node: Option<GlobalElementId>,
            viewport: Option<GlobalElementId>,
            listbox: Option<GlobalElementId>,
            content_panel: Option<GlobalElementId>,
            selected_item: Option<GlobalElementId>,
            selected_item_text: Option<GlobalElementId>,
            alignment_item_pos: Option<usize>,
            alignment_item_has_leading_non_item: bool,
            width_probe: Option<GlobalElementId>,
            pending_item_aligned_scroll_to_y: Option<Px>,
            last_item_aligned_scroll_to_y: Option<Px>,
            item_aligned_user_scrolled: bool,
            did_item_aligned_scroll_initial: bool,
            did_item_aligned_scroll_reposition: bool,
            did_item_aligned_focus_scroll: bool,
            item_aligned_scroll_up_visible: bool,
            pending_active_align_top_scroll: bool,
            pending_active_scroll_into_view: bool,
        }

        impl SelectTriggerKeyState {
            fn new() -> Self {
                Self {
                    trigger: radix_select::SelectTriggerKeyState::default(),
                    pointer: radix_select::SelectTriggerPointerState::default(),
                    content: radix_select::SelectContentKeyState::default(),
                    was_open: false,
                    scroll_handle: fret_ui::scroll::ScrollHandle::default(),
                    value_node: None,
                    viewport: None,
                    listbox: None,
                    content_panel: None,
                    selected_item: None,
                    selected_item_text: None,
                    alignment_item_pos: None,
                    alignment_item_has_leading_non_item: false,
                    width_probe: None,
                    pending_item_aligned_scroll_to_y: None,
                    last_item_aligned_scroll_to_y: None,
                    item_aligned_user_scrolled: false,
                    did_item_aligned_scroll_initial: false,
                    did_item_aligned_scroll_reposition: false,
                    did_item_aligned_focus_scroll: false,
                    item_aligned_scroll_up_visible: false,
                    pending_active_align_top_scroll: false,
                    pending_active_scroll_into_view: false,
                }
            }
        }

        fn flatten_items_for_typeahead(
            entries: &[SelectEntry],
            enabled: bool,
            values: &mut Vec<Arc<str>>,
            labels: &mut Vec<Arc<str>>,
            disabled: &mut Vec<bool>,
        ) {
            for entry in entries {
                match entry {
                    SelectEntry::Item(item) => {
                        values.push(item.value.clone());
                        labels.push(item.label.clone());
                        disabled.push(item.disabled || !enabled);
                    }
                    SelectEntry::Group(group) => {
                        flatten_items_for_typeahead(&group.entries, enabled, values, labels, disabled);
                    }
                    SelectEntry::Label(_) | SelectEntry::Separator(_) => {}
                }
            }
        }

        // `control_chrome_pressable_with_id_props` stores handlers; keep a dedicated `open` clone
        // for trigger-owned hooks.
        let open_for_trigger = open.clone();
        let trigger_test_id_for_trigger = trigger_test_id.clone();

        let trigger = decl_chrome::control_chrome_pressable_with_id_props(cx, move |cx, st, trigger_id| {
            let mut typeahead_values: Vec<Arc<str>> = Vec::new();
            let mut typeahead_labels: Vec<Arc<str>> = Vec::new();
            let mut typeahead_disabled: Vec<bool> = Vec::new();
            flatten_items_for_typeahead(
                entries,
                enabled,
                &mut typeahead_values,
                &mut typeahead_labels,
                &mut typeahead_disabled,
            );

            let typeahead_values: Arc<[Arc<str>]> = Arc::from(typeahead_values.into_boxed_slice());
            let typeahead_labels: Arc<[Arc<str>]> = Arc::from(typeahead_labels.into_boxed_slice());
            let typeahead_disabled: Arc<[bool]> = Arc::from(typeahead_disabled.into_boxed_slice());

            let trigger_state: Arc<Mutex<SelectTriggerKeyState>> = cx.with_state_for(
                trigger_id,
                || Arc::new(Mutex::new(SelectTriggerKeyState::new())),
                |s| s.clone(),
            );
            let mouse_open_guard: radix_select::SelectMouseOpenGuard = cx.with_state_for(
                trigger_id,
                || radix_select::select_mouse_open_guard(),
                |g| g.clone(),
            );

            let state_for_timer = trigger_state.clone();
            cx.timer_on_timer_for(
                trigger_id,
                Arc::new(move |_host, _action_cx, token| {
                    let mut state = state_for_timer
                        .lock()
                        .unwrap_or_else(|e| e.into_inner());
                    state.trigger.on_timer(token) || state.content.on_timer(token)
                }),
            );

            let open_for_key = open_for_trigger.clone();
            let model_for_key = model.clone();
            let values_for_key = typeahead_values.clone();
            let labels_for_key = typeahead_labels.clone();
            let disabled_for_key = typeahead_disabled.clone();
            let state_for_key = trigger_state.clone();
            let mouse_open_guard_for_key = mouse_open_guard.clone();
            cx.key_on_key_down_for(
                trigger_id,
                Arc::new(move |host, action_cx, it| {
                    let mut state = state_for_key
                        .lock()
                        .unwrap_or_else(|e| e.into_inner());
                    radix_select::select_mouse_open_guard_clear(&mouse_open_guard_for_key);
                    state.trigger.handle_key_down_when_closed(
                        host,
                        action_cx.window,
                        &open_for_key,
                        &model_for_key,
                        values_for_key.as_ref(),
                        labels_for_key.as_ref(),
                        disabled_for_key.as_ref(),
                        it.key,
                        it.modifiers,
                        it.repeat,
                    )
                }),
            );

            let open_for_pointer_down = open_for_trigger.clone();
            let state_for_pointer_down = trigger_state.clone();
            let mouse_open_guard_for_pointer_down = mouse_open_guard.clone();
            let enabled_for_pointer_down = enabled;
            cx.pressable_add_on_pointer_down(Arc::new(move |host, action_cx, down| {
                if !matches!(
                    down.pointer_type,
                    fret_core::PointerType::Mouse | fret_core::PointerType::Unknown
                ) {
                    return fret_ui::action::PressablePointerDownResult::Continue;
                }

                let was_open = host
                    .models_mut()
                    .get_copied(&open_for_pointer_down)
                    .unwrap_or(false);

                let mut state = state_for_pointer_down
                    .lock()
                    .unwrap_or_else(|e| e.into_inner());

                let handled = state.pointer.handle_pointer_down(
                    host,
                    action_cx,
                    down,
                    &open_for_pointer_down,
                    enabled_for_pointer_down,
                );
                if !handled {
                    return fret_ui::action::PressablePointerDownResult::Continue;
                }

                let now_open = host
                    .models_mut()
                    .get_copied(&open_for_pointer_down)
                    .unwrap_or(false);
                radix_select::select_mouse_open_guard_record_if_opened(
                    &mouse_open_guard_for_pointer_down,
                    was_open,
                    now_open,
                    down.position,
                );
                state.trigger.clear_typeahead(host);

                fret_ui::action::PressablePointerDownResult::SkipDefaultAndStopPropagation
            }));

            let open_for_activate = open_for_trigger.clone();
            let state_for_activate = trigger_state.clone();
            let mouse_open_guard_for_activate = mouse_open_guard.clone();
            cx.pressable_add_on_activate(Arc::new(move |host, action_cx, _reason| {
                let mut state = state_for_activate
                    .lock()
                    .unwrap_or_else(|e| e.into_inner());

                if state.trigger.take_suppress_next_activate() {
                    return;
                }

                radix_select::select_mouse_open_guard_clear(&mouse_open_guard_for_activate);
                state.trigger.clear_typeahead(host);

                let _ = host.models_mut().update(&open_for_activate, |v| *v = true);
                host.request_redraw(action_cx.window);
            }));

            let mut states = WidgetStates::from_pressable(cx, st, enabled);
            states.set(WidgetState::Open, is_open);

            let highlight = ColorRef::Color(alpha_mul(border_focus, 0.85));
            let default_border_color = WidgetStateProperty::new(ColorRef::Color(border))
                .when(WidgetStates::HOVERED, highlight.clone())
                .when(WidgetStates::ACTIVE, highlight.clone())
                .when(WidgetStates::FOCUS_VISIBLE, highlight.clone())
                .when(WidgetStates::OPEN, highlight);

            let border_color = resolve_override_slot(
                style_for_trigger.trigger_border_color.as_ref(),
                &default_border_color,
                states,
            )
            .resolve(&theme);

            let mut props = PressableProps {
                layout: trigger_layout,
                enabled,
                focusable: enabled,
                focus_ring: Some(ring),
                a11y: radix_select::select_trigger_a11y(a11y_label.clone(), is_open, None),
                ..Default::default()
            };
            props.a11y.test_id = trigger_test_id_for_trigger.clone();

            // Radix Select uses `hideOthers(content)` (aria-hide outside) and disables outside
            // pointer events while open. In Fret we approximate that by installing a modal barrier
            // layer (blocks underlay input + gates accessibility roots) even though the content
            // itself remains `role=listbox` (not a dialog).
            let overlay_root_name = radix_select::select_root_name(trigger_id);
            let listbox_id_for_trigger =
                radix_select::select_listbox_semantics_id(cx, overlay_root_name.as_str());
            props.a11y.controls_element = Some(listbox_id_for_trigger.0);
            props.a11y.test_id = trigger_test_id.clone();

            if motion.present && enabled {
                let debug_item_aligned = std::env::var("FRET_DEBUG_SELECT_ITEM_ALIGNED")
                    .ok()
                    .is_some_and(|v| v == "1");
                if debug_item_aligned {
                    eprintln!(
                        "select trigger chrome: padding(l={}, r={}, t={}, b={}) border_w={} min_h={}",
                        resolved.padding.left.0,
                        resolved.padding.right.0,
                        resolved.padding.top.0,
                        resolved.padding.bottom.0,
                        resolved.border_width.0,
                        resolved.min_height.0
                    );
                    eprintln!(
                        "select trigger theme metrics: metric.padding.sm={:?} metric.padding.md={:?} component.input.padding_x={:?} component.input.padding_y={:?} component.size.md.input.px={:?} component.size.md.input.py={:?}",
                        theme.metric_by_key("metric.padding.sm").map(|v| v.0),
                        theme.metric_by_key("metric.padding.md").map(|v| v.0),
                        theme.metric_by_key("component.input.padding_x").map(|v| v.0),
                        theme.metric_by_key("component.input.padding_y").map(|v| v.0),
                        theme.metric_by_key("component.size.md.input.px").map(|v| v.0),
                        theme.metric_by_key("component.size.md.input.py").map(|v| v.0),
                    );
                }

                // Anchor bounds are derived from the previous layout pass. When `open=true` before
                // the first layout (or immediately after a large tree change), the anchor may be
                // missing for a frame. We still install the modal barrier layer to preserve Radix
                // Select's "disable outside pointer events" outcome.
                if let Some(anchor) = overlay::anchor_bounds_for_element(cx, trigger_id) {
                    let dismiss_on_overlay_press = true;
                    let window_margin = theme
                        .metric_by_key("component.select.window_margin")
                        .unwrap_or(Px(0.0));
                    let outer = overlay::outer_bounds_with_window_margin(cx.bounds, window_margin);

                    let item_h = theme
                        .metric_by_key("component.select.item_height")
                        .unwrap_or(Px(32.0));

                    let border_width = resolved.border_width;
                    let direction = direction_prim::use_direction_in_scope(cx, None);

                    let (
                        item_aligned_inputs,
                        _did_item_aligned_scroll_initial,
                        _did_item_aligned_scroll_reposition,
                        _item_aligned_scroll_up_visible,
                    ) = if position == SelectPosition::ItemAligned {
                        let (
                            value_node,
                            viewport,
                            listbox,
                            content_panel,
                            width_probe,
                            selected_item,
                            selected_item_text,
                            alignment_item_pos,
                            alignment_item_has_leading_non_item,
                            did_item_aligned_scroll_initial,
                            did_item_aligned_scroll_reposition,
                            item_aligned_scroll_up_visible,
                        ) = {
                            let state = trigger_state.lock().unwrap_or_else(|e| e.into_inner());
                            (
                                state.value_node,
                                state.viewport,
                                state.listbox,
                                state.content_panel,
                                state.width_probe,
                                state.selected_item,
                                state.selected_item_text,
                                state.alignment_item_pos,
                                state.alignment_item_has_leading_non_item,
                                state.did_item_aligned_scroll_initial,
                                state.did_item_aligned_scroll_reposition,
                                state.item_aligned_scroll_up_visible,
                            )
                        };

                        let item_aligned_inputs = if let (
                            Some(value_node),
                            Some(viewport),
                            Some(listbox),
                            Some(content_panel),
                            Some(selected_item),
                            Some(selected_item_text),
                        ) = (
                            value_node,
                            viewport,
                            listbox,
                            content_panel,
                            selected_item,
                            selected_item_text,
                        ) {
                            if debug_item_aligned
                                && std::env::var("FRET_DEBUG_SELECT_ITEM_ALIGNED")
                                    .ok()
                                    .is_some_and(|v| v == "1")
                            {
                                eprintln!(
                                    "select item-aligned state: did_initial_scroll={} did_reposition_scroll={} scroll_up_visible={}",
                                    did_item_aligned_scroll_initial,
                                    did_item_aligned_scroll_reposition,
                                    item_aligned_scroll_up_visible
                                );
                            }
                            let (selected_item_is_first, selected_item_is_last) = alignment_item_pos
                                .and_then(|pos| (item_len > 0).then_some(pos))
                                .map(|pos| {
                                    let is_first_item = pos == 0 && !alignment_item_has_leading_non_item;
                                    let is_last_item = pos + 1 == item_len;
                                    (is_first_item, is_last_item)
                                })
                                .unwrap_or((false, false));
                            if debug_item_aligned {
                                eprintln!("select item-aligned theme min_width={}", min_width.0);
                                eprintln!(
                                    "select item-aligned window bounds={:?} trigger={:?}",
                                    cx.bounds, anchor
                                );
                                let dbg = |label: &str, id: GlobalElementId| {
                                    let b = overlay::anchor_bounds_for_element(cx, id);
                                    eprintln!("select item-aligned {label}: id={id:?} bounds={b:?}");
                                };
                                dbg("value_node", value_node);
                                dbg("viewport", viewport);
                                dbg("listbox", listbox);
                                dbg("content_panel", content_panel);
                                if let Some(width_probe) = width_probe {
                                    dbg("width_probe", width_probe);
                                }
                                dbg("selected_item", selected_item);
                                dbg("selected_item_text", selected_item_text);
                            }
                            Some(radix_select::SelectItemAlignedElementInputs {
                                direction,
                                window: cx.bounds,
                                trigger: anchor,
                                content_min_width: min_width,
                                content_border_top: border_width,
                                content_padding_top: Px(0.0),
                                content_border_bottom: border_width,
                                content_padding_bottom: Px(0.0),
                                viewport_padding_top: Px(4.0),
                                viewport_padding_bottom: Px(4.0),
                                selected_item_is_first,
                                selected_item_is_last,
                                value_node,
                                viewport,
                                listbox,
                                content_panel,
                                content_width_probe: width_probe,
                                selected_item,
                                selected_item_text,
                            })
                        } else {
                            None
                        };
                        (
                            item_aligned_inputs,
                            did_item_aligned_scroll_initial,
                            did_item_aligned_scroll_reposition,
                            item_aligned_scroll_up_visible,
                        )
                    } else {
                        (None, false, false, false)
                    };

                    let side_offset = side_offset_override.unwrap_or_else(|| {
                        if position == SelectPosition::Popper {
                            // shadcn/ui v4 uses a CSS `translate-*` recipe on the content element
                            // when `position="popper"` instead of a Popper `sideOffset`.
                            //
                            // Keep the Popper wrapper flush to the trigger so placement checks
                            // match upstream `data-radix-popper-content-wrapper` rects.
                            Px(0.0)
                        } else {
                            theme
                                .metric_by_key("component.select.popover_offset")
                                .unwrap_or(Px(6.0))
                        }
                    });
                    let (arrow_options, arrow_protrusion) =
                        popper::diamond_arrow_options(arrow, arrow_size, arrow_padding);
                    let popper_placement = popper::PopperContentPlacement::new(
                        direction,
                        side.into(),
                        align.into(),
                        side_offset,
                    )
                    .with_align_offset(align_offset)
                    .with_arrow(arrow_options, arrow_protrusion);
                    let popper_placement = if position == SelectPosition::Popper {
                        // Radix Select uses a default collision padding of 10px for popper-positioned
                        // content. This keeps the wrapper from touching the window edges when the
                        // trigger is near the boundary.
                        popper_placement
                            .with_shift_cross_axis(true)
                            .with_collision_padding(Edges::all(Px(10.0)))
                    } else {
                        popper_placement
                    };

                    let width_probe_w = {
                        let probe = trigger_state
                            .lock()
                            .unwrap_or_else(|e| e.into_inner())
                            .width_probe;
                        probe
                            .and_then(|id| cx.last_bounds_for_element(id))
                            .map(|rect| rect.size.width)
                    };
                    let desired_w = if let Some(probe_w) = width_probe_w {
                        let border_extra = Px(border_width.0 * 2.0);
                        Px(probe_w.0 + border_extra.0)
                    } else {
                        radix_select::select_popper_desired_width(outer, anchor, min_width)
                    };
                    let desired_w = Px(desired_w.0.max(min_width.0).min(outer.size.width.0));
                    if std::env::var("FRET_DEBUG_SELECT_POPPER_WIDTH")
                        .ok()
                        .is_some_and(|v| v == "1")
                    {
                        eprintln!(
                            "select popper desired_w: probe={:?} anchor_w={} min_w={} border_w={} -> {}",
                            width_probe_w.map(|v| v.0),
                            anchor.size.width.0,
                            min_width.0,
                            border_width.0,
                            desired_w.0
                        );
                    }

                    // new-york-v4 uses Radix's `--radix-select-content-available-height` which adapts
                    // to the current window + trigger placement. Prefer that behavior by computing
                    // the available height from our popper substrate, while still allowing an
                    // explicit theme override for apps that want a fixed cap.
                    let available_h = (position == SelectPosition::Popper)
                        .then(|| {
                            let probe_desired = fret_core::Size::new(desired_w, outer.size.height);
                            let layout = popper::popper_content_layout_sized(
                                outer,
                                anchor,
                                probe_desired,
                                popper_placement,
                            );
                            popper::popper_available_metrics(
                                outer,
                                anchor,
                                &layout,
                                popper_placement.direction,
                            )
                            .available_height
                        })
                        .unwrap_or(outer.size.height);
                    let max_h = theme
                        .metric_by_key("component.select.max_list_height")
                        .map(|h| Px(h.0.min(available_h.0)))
                        .unwrap_or(available_h);

                    // shadcn/ui v4 Select content wrapper includes:
                    // - `p-1` viewport padding
                    // - `border` width
                    //
                    // Compute the desired height in terms of the scrollable content and then add
                    // the wrapper chrome so placement/size checks match the upstream popper wrapper.
                    let content_padding_y = Px(4.0 * 2.0);
                    let border_y = Px(border_width.0 * 2.0);
                    let chrome_extra_y = Px(content_padding_y.0 + border_y.0);

                    let max_content_h = Px((max_h.0 - chrome_extra_y.0).max(0.0));
                    let outer_content_h = Px((outer.size.height.0 - chrome_extra_y.0).max(0.0));
                    let desired_content_h = select_list_desired_height(
                        item_h,
                        item_len,
                        max_content_h,
                        outer_content_h,
                    );
                    let desired_h = Px(desired_content_h.0 + chrome_extra_y.0);
                    let desired = fret_core::Size::new(desired_w, desired_h);

                    let resolved = radix_select::select_resolve_content_placement_from_elements(
                        cx,
                        anchor,
                        outer,
                        desired,
                        popper_placement,
                        arrow.then_some(arrow_size),
                        item_aligned_inputs,
                    );
                    if let Some(layout) = resolved.item_aligned_layout
                        && let Some(scroll_to) = layout.outputs.scroll_to_y
                    {
                        // Radix repositions once after the scroll-up button mounts (it shifts the
                        // viewport down in the normal flow). Model this as an initial scroll plus
                        // a single follow-up scroll if the viewport became scrollable at the top.
                        let mut state = trigger_state.lock().unwrap_or_else(|e| e.into_inner());
                        if !state.item_aligned_user_scrolled {
                            if let Some(last) = state.last_item_aligned_scroll_to_y
                                && state.did_item_aligned_scroll_initial
                            {
                                let offset = state.scroll_handle.offset();
                                let drift = (offset.y.0 - last.0).abs();
                                let drift_threshold = item_h.0 * 2.0;
                                if drift > drift_threshold {
                                    state.item_aligned_user_scrolled = true;
                                }
                            }
                        }

                        let should_scroll_initial =
                            !state.did_item_aligned_scroll_initial && !state.item_aligned_user_scrolled;
                        let should_scroll_reposition = state.did_item_aligned_scroll_initial
                            && !state.did_item_aligned_scroll_reposition
                            && state.item_aligned_scroll_up_visible
                            && !state.item_aligned_user_scrolled;
                        if should_scroll_initial || should_scroll_reposition {
                            if std::env::var("FRET_DEBUG_SELECT_ITEM_ALIGNED")
                                .ok()
                                .is_some_and(|v| v == "1")
                            {
                                eprintln!(
                                    "select item-aligned requested scroll_to_y={} initial={} reposition={}",
                                    scroll_to.0, should_scroll_initial, should_scroll_reposition
                                );
                            }
                            state.pending_item_aligned_scroll_to_y = Some(scroll_to);
                            state.last_item_aligned_scroll_to_y = Some(scroll_to);
                            if should_scroll_initial {
                                state.did_item_aligned_scroll_initial = true;
                            }
                            if should_scroll_reposition {
                                state.did_item_aligned_scroll_reposition = true;
                            }
                        }
                    }
                    let placement = resolved.placement;
                    let wrapper_insets = placement.wrapper_insets;
                    let motion_side = placement.side;
                    let transform_origin = placement.transform_origin;
                    let popper_layout = placement.popper_layout;
                    let placed = placement.placed;

                    let opacity = motion.opacity;
                    let scale = motion.scale;
                    let transform = overlay_motion::shadcn_popper_presence_transform(
                        motion_side,
                        transform_origin,
                        opacity,
                        scale,
                        is_open,
                    );

                    let theme_for_overlay = theme.clone();
                    let text_style_for_overlay = text_style.clone();
                    let open_for_overlay = open_for_trigger.clone();
                    let trigger_state_for_overlay = trigger_state.clone();
                    let viewport_id_out_cell = Cell::new(None::<GlobalElementId>);
                    let viewport_id_out = &viewport_id_out_cell;
                    let active_element_id_out_cell = Cell::new(None::<GlobalElementId>);
                    let active_element_id_out = &active_element_id_out_cell;
                    let listbox_id_out_cell = Cell::new(None::<GlobalElementId>);
                    let listbox_id_out = &listbox_id_out_cell;
                    let content_panel_id_out_cell = Cell::new(None::<GlobalElementId>);
                    let content_panel_id_out = &content_panel_id_out_cell;
                    let width_probe_id_out_cell = Cell::new(None::<GlobalElementId>);
                    let width_probe_id_out = &width_probe_id_out_cell;
                    let selected_item_id_out_cell = Cell::new(None::<GlobalElementId>);
                    let selected_item_id_out = &selected_item_id_out_cell;
                    let selected_item_id_for_request_cell = Cell::new(None::<GlobalElementId>);
                    let selected_item_id_for_request = &selected_item_id_for_request_cell;
                    let selected_item_text_id_out_cell = Cell::new(None::<GlobalElementId>);
                    let selected_item_text_id_out = &selected_item_text_id_out_cell;
                    let alignment_item_pos_out_cell = Cell::new(None::<usize>);
                    let alignment_item_pos_out = &alignment_item_pos_out_cell;
                    let alignment_item_has_leading_non_item_out_cell = Cell::new(None::<bool>);
                    let alignment_item_has_leading_non_item_out =
                        &alignment_item_has_leading_non_item_out_cell;
                    let trigger_state_for_overlay_for_children = trigger_state_for_overlay.clone();
                    let popper_layout_for_children = popper_layout;
                    let mouse_open_guard_for_overlay = mouse_open_guard.clone();
                    let on_dismiss_request_for_overlay_children = on_dismiss_request.clone();
                    let on_value_change_for_overlay_children = on_value_change.clone();

                    let overlay_children = cx.with_root_name(&overlay_root_name, move |cx| {
                        let trigger_state_for_overlay = trigger_state_for_overlay_for_children.clone();
                        let open_for_content = open_for_overlay.clone();
                        let open_for_barrier_children = open_for_overlay.clone();
                        let mouse_open_guard_for_barrier_children = mouse_open_guard_for_overlay.clone();

                        let selected = cx.watch_model(&model).cloned().unwrap_or_default();
                        let on_value_change = on_value_change_for_overlay_children.clone();

                        #[derive(Clone)]
                        enum SelectRow {
                            Item(SelectItem),
                            Label(SelectLabel),
                            Separator,
                        }

                        fn flatten_entries(into: &mut Vec<SelectRow>, entries: &[SelectEntry]) {
                            for entry in entries {
                                match entry {
                                    SelectEntry::Item(item) => into.push(SelectRow::Item(item.clone())),
                                    SelectEntry::Label(label) => into.push(SelectRow::Label(label.clone())),
                                    SelectEntry::Group(group) => flatten_entries(into, &group.entries),
                                    SelectEntry::Separator(_) => into.push(SelectRow::Separator),
                                }
                            }
                        }

                        let mut rows: Vec<SelectRow> = Vec::new();
                        flatten_entries(&mut rows, entries);

                        let item_count = rows
                            .iter()
                            .filter(|r| matches!(r, SelectRow::Item(_)))
                            .count();

                        let disabled: Vec<bool> = rows
                            .iter()
                            .map(|row| match row {
                                SelectRow::Item(item) => item.disabled || !enabled,
                                SelectRow::Label(_) | SelectRow::Separator => true,
                            })
                            .collect();

                        let labels: Vec<Arc<str>> = rows
                            .iter()
                            .map(|row| match row {
                                SelectRow::Item(item) => item.label.clone(),
                                SelectRow::Label(_) | SelectRow::Separator => Arc::from(""),
                            })
                            .collect();
                        let labels_arc: Arc<[Arc<str>]> = Arc::from(labels.into_boxed_slice());

                        let initial_active_row = if let Some(selected) = selected.as_deref() {
                            let selected_idx = rows.iter().position(|row| match row {
                                SelectRow::Item(item) => item.value.as_ref() == selected,
                                SelectRow::Label(_) | SelectRow::Separator => false,
                            });
                            selected_idx
                                .and_then(|idx| (!disabled.get(idx).copied().unwrap_or(true)).then_some(idx))
                                .or_else(|| roving_focus_group::first_enabled(&disabled))
                        } else {
                            roving_focus_group::first_enabled(&disabled)
                        };
                        let active_row = {
                            let mut state = trigger_state_for_overlay
                                .lock()
                                .unwrap_or_else(|e| e.into_inner());

                            if is_open {
                                if !state.was_open {
                                    state.was_open = true;
                                    state.content.reset_on_open(initial_active_row);
                                    state.trigger.reset_typeahead_buffer();
                                    state.pending_active_align_top_scroll = true;
                            } else if state.content.active_row().is_none() {
                                state.content.set_active_row(initial_active_row);
                            }
                        } else {
                            state.was_open = false;
                            state.content.set_active_row(None);
                            state.pending_active_align_top_scroll = false;
                        }

                        state.content.active_row()
                    };

                        let shadow = decl_style::shadow_md(&theme_for_overlay, radius);
                        let arrow_bg = theme_for_overlay.colors.panel_background;
                        let overlay_border = theme_for_overlay
                            .color_by_key("border")
                            .unwrap_or_else(|| theme_for_overlay.color_required("border"));
                        let arrow_border = overlay_border;
                                        let initial_scroll_to_y = {
                                            let mut state = trigger_state_for_overlay
                                                .lock()
                                                .unwrap_or_else(|e| e.into_inner());
                                            state.pending_item_aligned_scroll_to_y.take()
                                        };
                                        let scroll_handle = {
                                            let state = trigger_state_for_overlay
                                                .lock()
                                                .unwrap_or_else(|e| e.into_inner());
                                            state.scroll_handle.clone()
                                        };

                        let probe = cx.container(
                            ContainerProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.position = PositionStyle::Absolute;
                                    layout.inset.left = Some(Px(-10000.0));
                                    layout.inset.top = Some(Px(0.0));
                                    layout
                                },
                                // new-york-v4: `SelectViewport` uses `p-1`.
                                padding: Edges::all(Px(4.0)),
                                ..Default::default()
                            },
                            |cx| {
                                let mut out: Vec<AnyElement> = Vec::new();
                                for row in rows.iter().cloned() {
                                    let SelectRow::Item(item) = row else {
                                        continue;
                                    };
                                    let label = item.label.clone();
                                    let style = text_style_for_overlay.clone();
                                    out.push(cx.container(
                                        ContainerProps {
                                            // new-york-v4: `py-1.5 pl-2 pr-8`
                                            padding: Edges {
                                                top: Px(6.0),
                                                right: Px(32.0),
                                                bottom: Px(6.0),
                                                left: Px(8.0),
                                            },
                                            ..Default::default()
                                        },
                                        |cx| {
                                            let mut text = ui::text(cx, label)
                                                .text_size_px(style.size)
                                                .font_weight(style.weight)
                                                .nowrap();
                                            if let Some(line_height) = style.line_height {
                                                text = text.line_height_px(line_height);
                                            }
                                            if let Some(letter_spacing_em) = style.letter_spacing_em {
                                                text = text.letter_spacing_em(letter_spacing_em);
                                            }
                                            vec![text.into_element(cx)]
                                        },
                                    ));
                                }
                                out
                            },
                        );
                        width_probe_id_out.set(Some(probe.id));

                        let trigger_state_for_overlay_in_content = trigger_state_for_overlay.clone();
                        let mouse_open_guard_for_content = mouse_open_guard_for_barrier_children.clone();
                        let content = popper_content::popper_wrapper_at(cx, placed, wrapper_insets, move |cx| {
                                let arrow_el = popper_layout_for_children.as_ref().and_then(|layout| {
                                    popper_arrow::diamond_arrow_element(
                                        cx,
                                        layout,
                                        wrapper_insets,
                                        arrow_size,
                                        DiamondArrowStyle {
                                            bg: arrow_bg,
                                            border: Some(arrow_border),
                                            border_width,
                                        },
                                    )
                                });

                                let panel = radix_select::select_listbox_pressable_with_id_props(
                                    cx,
                                    move |cx, _st, listbox_id| {
                                        content_panel_id_out.set(Some(listbox_id));

                                        let disabled_for_key: Arc<[bool]> =
                                            Arc::from(disabled.clone().into_boxed_slice());
                                        let labels_for_key = labels_arc.clone();
                                        let values_by_row: Arc<[Option<Arc<str>>]> = Arc::from(
                                            rows.iter()
                                                .map(|row| match row {
                                                    SelectRow::Item(item) => {
                                                        Some(item.value.clone())
                                                    }
                                                    SelectRow::Label(_) | SelectRow::Separator => {
                                                        None
                                                    }
                                                })
                                                .collect::<Vec<_>>()
                                                .into_boxed_slice(),
                                        );

                                        let state_for_key =
                                            trigger_state_for_overlay_in_content.clone();
                                        let open_for_key = open_for_content.clone();
                                        let model_for_key = model.clone();
                                        let loop_navigation_for_key = loop_navigation;

                                        cx.key_on_key_down_for(
                                            listbox_id,
                                            Arc::new(move |host, action_cx, it| {
                                                let mut state = state_for_key
                                                    .lock()
                                                    .unwrap_or_else(|e| e.into_inner());
                                                let before_active = state.content.active_row();
                                                let handled = state.content.handle_key_down_when_open(
                                                    host,
                                                    action_cx.window,
                                                    &open_for_key,
                                                    &model_for_key,
                                                    values_by_row.as_ref(),
                                                    labels_for_key.as_ref(),
                                                    disabled_for_key.as_ref(),
                                                    it.key,
                                                    it.repeat,
                                                    loop_navigation_for_key,
                                                );
                                                let after_active = state.content.active_row();
                                                if handled && before_active != after_active {
                                                    state.pending_active_scroll_into_view = true;
                                                }
                                                handled
                                            }),
                                        );

                                        let state_for_align_check =
                                            trigger_state_for_overlay_in_content.clone();
                                        let state_for_align_done =
                                            trigger_state_for_overlay_in_content.clone();
                                        let state_for_scroll_up_visible =
                                            trigger_state_for_overlay_in_content.clone();
                                        let state_for_should_focus_selected_item =
                                            trigger_state_for_overlay_in_content.clone();
                                        let state_for_focused_selected_item =
                                            trigger_state_for_overlay_in_content.clone();
                                        let state_for_consume_active_scroll_into_view =
                                            trigger_state_for_overlay_in_content.clone();

                                        let scroll = select_scroll_with_buttons(
                                            cx,
                                            theme_for_overlay.clone(),
                                            item_h,
                                            scroll_handle,
                                            initial_scroll_to_y,
                                            viewport_id_out,
                                            active_element_id_out,
                                            move || {
                                                let mut state =
                                                    state_for_consume_active_scroll_into_view
                                                    .lock()
                                                    .unwrap_or_else(|e| e.into_inner());
                                                if state.pending_active_scroll_into_view {
                                                    state.pending_active_scroll_into_view = false;
                                                    true
                                                } else {
                                                    false
                                                }
                                            },
                                            move || {
                                                let state = state_for_align_check
                                                    .lock()
                                                    .unwrap_or_else(|e| e.into_inner());
                                                state.pending_active_align_top_scroll
                                                    && !state.did_item_aligned_scroll_initial
                                                    && !state.did_item_aligned_scroll_reposition
                                            },
                                            move || {
                                                let mut state = state_for_align_done
                                                    .lock()
                                                    .unwrap_or_else(|e| e.into_inner());
                                                state.pending_active_align_top_scroll = false;
                                            },
                                            move |visible| {
                                                let mut state = state_for_scroll_up_visible
                                                    .lock()
                                                    .unwrap_or_else(|e| e.into_inner());
                                                state.item_aligned_scroll_up_visible = visible;
                                            },
                                            move || {
                                                let state = state_for_should_focus_selected_item
                                                    .lock()
                                                    .unwrap_or_else(|e| e.into_inner());
                                                let positioned = state.did_item_aligned_scroll_initial
                                                    && (!state.item_aligned_scroll_up_visible
                                                        || state.did_item_aligned_scroll_reposition);
                                                positioned && !state.did_item_aligned_focus_scroll
                                            },
                                            move || {
                                                let mut state = state_for_focused_selected_item
                                                    .lock()
                                                    .unwrap_or_else(|e| e.into_inner());
                                                state.did_item_aligned_focus_scroll = true;
                                            },
                                            move |cx, active_element| {
                                                                let mut out = Vec::with_capacity(rows.len());
                                                                let mut item_ordinal: usize = 0;
                                                                let alignment_selected_value = selected.as_ref().and_then(|value| {
                                                                    rows.iter()
                                                                        .any(|row| match row {
                                                                            SelectRow::Item(item) => item.value.as_ref() == value.as_ref(),
                                                                            SelectRow::Label(_) | SelectRow::Separator => false,
                                                                        })
                                                                        .then(|| value.clone())
                                                                });
                                                                let mut first_valid_alignment_item_found = false;

                                                                for (row_idx, row) in rows.iter().cloned().enumerate() {
                                                                    match row {
                                                                        SelectRow::Label(label) => {
                                                                            let theme = Theme::global(&*cx.app).clone();
                                                                            let fg = theme
                                                                                .color_by_key("muted.foreground")
                                                                                .or_else(|| theme.color_by_key("muted-foreground"))
                                                                                .unwrap_or_else(|| theme.color_required("muted.foreground"));

                                                                            let base_size = theme.metric_required(
                                                                                theme_tokens::metric::COMPONENT_TEXT_SM_PX,
                                                                            );
                                                                            let base_line_height = theme.metric_required(
                                                                                theme_tokens::metric::COMPONENT_TEXT_SM_LINE_HEIGHT,
                                                                            );
                                                                            let label_text_px =
                                                                                Px((base_size.0 - 2.0).max(10.0));
                                                                            let label_line_height = Px(
                                                                                (base_line_height.0 - 4.0)
                                                                                    .max(label_text_px.0),
                                                                            );

                                                                            out.push(cx.container(
                                                                                ContainerProps {
                                                                                    layout: {
                                                                                        let mut layout =
                                                                                            LayoutStyle::default();
                                                                                        layout.size.width = Length::Fill;
                                                                                        layout
                                                                                    },
                                                                                    // new-york-v4: `px-2 py-1.5`
                                                                                    padding: Edges {
                                                                                        top: Px(6.0),
                                                                                        right: Px(8.0),
                                                                                        bottom: Px(6.0),
                                                                                        left: Px(8.0),
                                                                                    },
                                                                                    background: None,
                                                                                    shadow: None,
                                                                                    border: Edges::all(Px(0.0)),
                                                                                    border_color: None,
                                                                                    corner_radii: Corners::all(Px(0.0)),
                                                                                    ..Default::default()
                                                                                },
                                                                                move |cx| {
                                                                                    vec![ui::text(cx, label.text)
                                                                                        .w_full()
                                                                                        .text_size_px(label_text_px)
                                                                                        .line_height_px(label_line_height)
                                                                                        .font_normal()
                                                                                        .text_color(ColorRef::Color(fg))
                                                                                        .nowrap()
                                                                                        .into_element(cx)]
                                                                                },
                                                                            ));
                                                                        }
                                                                        SelectRow::Separator => {
                                                                            let theme = Theme::global(&*cx.app).clone();
                                                                            let border = theme
                                                                                .color_by_key("border")
                                                                                .unwrap_or_else(|| theme.color_required("border"));

                                                                            out.push(cx.container(
                                                                                ContainerProps {
                                                                                    layout: {
                                                                                        let mut layout = LayoutStyle::default();
                                                                                        layout.size.width = Length::Fill;
                                                                                        layout.size.height = Length::Px(Px(1.0));
                                                                                        // new-york-v4: `SelectSeparator` uses `-mx-1 my-1`.
                                                                                        layout.margin.left =
                                                                                            fret_ui::element::MarginEdge::Px(Px(-4.0));
                                                                                        layout.margin.right =
                                                                                            fret_ui::element::MarginEdge::Px(Px(-4.0));
                                                                                        layout.margin.top =
                                                                                            fret_ui::element::MarginEdge::Px(Px(4.0));
                                                                                        layout.margin.bottom =
                                                                                            fret_ui::element::MarginEdge::Px(Px(4.0));
                                                                                        layout
                                                                                    },
                                                                                    background: Some(border),
                                                                                    ..Default::default()
                                                                                },
                                                                                |_cx| Vec::new(),
                                                                            ));
                                                                        }
                                                                        SelectRow::Item(item) => {
                                                                            let item_disabled =
                                                                                disabled.get(row_idx).copied().unwrap_or(true);
                                                                            let is_active =
                                                                                active_row.is_some_and(|a| a == row_idx);
                                                                            let is_selected = alignment_selected_value
                                                                                .as_ref()
                                                                                .is_some_and(|v| v.as_ref() == item.value.as_ref());
                                                                            let is_first_valid_item = alignment_selected_value.is_none()
                                                                                && !first_valid_alignment_item_found
                                                                                && !item_disabled;
                                                                            if is_first_valid_item {
                                                                                first_valid_alignment_item_found = true;
                                                                            }
                                                                            let is_alignment_item =
                                                                                is_selected || is_first_valid_item;

                                                                            let model = model.clone();
                                                                            let open = open_for_content.clone();
                                                                            let text_style = text_style_for_overlay.clone();
                                                                            let on_value_change_for_item =
                                                                                on_value_change.clone();

                                                                            let pos = item_ordinal;
                                                                            item_ordinal = item_ordinal.saturating_add(1);
                                                                            let state_for_hover =
                                                                                trigger_state_for_overlay_in_content
                                                                                    .clone();
                                                                            let row_idx_for_hover = row_idx;
                                                                            let mouse_open_guard_for_item_pointer_up =
                                                                                mouse_open_guard_for_content.clone();

                                                                            let value_key = item.value.clone();
                                                                            let style_for_item =
                                                                                style_for_options.clone();

                                                                            out.push(cx.keyed(value_key, move |cx| {
                                                                                cx.pressable_with_id(
                                                                                PressableProps {
                                                                                    layout: {
                                                                                        let mut layout = LayoutStyle::default();
                                                                                        layout.size.width = Length::Fill;
                                                                                        layout.size.height = Length::Px(item_h);
                                                                                        layout
                                                                                    },
                                                                                    enabled: !item_disabled,
                                                                                    focusable: false,
                                                                                    focus_ring: None,
                                                                                    a11y: PressableA11y {
                                                                                        role: Some(SemanticsRole::ListBoxOption),
                                                                                        label: Some(item.label.clone()),
                                                                                        test_id: item.test_id.clone(),
                                                                                        selected: is_selected,
                                                                                        ..Default::default()
                                                                                    }
                                                                                    .with_collection_position(pos, item_count),
                                                                                    ..Default::default()
                                                                                },
                                                                                move |cx, st, id| {
                                                                                    if is_alignment_item {
                                                                                selected_item_id_out.set(Some(id));
                                                                                selected_item_id_for_request.set(Some(id));
                                                                                        alignment_item_pos_out.set(Some(pos));
                                                                                        alignment_item_has_leading_non_item_out
                                                                                            .set(Some(row_idx_for_hover > 0));
                                                                                    }
                                                                                    if is_active {
                                                                                        active_element.set(Some(id));
                                                                                    }

                                                                                    let item_value = item.value.clone();
                                                                                    let item_label = item.label.clone();
                                                                                     cx.pressable_set_option_arc_str(
                                                                                         &model,
                                                                                         item_value.clone(),
                                                                                     );
                                                                                    cx.pressable_set_bool(&open, false);

                                                                                    if !item_disabled
                                                                                        && let Some(
                                                                                            on_value_change,
                                                                                        ) = on_value_change_for_item.clone()
                                                                                    {
                                                                                        let item_value_for_activate =
                                                                                            item_value.clone();
                                                                                        cx.pressable_add_on_activate(
                                                                                            Arc::new(move |host, action_cx, _reason| {
                                                                                                on_value_change(
                                                                                                    host,
                                                                                                    action_cx,
                                                                                                    item_value_for_activate.clone(),
                                                                                                );
                                                                                            }),
                                                                                        );
                                                                                    }

                                                                                    if !item_disabled {
                                                                                        cx.pressable_add_on_hover_change(Arc::new(
                                                                                            move |host, action_cx, hovered| {
                                                                                                if !hovered {
                                                                                                    return;
                                                                                                }
                                                                                                let mut state = state_for_hover
                                                                                                    .lock()
                                                                                                    .unwrap_or_else(|e| e.into_inner());
                                                                                                if state.content.active_row()
                                                                                                    != Some(row_idx_for_hover)
                                                                                                {
                                                                                                    state.content.set_active_row(
                                                                                                        Some(row_idx_for_hover),
                                                                                                    );
                                                                                                    host.request_redraw(
                                                                                                        action_cx.window,
                                                                                                    );
                                                                                                }
                                                                                            },
                                                                                        ));
                                                                                    }

                                                                                    let theme = Theme::global(&*cx.app).clone();
                                                                                    // new-york-v4: items highlight on focus/hover via `bg-accent`.
                                                                                    let bg_accent = theme
                                                                                        .color_by_key("accent")
                                                                                        .or_else(|| theme.color_by_key("accent.background"))
                                                                                        .unwrap_or_else(|| theme.color_required("accent"));
                                                                                    let fg_accent = theme
                                                                                        .color_by_key("accent-foreground")
                                                                                        .or_else(|| theme.color_by_key("accent.foreground"))
                                                                                        .unwrap_or_else(|| theme.color_required("accent.foreground"));

                                                                                    let item_enabled = !item_disabled;

                                                                                    let mut states = WidgetStates::from_pressable(cx, st, item_enabled);
                                                                                    states.set(WidgetState::Focused, is_active);
                                                                                    states.set(WidgetState::Selected, is_selected);

                                                                                    let default_bg = WidgetStateProperty::new(
                                                                                        ColorRef::Color(Color::TRANSPARENT),
                                                                                    )
                                                                                    .when(
                                                                                        WidgetStates::FOCUSED,
                                                                                        ColorRef::Color(bg_accent),
                                                                                    )
                                                                                    .when(
                                                                                        WidgetStates::HOVERED,
                                                                                        ColorRef::Color(bg_accent),
                                                                                    )
                                                                                    .when(
                                                                                        WidgetStates::ACTIVE,
                                                                                        ColorRef::Color(bg_accent),
                                                                                    )
                                                                                    .when(
                                                                                        WidgetStates::DISABLED,
                                                                                        ColorRef::Color(Color::TRANSPARENT),
                                                                                    );

                                                                                    let default_fg = WidgetStateProperty::new(ColorRef::Color(fg))
                                                                                        .when(
                                                                                            WidgetStates::FOCUSED,
                                                                                            ColorRef::Color(fg_accent),
                                                                                        )
                                                                                        .when(
                                                                                            WidgetStates::HOVERED,
                                                                                            ColorRef::Color(fg_accent),
                                                                                        )
                                                                                        .when(
                                                                                            WidgetStates::ACTIVE,
                                                                                            ColorRef::Color(fg_accent),
                                                                                        )
                                                                                        .when(
                                                                                            WidgetStates::DISABLED,
                                                                                            ColorRef::Color(alpha_mul(fg_muted, 0.8)),
                                                                                        );

                                                                                    let bg = resolve_override_slot(
                                                                                        style_for_item.option_background.as_ref(),
                                                                                        &default_bg,
                                                                                        states,
                                                                                    )
                                                                                    .resolve(&theme);
                                                                                    let fg = resolve_override_slot(
                                                                                        style_for_item.option_foreground.as_ref(),
                                                                                        &default_fg,
                                                                                        states,
                                                                                    )
                                                                                    .resolve(&theme);

                                                                                    let icon = decl_icon::icon_with(
                                                                                        cx,
                                                                                        ids::ui::CHECK,
                                                                                        Some(Px(16.0)),
                                                                                        Some(ColorRef::Color(fg)),
                                                                                    );
                                                                                    let icon = cx.opacity(
                                                                                        if is_selected { 1.0 } else { 0.0 },
                                                                                        move |_cx| vec![icon],
                                                                                    );

                                                                                    vec![cx.pointer_region(
                                                                                        PointerRegionProps {
                                                                                            layout: {
                                                                                                let mut layout =
                                                                                                    LayoutStyle::default();
                                                                                                layout.size.width =
                                                                                                    Length::Fill;
                                                                                                layout.size.height =
                                                                                                    Length::Fill;
                                                                                                layout
                                                                                            },
                                                                                            enabled: true,
                                                                                         },
                                                                                         move |cx| {
                                                                                             let open_for_pointer_up = open.clone();
                                                                                             let model_for_pointer_up = model.clone();
                                                                                             let item_value_for_pointer_up = item_value.clone();
                                                                                             let item_disabled_for_pointer_up = item_disabled;
                                                                                             let mouse_open_guard_for_pointer_up =
                                                                                                 mouse_open_guard_for_item_pointer_up.clone();

                                                                                             cx.pointer_region_on_pointer_up(
                                                                                                 radix_select::select_item_pointer_up_handler(
                                                                                                     open_for_pointer_up,
                                                                                                     model_for_pointer_up,
                                                                                                     item_value_for_pointer_up,
                                                                                                     item_disabled_for_pointer_up,
                                                                                                     mouse_open_guard_for_pointer_up,
                                                                                                 ),
                                                                                             );

                                                                                             vec![cx.container(
                                                                                                ContainerProps {
                                                                                                    layout: {
                                                                                                        let mut layout =
                                                                                                            LayoutStyle::default();
                                                                                                        layout.size.width = Length::Fill;
                                                                                                        layout.size.height = Length::Fill;
                                                                                                        layout
                                                                                                    },
                                                                                                    // new-york-v4: `py-1.5 pl-2 pr-8`
                                                                                                    padding: Edges {
                                                                                                        top: Px(6.0),
                                                                                                        right: Px(32.0),
                                                                                                        bottom: Px(6.0),
                                                                                                        left: Px(8.0),
                                                                                                    },
                                                                                                    background: Some(bg),
                                                                                                    shadow: None,
                                                                                                    border: Edges::all(Px(0.0)),
                                                                                                    border_color: None,
                                                                                                    corner_radii: Corners::all(
                                                                                                        theme.metric_required("metric.radius.sm"),
                                                                                                    ),
                                                                                                    ..Default::default()
                                                                                                },
                                                                                                 |cx| {
                                                                                             let text = cx.container(
                                                                                                 ContainerProps {
                                                                                                    layout: {
                                                                                                        let mut layout =
                                                                                                            LayoutStyle::default();
                                                                                                        layout.size.width =
                                                                                                            Length::Fill;
                                                                                                        layout
                                                                                                    },
                                                                                                    ..Default::default()
                                                                                                },
                                                                                                |cx| {
                                                                                                    let mut text = ui::text(cx, item_label.clone())
                                                                                                        .w_full()
                                                                                                        .text_size_px(text_style.size)
                                                                                                        .font_weight(text_style.weight)
                                                                                                        .text_color(ColorRef::Color(fg))
                                                                                                        .nowrap();
                                                                                                    if let Some(line_height) = text_style.line_height {
                                                                                                        text = text.line_height_px(line_height);
                                                                                                    }
                                                                                                    if let Some(letter_spacing_em) = text_style.letter_spacing_em {
                                                                                                        text = text.letter_spacing_em(letter_spacing_em);
                                                                                                    }
                                                                                                    vec![text.into_element(cx)]
                                                                                                },
                                                                                            );
                                                                                            if is_alignment_item {
                                                                                                selected_item_text_id_out
                                                                                                    .set(Some(text.id));
                                                                                            }

                                                                                            // Indicator slot matches upstream: absolute at the end, but reserve `pr-8`.
                                                                                            let indicator_size = Px(14.0);
                                                                                            let indicator_top = Px(
                                                                                                ((item_h.0 - indicator_size.0)
                                                                                                    * 0.5)
                                                                                                    .max(0.0),
                                                                                            );
                                                                                                 let indicator = cx.container(
                                                                                                     ContainerProps {
                                                                                                         layout: LayoutStyle {
                                                                                                             position: PositionStyle::Absolute,
                                                                                                             inset: InsetStyle {
                                                                                                                 top: Some(indicator_top),
                                                                                                                 right: Some(Px(8.0)),
                                                                                                                 bottom: None,
                                                                                                                 left: None,
                                                                                                             },
                                                                                                             size: SizeStyle {
                                                                                                                 width: Length::Px(
                                                                                                                     indicator_size,
                                                                                                                 ),
                                                                                                                 height: Length::Px(
                                                                                                                     indicator_size,
                                                                                                                 ),
                                                                                                                 ..Default::default()
                                                                                                             },
                                                                                                             ..Default::default()
                                                                                                         },
                                                                                                         padding: Edges::all(Px(0.0)),
                                                                                                         background: None,
                                                                                                         shadow: None,
                                                                                                         border: Edges::all(Px(0.0)),
                                                                                                         border_color: None,
                                                                                                         corner_radii: Corners::all(Px(0.0)),
                                                                                                         ..Default::default()
                                                                                                     },
                                                                                                     |cx| {
                                                                                                         vec![cx.flex(
                                                                                                             FlexProps {
                                                                                                            layout: {
                                                                                                                let mut layout =
                                                                                                                    LayoutStyle::default();
                                                                                                                layout.size.width =
                                                                                                                    Length::Fill;
                                                                                                                layout.size.height =
                                                                                                                    Length::Fill;
                                                                                                                layout
                                                                                                            },
                                                                                                            direction: fret_core::Axis::Horizontal,
                                                                                                            gap: Px(0.0),
                                                                                                            padding: Edges::all(Px(0.0)),
                                                                                                            justify: MainAlign::Center,
                                                                                                            align: CrossAlign::Center,
                                                                                                            wrap: false,
                                                                                                        },
                                                                                                        |_cx| vec![icon.clone()],
                                                                                                    )]
                                                                                                },
                                                                                            );

                                                                                            vec![cx.stack_props(
                                                                                                StackProps {
                                                                                                    layout: {
                                                                                                        let mut layout =
                                                                                                            LayoutStyle::default();
                                                                                                        layout.size.width =
                                                                                                            Length::Fill;
                                                                                                        layout.size.height =
                                                                                                            Length::Fill;
                                                                                                        layout
                                                                                                    },
                                                                                                },
                                                                                                |_cx| vec![text, indicator],
                                                                                            )]
                                                                                        },
                                                                                            )]
                                                                                        },
                                                                                    )]
                                                                                },
                                                                                )
                                                                            }));
                                                                        }
                                                                    }
                                                                }

                                                                let listbox_content = cx.flex(
                                                                    FlexProps {
                                                                        layout: LayoutStyle::default(),
                                                                        direction: fret_core::Axis::Vertical,
                                                                        gap: Px(0.0),
                                                                        padding: Edges::all(Px(4.0)),
                                                                        justify: MainAlign::Start,
                                                                        align: CrossAlign::Stretch,
                                                                        wrap: false,
                                                                    },
                                                                    |_cx| out,
                                                                );
                                                                listbox_id_out.set(Some(listbox_content.id));
                                                                vec![listbox_content]
                                            },
                                        );

                                        let active_descendant = active_element_id_out
                                            .get()
                                            .and_then(|id| cx.node_for_element(id));

                                        let inner = cx.container(
                                            ContainerProps {
                                                layout: {
                                                    let mut layout = LayoutStyle::default();
                                                    layout.size.width = Length::Fill;
                                                    layout.size.height = Length::Fill;
                                                    layout.overflow = Overflow::Clip;
                                                    layout
                                                },
                                                padding: Edges::all(Px(0.0)),
                                                background: Some(
                                                    theme_for_overlay.colors.panel_background,
                                                ),
                                                shadow: Some(shadow),
                                                border: Edges::all(border_width),
                                                border_color: Some(overlay_border),
                                                corner_radii: Corners::all(radius),
                                                ..Default::default()
                                            },
                                            move |_cx| vec![scroll],
                                        );

                                        (
                                            PressableProps {
                                                layout: popper_content::popper_panel_layout(
                                                    placed,
                                                    wrapper_insets,
                                                    Overflow::Clip,
                                                ),
                                                enabled: true,
                                                focusable: true,
                                                focus_ring: None,
                                                a11y: PressableA11y {
                                                    role: Some(SemanticsRole::ListBox),
                                                    active_descendant,
                                                    labelled_by_element: Some(trigger_id.0),
                                                    ..Default::default()
                                                },
                                                ..Default::default()
                                            },
                                            vec![inner],
                                        )
                                    },
                                );

                                if let Some(arrow_el) = arrow_el {
                                    vec![arrow_el, panel]
                                } else {
                                    vec![panel]
                                }
                            });

                        let animated =
                            overlay_motion::wrap_opacity_and_render_transform(cx, opacity, transform, vec![content]);

                        {
                            let mut state = trigger_state_for_overlay
                                .lock()
                                .unwrap_or_else(|e| e.into_inner());
                            state.viewport = viewport_id_out.get();
                            state.listbox = listbox_id_out.get();
                            state.content_panel = content_panel_id_out.get();
                            state.width_probe = width_probe_id_out.get();
                            state.selected_item = selected_item_id_out.get();
                            state.selected_item_text = selected_item_text_id_out.get();
                            state.alignment_item_pos = alignment_item_pos_out.get();
                            state.alignment_item_has_leading_non_item =
                                alignment_item_has_leading_non_item_out
                                    .get()
                                    .unwrap_or(false);
                            if !is_open {
                                state.did_item_aligned_scroll_initial = false;
                                state.did_item_aligned_scroll_reposition = false;
                                state.did_item_aligned_focus_scroll = false;
                                state.item_aligned_scroll_up_visible = false;
                                state.last_item_aligned_scroll_to_y = None;
                                state.item_aligned_user_scrolled = false;
                                state.pending_active_scroll_into_view = false;
                            }
                        }

                        radix_select::select_modal_layer_elements_with_pointer_up_guard_and_dismiss_handler(
                            cx,
                            open_for_barrier_children.clone(),
                            dismiss_on_overlay_press,
                            on_dismiss_request_for_overlay_children.clone(),
                            mouse_open_guard_for_barrier_children.clone(),
                            [probe],
                            animated,
                        )
                    });

                    let mut request = radix_select::modal_select_request_with_dismiss_handler(
                        trigger_id,
                        trigger_id,
                        open_for_trigger.clone(),
                        overlay_presence,
                        on_dismiss_request.clone(),
                        overlay_children,
                    );
                    request.initial_focus = radix_select::SelectInitialFocusTargets::new()
                        .pointer_content_focus(Some(listbox_id_for_trigger))
                        .keyboard_entry_focus(selected_item_id_for_request_cell.get())
                        .resolve(cx, cx.window);
                    radix_select::request_select(cx, request);
                } else {
                    let open_for_overlay = open_for_trigger.clone();
                    let mouse_open_guard_for_overlay = mouse_open_guard.clone();
                    let on_dismiss_request_for_overlay_children = on_dismiss_request.clone();
                    let overlay_children = cx.with_root_name(&overlay_root_name, move |cx| {
                        let pointer_up_guard = radix_select::select_modal_barrier_pointer_up_guard(
                            cx,
                            open_for_overlay.clone(),
                            mouse_open_guard_for_overlay.clone(),
                        );
                        vec![radix_select::select_modal_barrier_with_dismiss_handler(
                            cx,
                            open_for_overlay.clone(),
                            true,
                            on_dismiss_request_for_overlay_children.clone(),
                            [pointer_up_guard],
                        )]
                    });

                    let mut request = radix_select::modal_select_request_with_dismiss_handler(
                        trigger_id,
                        trigger_id,
                        open_for_trigger.clone(),
                        overlay_presence,
                        on_dismiss_request.clone(),
                        overlay_children,
                    );
                    request.initial_focus = radix_select::SelectInitialFocusTargets::new()
                        .pointer_content_focus(Some(listbox_id_for_trigger))
                        .keyboard_entry_focus(None)
                        .resolve(cx, cx.window);
                    radix_select::request_select(cx, request);
                }
            }

            let chrome_width = props.layout.size.width;
            let content_width = if matches!(chrome_width, Length::Auto) {
                Length::Auto
            } else {
                Length::Fill
            };
            let auto_width_trigger = matches!(content_width, Length::Auto);
            let mut chrome = input_chrome_container_props(
                {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = chrome_width;
                    layout
                },
                resolved,
                border_color,
            );
            if let Some(corners) = trigger_corner_radii_override {
                chrome.corner_radii = corners;
            }
            if let Some(border) = trigger_border_width_override.top {
                chrome.border.top = border;
            }
            if let Some(border) = trigger_border_width_override.right {
                chrome.border.right = border;
            }
            if let Some(border) = trigger_border_width_override.bottom {
                chrome.border.bottom = border;
            }
            if let Some(border) = trigger_border_width_override.left {
                chrome.border.left = border;
            }

            let state_for_value_node = trigger_state.clone();

            let content = move |cx: &mut ElementContext<'_, H>| {
                vec![cx.flex(
                    FlexProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = content_width;
                            layout
                        },
                        direction: fret_core::Axis::Horizontal,
                        gap: MetricRef::space(Space::N1p5).resolve(&theme),
                        padding: Edges::all(Px(0.0)),
                        justify: MainAlign::SpaceBetween,
                        align: CrossAlign::Center,
                        wrap: false,
                    },
                    |cx| {
                        vec![
                            {
                                let layout = {
                                    if auto_width_trigger {
                                        LayoutStyle::default()
                                    } else {
                                        let mut layout = LayoutStyle::default();
                                        layout.size.width = Length::Auto;
                                        layout.size.min_width = Some(Px(0.0));
                                        layout.flex.grow = 1.0;
                                        layout.flex.shrink = 1.0;
                                        layout.flex.basis = Length::Px(Px(0.0));
                                        layout
                                    }
                                };

                                let value_node = cx.container(
                                    ContainerProps {
                                        layout,
                                        ..Default::default()
                                    },
                                    move |cx| {
                                        let mut text = ui::text(cx, label)
                                            .text_size_px(text_style.size)
                                            .font_weight(text_style.weight)
                                            .text_color(ColorRef::Color(if selected.is_some() {
                                                fg
                                            } else {
                                                fg_muted
                                            }))
                                            .truncate();
                                        if let Some(line_height) = text_style.line_height {
                                            text = text.line_height_px(line_height);
                                        }
                                        if let Some(letter_spacing_em) = text_style.letter_spacing_em {
                                            text = text.letter_spacing_em(letter_spacing_em);
                                        }
                                        if !auto_width_trigger {
                                            text = text.w_full();
                                        }
                                        vec![text.into_element(cx)]
                                    },
                                );

                                let mut state = state_for_value_node
                                    .lock()
                                    .unwrap_or_else(|e| e.into_inner());
                                state.value_node = Some(value_node.id);

                                value_node
                            },
                                                        cx.opacity(0.5, |cx| {
                                vec![decl_icon::icon_with(
                                    cx,
                                    ids::ui::CHEVRON_DOWN,
                                    Some(Px(16.0)),
                                    Some(ColorRef::Color(fg_muted)),
                                )]
                            }),
                        ]
                    },
                )]
            };

            (props, chrome, content)
        });

        trigger
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    use fret_app::App;
    use fret_core::UiServices;
    use fret_core::{
        AppWindowId, Event, KeyCode, Modifiers, MouseButton, PathCommand, PathConstraints, PathId,
        PathMetrics, PointerEvent,
    };
    use fret_core::{PathService, PathStyle, Point, Px, Rect, SemanticsRole, Size};
    use fret_core::{SvgId, SvgService, TextBlobId, TextConstraints, TextMetrics, TextService};
    use fret_runtime::{Effect, FrameId};
    use fret_ui::tree::UiTree;

    #[test]
    fn select_align_default_is_start() {
        assert_eq!(SelectAlign::default(), SelectAlign::Start);
    }

    #[test]
    fn select_new_controllable_uses_controlled_models_when_provided() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );

        let controlled_value = app.models_mut().insert(Some(Arc::from("alpha")));
        let controlled_open = app.models_mut().insert(true);

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let select = Select::new_controllable(
                cx,
                Some(controlled_value.clone()),
                Some("beta"),
                Some(controlled_open.clone()),
                false,
            );
            assert_eq!(select.model, controlled_value);
            assert_eq!(select.open, controlled_open);
        });
    }

    #[test]
    fn select_new_controllable_applies_default_value_and_default_open() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let select = Select::new_controllable(cx, None, Some("alpha"), None, true);
            let value = cx.watch_model(&select.model).cloned().unwrap_or_default();
            let open = cx
                .watch_model(&select.open)
                .layout()
                .copied()
                .unwrap_or(false);

            assert_eq!(value.as_deref(), Some("alpha"));
            assert!(open);
        });
    }

    #[derive(Default)]
    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _input: &fret_core::TextInput,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: Size::new(Px(10.0), Px(10.0)),
                    baseline: Px(8.0),
                },
            )
        }

        fn release(&mut self, _blob: TextBlobId) {}
    }

    impl PathService for FakeServices {
        fn prepare(
            &mut self,
            _commands: &[PathCommand],
            _style: PathStyle,
            _constraints: PathConstraints,
        ) -> (PathId, PathMetrics) {
            (PathId::default(), PathMetrics::default())
        }

        fn release(&mut self, _path: PathId) {}
    }

    impl SvgService for FakeServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
            SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: SvgId) -> bool {
            true
        }
    }

    fn render_frame(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        model: Model<Option<Arc<str>>>,
        open: Model<bool>,
        items: Vec<SelectItem>,
    ) -> fret_core::NodeId {
        let next_frame = FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        fret_ui_kit::OverlayController::begin_frame(app, window);
        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "select", |cx| {
                vec![Select::new(model, open).items(items).into_element(cx)]
            });
        ui.set_root(root);
        fret_ui_kit::OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    fn render_frame_with_underlay(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        model: Model<Option<Arc<str>>>,
        open: Model<bool>,
        items: Vec<SelectItem>,
        underlay_activated: Model<bool>,
    ) -> fret_core::NodeId {
        let next_frame = FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        fret_ui_kit::OverlayController::begin_frame(app, window);
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "select-underlay",
            |cx| {
                let underlay_activated = underlay_activated.clone();
                let underlay = cx.pressable(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(120.0));
                            layout.size.height = Length::Px(Px(40.0));
                            layout.inset.top = Some(Px(180.0));
                            layout.inset.left = Some(Px(240.0));
                            layout.position = PositionStyle::Absolute;
                            layout
                        },
                        enabled: true,
                        focusable: true,
                        ..Default::default()
                    },
                    move |cx, _st| {
                        cx.pressable_set_bool(&underlay_activated, true);
                        Vec::new()
                    },
                );

                vec![
                    underlay,
                    Select::new(model.clone(), open.clone())
                        .items(items.clone())
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        fret_ui_kit::OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    fn render_frame_with_dismiss_handler(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        model: Model<Option<Arc<str>>>,
        open: Model<bool>,
        items: Vec<SelectItem>,
        on_dismiss_request: Option<OnDismissRequest>,
    ) -> fret_core::NodeId {
        let next_frame = FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        fret_ui_kit::OverlayController::begin_frame(app, window);
        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "select", |cx| {
                vec![
                    Select::new(model, open)
                        .items(items)
                        .on_dismiss_request(on_dismiss_request)
                        .into_element(cx),
                ]
            });
        ui.set_root(root);
        fret_ui_kit::OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    fn render_frame_with_arrow(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        model: Model<Option<Arc<str>>>,
        open: Model<bool>,
        items: Vec<SelectItem>,
        arrow: bool,
    ) -> fret_core::NodeId {
        let next_frame = FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        fret_ui_kit::OverlayController::begin_frame(app, window);
        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "select", |cx| {
                vec![
                    Select::new(model, open)
                        .items(items)
                        .arrow(arrow)
                        .into_element(cx),
                ]
            });
        ui.set_root(root);
        fret_ui_kit::OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    fn render_frame_entries(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        model: Model<Option<Arc<str>>>,
        open: Model<bool>,
        entries: Vec<SelectEntry>,
    ) -> fret_core::NodeId {
        let next_frame = FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        fret_ui_kit::OverlayController::begin_frame(app, window);
        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "select", |cx| {
                vec![Select::new(model, open).entries(entries).into_element(cx)]
            });
        ui.set_root(root);
        fret_ui_kit::OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    #[test]
    fn select_popover_items_have_collection_position_metadata() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(None::<Arc<str>>);
        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let items = vec![
            SelectItem::new("alpha", "Alpha"),
            SelectItem::new("beta", "Beta"),
            SelectItem::new("gamma", "Gamma"),
        ];

        // First frame: establish stable trigger bounds.
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items.clone(),
        );

        let _ = app.models_mut().update(&open, |v| *v = true);

        // Second frame: open the popover and verify item metadata.
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items,
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let beta = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ListBoxOption && n.label.as_deref() == Some("Beta"))
            .expect("Beta list item");
        assert_eq!(beta.pos_in_set, Some(2));
        assert_eq!(beta.set_size, Some(3));
    }

    #[test]
    fn select_trigger_enter_opens_on_key_down_and_does_not_toggle_closed_on_key_up() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(None::<Arc<str>>);
        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let items = vec![
            SelectItem::new("alpha", "Alpha"),
            SelectItem::new("beta", "Beta"),
        ];

        let root = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model,
            open.clone(),
            items,
        );

        let trigger = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("trigger node");
        ui.set_focus(Some(trigger));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::Enter,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        assert!(app.models().get_copied(&open).unwrap_or(false));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyUp {
                key: KeyCode::Enter,
                modifiers: Modifiers::default(),
            },
        );
        assert!(app.models().get_copied(&open).unwrap_or(false));
    }

    #[test]
    fn select_pointer_open_focuses_listbox_container() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(Some(Arc::from("beta")));
        let open = app.models_mut().insert(false);
        let underlay_activated = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let items = vec![
            SelectItem::new("alpha", "Alpha"),
            SelectItem::new("beta", "Beta"),
            SelectItem::new("gamma", "Gamma"),
        ];

        // Pointer open: focus listbox (content), not the selected item entry.
        let _root = render_frame_with_underlay(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items.clone(),
            underlay_activated.clone(),
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ComboBox)
            .expect("select trigger semantics");
        let trigger_bounds = ui
            .debug_node_visual_bounds(trigger.id)
            .expect("trigger bounds");
        let trigger_center = Point::new(
            Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 * 0.5),
            Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                position: trigger_center,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_id: fret_core::PointerId(0),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        assert!(
            app.models().get_copied(&open).unwrap_or(false),
            "expected pointer down to open select"
        );

        let _root = render_frame_with_underlay(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items.clone(),
            underlay_activated.clone(),
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let listbox = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ListBox)
            .expect("listbox node");
        let beta = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ListBoxOption && n.label.as_deref() == Some("Beta"))
            .expect("Beta list item");
        assert_eq!(
            ui.focus(),
            Some(listbox.id),
            "expected pointer-open to focus the listbox container"
        );
        assert_ne!(
            ui.focus(),
            Some(beta.id),
            "expected pointer-open to not focus the selected entry"
        );
    }

    #[test]
    fn select_keyboard_open_focuses_selected_entry() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(Some(Arc::from("beta")));
        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let items = vec![
            SelectItem::new("alpha", "Alpha"),
            SelectItem::new("beta", "Beta"),
            SelectItem::new("gamma", "Gamma"),
        ];

        let root = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items.clone(),
        );

        let trigger = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("trigger node");
        ui.set_focus(Some(trigger));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::Enter,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        assert!(
            app.models().get_copied(&open).unwrap_or(false),
            "expected keydown to open select"
        );

        let _root = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model,
            open,
            items,
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let beta = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ListBoxOption && n.label.as_deref() == Some("Beta"))
            .expect("Beta list item");
        assert_eq!(
            ui.focus(),
            Some(beta.id),
            "expected keyboard-open to focus the selected entry"
        );
    }

    #[test]
    fn select_trigger_typeahead_updates_selection_without_opening() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(None::<Arc<str>>);
        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let items = vec![
            SelectItem::new("alpha", "Alpha"),
            SelectItem::new("beta", "Beta"),
            SelectItem::new("gamma", "Gamma"),
        ];

        let root = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items,
        );

        let trigger = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("trigger node");
        ui.set_focus(Some(trigger));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::KeyB,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        assert!(!app.models().get_copied(&open).unwrap_or(false));
        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(selected.as_deref(), Some("beta"));

        let effects = app.flush_effects();
        let token = effects
            .iter()
            .find_map(|e| match e {
                Effect::SetTimer { token, after, .. }
                    if *after
                        == Duration::from_millis(
                            radix_select::SELECT_TYPEAHEAD_CLEAR_TIMEOUT_MS,
                        ) =>
                {
                    Some(*token)
                }
                _ => None,
            })
            .expect("typeahead clear timer token");

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::Space,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        assert!(!app.models().get_copied(&open).unwrap_or(false));

        ui.dispatch_event(&mut app, &mut services, &Event::Timer { token });

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::Space,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        assert!(app.models().get_copied(&open).unwrap_or(false));
    }

    #[test]
    fn select_label_and_separator_do_not_affect_positions_or_initial_focus() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(Some(Arc::from("beta")));
        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let entries = vec![
            SelectEntry::Label(SelectLabel::new("Fruits")),
            SelectEntry::Item(SelectItem::new("alpha", "Alpha")),
            SelectEntry::Separator(SelectSeparator),
            SelectEntry::Item(SelectItem::new("beta", "Beta")),
        ];

        let _ = render_frame_entries(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            entries.clone(),
        );

        let _ = app.models_mut().update(&open, |v| *v = true);

        let _ = render_frame_entries(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            entries.clone(),
        );
        // Third frame: allow `active_descendant` to resolve via last-frame node IDs.
        let _ = render_frame_entries(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            entries,
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let focus = snap.focus.expect("focus");
        let focused_node = snap
            .nodes
            .iter()
            .find(|n| n.id == focus)
            .expect("focused node");
        assert_eq!(focused_node.role, SemanticsRole::ListBox);

        let trigger = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ComboBox && n.flags.expanded)
            .expect("select trigger node");
        assert!(
            focused_node.labelled_by.iter().any(|id| *id == trigger.id),
            "listbox should be labelled by the trigger"
        );
        assert!(
            trigger.controls.iter().any(|id| *id == focused_node.id),
            "trigger should control the listbox"
        );

        let active = focused_node
            .active_descendant
            .expect("active_descendant should be set");
        let active_node = snap
            .nodes
            .iter()
            .find(|n| n.id == active)
            .expect("active_descendant should reference a node in the snapshot");

        assert_eq!(active_node.role, SemanticsRole::ListBoxOption);
        assert_eq!(active_node.label.as_deref(), Some("Beta"));
        assert_eq!(active_node.pos_in_set, Some(2));
        assert_eq!(active_node.set_size, Some(2));
    }

    #[test]
    fn select_open_installs_modal_barrier_root_for_a11y_isolation() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(None::<Arc<str>>);
        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let items = vec![
            SelectItem::new("alpha", "Alpha"),
            SelectItem::new("beta", "Beta"),
        ];

        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items.clone(),
        );

        let _ = app.models_mut().update(&open, |v| *v = true);
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model,
            open,
            items,
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let barrier_root = snap
            .barrier_root
            .expect("expected select to install a modal barrier root");
        assert!(
            snap.roots
                .iter()
                .any(|r| r.root == barrier_root && r.blocks_underlay_input),
            "expected barrier root to correspond to a blocks-underlay-input layer"
        );

        let listbox = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ListBox)
            .expect("listbox node");

        let mut parent_by_id: std::collections::HashMap<
            fret_core::NodeId,
            Option<fret_core::NodeId>,
        > = std::collections::HashMap::new();
        for n in snap.nodes.iter() {
            parent_by_id.insert(n.id, n.parent);
        }

        let mut root = listbox.id;
        while let Some(parent) = parent_by_id.get(&root).copied().flatten() {
            root = parent;
        }

        assert_eq!(
            root, barrier_root,
            "expected listbox to be rooted under the barrier layer"
        );
    }

    #[test]
    fn select_open_before_first_layout_installs_modal_barrier_and_blocks_underlay() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(None::<Arc<str>>);
        let open = app.models_mut().insert(true);
        let underlay_activated = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let items = vec![
            SelectItem::new("alpha", "Alpha"),
            SelectItem::new("beta", "Beta"),
        ];

        let next_frame = FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        fret_ui_kit::OverlayController::begin_frame(&mut app, window);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "select",
            |cx| {
                let underlay_activated = underlay_activated.clone();
                let underlay = cx.pressable(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(120.0));
                            layout.size.height = Length::Px(Px(40.0));
                            layout.inset.top = Some(Px(180.0));
                            layout.inset.left = Some(Px(240.0));
                            layout.position = PositionStyle::Absolute;
                            layout
                        },
                        enabled: true,
                        focusable: true,
                        ..Default::default()
                    },
                    move |cx, _st| {
                        cx.pressable_set_bool(&underlay_activated, true);
                        Vec::new()
                    },
                );

                vec![
                    underlay,
                    Select::new(model.clone(), open.clone())
                        .items(items.clone())
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        fret_ui_kit::OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(
            snap.barrier_root.is_some(),
            "expected select to install a modal barrier root"
        );

        let underlay_point = Point::new(Px(250.0), Px(190.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: underlay_point,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: underlay_point,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        assert_eq!(app.models().get_copied(&open), Some(false));
        assert_eq!(app.models().get_copied(&underlay_activated), Some(false));
    }

    #[test]
    fn select_close_transition_keeps_modal_barrier_blocking_underlay() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(None::<Arc<str>>);
        let open = app.models_mut().insert(false);
        let underlay_activated = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let items = vec![
            SelectItem::new("alpha", "Alpha"),
            SelectItem::new("beta", "Beta"),
        ];

        // Frame 1: closed.
        let _ = render_frame_with_underlay(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items.clone(),
            underlay_activated.clone(),
        );

        let _ = app.models_mut().update(&open, |v| *v = true);

        // Frame 2: open.
        let _ = render_frame_with_underlay(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items.clone(),
            underlay_activated.clone(),
        );
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(
            snap.barrier_root.is_some(),
            "expected select to install a modal barrier root"
        );

        let _ = app.models_mut().update(&open, |v| *v = false);

        // Frame 3: closing (present=true, interactive=false).
        let _ = render_frame_with_underlay(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items.clone(),
            underlay_activated.clone(),
        );
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let barrier_root = snap
            .barrier_root
            .expect("expected the barrier root to remain while the select is closing");
        let barrier_layer = ui.node_layer(barrier_root).expect("barrier layer");
        let barrier = ui
            .debug_layers_in_paint_order()
            .into_iter()
            .find(|l| l.id == barrier_layer)
            .expect("barrier debug layer info");
        assert!(barrier.visible);
        assert!(barrier.hit_testable);
        assert!(
            barrier.blocks_underlay_input,
            "expected modal barrier layer to block underlay input"
        );

        let underlay_point = Point::new(Px(250.0), Px(190.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: underlay_point,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: underlay_point,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        assert_eq!(app.models().get_copied(&underlay_activated), Some(false));

        // Once the exit transition settles, the barrier should drop and the underlay should be
        // interactive again.
        let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
        for _ in 0..settle_frames {
            let _ = render_frame_with_underlay(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                model.clone(),
                open.clone(),
                items.clone(),
                underlay_activated.clone(),
            );
        }

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(
            snap.barrier_root.is_none(),
            "expected the barrier root to be cleared once the exit transition completes"
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(1),
                position: underlay_point,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(1),
                position: underlay_point,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        assert_eq!(app.models().get_copied(&underlay_activated), Some(true));
    }

    #[test]
    fn select_mouse_drag_release_selects_item() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(Some(Arc::from("beta")));
        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let items = vec![
            SelectItem::new("alpha", "Alpha"),
            SelectItem::new("beta", "Beta"),
            SelectItem::new("gamma", "Gamma"),
        ];

        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items.clone(),
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ComboBox)
            .expect("select trigger node");
        let trigger_center = Point::new(
            Px(trigger.bounds.origin.x.0 + trigger.bounds.size.width.0 * 0.5),
            Px(trigger.bounds.origin.y.0 + trigger.bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: trigger_center,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        assert_eq!(app.models().get_copied(&open), Some(true));

        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items,
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let gamma = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ListBoxOption && n.label.as_deref() == Some("Gamma"))
            .expect("gamma option node");
        let gamma_center = Point::new(
            Px(gamma.bounds.origin.x.0 + gamma.bounds.size.width.0 * 0.5),
            Px(gamma.bounds.origin.y.0 + gamma.bounds.size.height.0 * 0.5),
        );

        let dx = (trigger_center.x.0 - gamma_center.x.0).abs();
        let dy = (trigger_center.y.0 - gamma_center.y.0).abs();
        assert!(
            dx > radix_select::SELECT_TRIGGER_CLICK_SLOP_PX
                || dy > radix_select::SELECT_TRIGGER_CLICK_SLOP_PX,
            "test expects a pointer-up delta larger than the Radix click slop"
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: gamma_center,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(selected.as_deref(), Some("gamma"));
        assert_eq!(app.models().get_copied(&open), Some(false));
    }

    #[test]
    fn select_mouse_drag_release_outside_closes_when_moved_beyond_slop() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(Some(Arc::from("beta")));
        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let items = vec![
            SelectItem::new("alpha", "Alpha"),
            SelectItem::new("beta", "Beta"),
            SelectItem::new("gamma", "Gamma"),
        ];

        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items.clone(),
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ComboBox)
            .expect("select trigger node");
        let trigger_center = Point::new(
            Px(trigger.bounds.origin.x.0 + trigger.bounds.size.width.0 * 0.5),
            Px(trigger.bounds.origin.y.0 + trigger.bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: trigger_center,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        assert_eq!(app.models().get_copied(&open), Some(true));

        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items,
        );

        let outside = Point::new(Px(390.0), Px(230.0));
        let dx = (trigger_center.x.0 - outside.x.0).abs();
        let dy = (trigger_center.y.0 - outside.y.0).abs();
        assert!(
            dx > radix_select::SELECT_TRIGGER_CLICK_SLOP_PX
                || dy > radix_select::SELECT_TRIGGER_CLICK_SLOP_PX,
            "test expects a pointer-up delta larger than the Radix click slop"
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: outside,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(selected.as_deref(), Some("beta"));
        assert_eq!(app.models().get_copied(&open), Some(false));
    }

    #[test]
    fn select_roving_navigation_does_not_commit_value_until_activation() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(Some(Arc::from("beta")));
        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let items = vec![
            SelectItem::new("alpha", "Alpha"),
            SelectItem::new("beta", "Beta"),
            SelectItem::new("gamma", "Gamma"),
        ];

        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items.clone(),
        );

        let _ = app.models_mut().update(&open, |v| *v = true);

        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items.clone(),
        );

        assert!(
            ui.focus().is_some(),
            "expected focus to move into the open select"
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::ArrowDown,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items,
        );

        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(selected.as_deref(), Some("beta"));
        assert!(app.models().get_copied(&open).unwrap_or(false));
    }

    #[test]
    fn select_arrow_is_hit_testable_and_does_not_dismiss_on_click() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(None::<Arc<str>>);
        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(800.0), Px(600.0)),
        );
        let mut services = FakeServices::default();

        let items = vec![
            SelectItem::new("alpha", "Alpha"),
            SelectItem::new("beta", "Beta"),
            SelectItem::new("gamma", "Gamma"),
        ];

        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items.clone(),
        );

        let _ = app.models_mut().update(&open, |v| *v = true);

        let _ = render_frame_with_arrow(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items,
            true,
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let list = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ListBox)
            .expect("list node");
        let list_bounds = ui.debug_node_visual_bounds(list.id).expect("list bounds");

        let click = Point::new(
            Px(list_bounds.origin.x.0 + list_bounds.size.width.0 * 0.5),
            Px(list_bounds.origin.y.0 - 1.0),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: click,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: click,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        assert_eq!(app.models().get_copied(&open), Some(true));
    }

    #[test]
    fn select_modal_barrier_dismiss_can_be_prevented_via_dismiss_handler() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(None::<Arc<str>>);
        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let items = vec![
            SelectItem::new("alpha", "Alpha"),
            SelectItem::new("beta", "Beta"),
            SelectItem::new("gamma", "Gamma"),
        ];

        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items.clone(),
        );

        let _ = app.models_mut().update(&open, |v| *v = true);

        let dismiss_calls = Arc::new(AtomicUsize::new(0));
        let dismiss_calls_for_handler = dismiss_calls.clone();
        let handler: OnDismissRequest = Arc::new(move |_host, _action_cx, req| {
            dismiss_calls_for_handler.fetch_add(1, Ordering::SeqCst);
            req.prevent_default();
        });

        let _ = render_frame_with_dismiss_handler(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model,
            open.clone(),
            items,
            Some(handler),
        );

        let outside = Point::new(Px(390.0), Px(230.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: outside,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: outside,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        assert!(dismiss_calls.load(Ordering::SeqCst) > 0);
        assert_eq!(app.models().get_copied(&open), Some(true));
    }

    #[test]
    fn select_scroll_buttons_scroll_without_dismissing() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(None::<Arc<str>>);
        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let items: Vec<SelectItem> = (0..50)
            .map(|i| SelectItem::new(format!("v{i}"), format!("Item {i}")))
            .collect();

        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items.clone(),
        );

        let _ = app.models_mut().update(&open, |v| *v = true);

        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items.clone(),
        );

        // Third frame: allow the scroll handle to observe content overflow.
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items.clone(),
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let scroll_down = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("select-scroll-down-button"))
            .expect("scroll down node");
        assert_eq!(scroll_down.role, SemanticsRole::Generic);

        let scroll_up = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("select-scroll-up-button"))
            .expect("scroll up node");
        assert_eq!(scroll_up.role, SemanticsRole::Generic);

        let down_bounds = ui
            .debug_node_bounds(scroll_down.id)
            .expect("scroll down bounds");
        let click = (|| {
            let candidates = [
                (0.5, 0.5),
                (0.25, 0.5),
                (0.75, 0.5),
                (0.5, 0.25),
                (0.5, 0.75),
            ];
            for (fx, fy) in candidates {
                let p = Point::new(
                    Px(down_bounds.origin.x.0 + down_bounds.size.width.0 * fx),
                    Px(down_bounds.origin.y.0 + down_bounds.size.height.0 * fy),
                );
                if let Some(hit) = ui.debug_hit_test(p).hit
                    && ui.debug_node_path(hit).contains(&scroll_down.id)
                {
                    return p;
                }
            }
            panic!("expected scroll down bounds to be hit-testable; bounds={down_bounds:?}");
        })();

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: click,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: click,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model,
            open.clone(),
            items,
        );

        assert_eq!(app.models().get_copied(&open), Some(true));
        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let scroll_up = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("select-scroll-up-button"))
            .expect("scroll up node");
        let up_bounds = ui
            .debug_node_bounds(scroll_up.id)
            .expect("scroll up bounds");
        let up_is_hit_testable = (|| {
            let candidates = [
                (0.5, 0.5),
                (0.25, 0.5),
                (0.75, 0.5),
                (0.5, 0.25),
                (0.5, 0.75),
            ];
            for (fx, fy) in candidates {
                let p = Point::new(
                    Px(up_bounds.origin.x.0 + up_bounds.size.width.0 * fx),
                    Px(up_bounds.origin.y.0 + up_bounds.size.height.0 * fy),
                );
                if let Some(hit) = ui.debug_hit_test(p).hit
                    && ui.debug_node_path(hit).contains(&scroll_up.id)
                {
                    return true;
                }
            }
            false
        })();
        assert!(
            up_is_hit_testable,
            "expected scroll up to become hit-testable after scrolling down; bounds={up_bounds:?}"
        );
    }

    #[test]
    fn select_wheel_scroll_clamps_to_last_item_without_blank_space() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(None::<Arc<str>>);
        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let items: Vec<SelectItem> = (0..60)
            .map(|i| SelectItem::new(format!("v{i}"), format!("Item {i}")))
            .collect();

        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items.clone(),
        );

        let _ = app.models_mut().update(&open, |v| *v = true);

        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items.clone(),
        );

        // Third frame: allow the scroll handle to observe content overflow.
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items.clone(),
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let list = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ListBox)
            .expect("list node");
        let list_bounds = ui.debug_node_bounds(list.id).expect("list bounds");
        let wheel_pos = Point::new(
            Px(list_bounds.origin.x.0 + list_bounds.size.width.0 * 0.5),
            Px(list_bounds.origin.y.0 + list_bounds.size.height.0 * 0.5),
        );

        // Simulate repeated wheel scrolling (large delta) and ensure we clamp to the bottom.
        for _ in 0..40 {
            ui.dispatch_event(
                &mut app,
                &mut services,
                &fret_core::Event::Pointer(fret_core::PointerEvent::Wheel {
                    pointer_id: fret_core::PointerId(0),
                    position: wheel_pos,
                    delta: fret_core::Point::new(Px(0.0), Px(-80.0)),
                    modifiers: Modifiers::default(),
                    pointer_type: fret_core::PointerType::Mouse,
                }),
            );
        }

        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model,
            open,
            items,
        );

        let snap = ui
            .semantics_snapshot()
            .expect("semantics snapshot after wheel");
        let list = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ListBox)
            .expect("list node after wheel");
        let last = snap
            .nodes
            .iter()
            .find(|n| {
                n.role == SemanticsRole::ListBoxOption && n.label.as_deref() == Some("Item 59")
            })
            .expect("last item node after wheel");

        let list_bounds = ui
            .debug_node_visual_bounds(list.id)
            .expect("list bounds after wheel");
        let last_bounds = ui
            .debug_node_visual_bounds(last.id)
            .expect("last item bounds after wheel");
        let list_top = list_bounds.origin.y.0;
        let list_bottom = list_bounds.origin.y.0 + list_bounds.size.height.0;
        let last_top = last_bounds.origin.y.0;
        let last_bottom = last_bounds.origin.y.0 + last_bounds.size.height.0;

        assert!(
            last_bottom > list_top + 0.01 && last_top < list_bottom - 0.01,
            "expected last item to remain visible after wheel scrolling; list={list_bounds:?} last={last_bounds:?}"
        );
    }

    #[test]
    fn select_item_aligned_overlay_does_not_drift_after_wheel_scroll() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(None::<Arc<str>>);
        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let items: Vec<SelectItem> = (0..60)
            .map(|i| SelectItem::new(format!("v{i}"), format!("Item {i}")))
            .collect();

        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items.clone(),
        );

        let _ = app.models_mut().update(&open, |v| *v = true);

        // Render a couple frames so scroll geometry is fully realized (matches other select tests).
        for _ in 0..2 {
            let _ = render_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                model.clone(),
                open.clone(),
                items.clone(),
            );
        }

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let list = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::ListBox)
            .expect("list node");
        let viewport = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("select-scroll-viewport"))
            .expect("select viewport node");

        let list_bounds = ui.debug_node_bounds(list.id).expect("list bounds");
        let viewport_bounds_before = ui.debug_node_bounds(viewport.id).expect("viewport bounds");
        assert!(
            viewport_bounds_before.size.height.0 > 1.0,
            "expected non-zero viewport height before wheel; bounds={viewport_bounds_before:?}"
        );

        let wheel_pos = Point::new(
            Px(list_bounds.origin.x.0 + list_bounds.size.width.0 * 0.5),
            Px(list_bounds.origin.y.0 + list_bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Wheel {
                pointer_id: fret_core::PointerId(0),
                position: wheel_pos,
                delta: fret_core::Point::new(Px(0.0), Px(-120.0)),
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            open.clone(),
            items.clone(),
        );

        let snap = ui
            .semantics_snapshot()
            .expect("semantics snapshot after wheel");
        let viewport = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("select-scroll-viewport"))
            .expect("select viewport node after wheel");
        let viewport_bounds_after = ui
            .debug_node_bounds(viewport.id)
            .expect("viewport bounds after wheel");

        for i in 0..8 {
            let _ = render_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                model.clone(),
                open.clone(),
                items.clone(),
            );

            let snap = ui.semantics_snapshot().expect("semantics snapshot");
            let viewport = snap
                .nodes
                .iter()
                .find(|n| n.test_id.as_deref() == Some("select-scroll-viewport"))
                .expect("select viewport node");
            let viewport_bounds = ui.debug_node_bounds(viewport.id).expect("viewport bounds");

            let drift = (viewport_bounds.origin.y.0 - viewport_bounds_after.origin.y.0).abs();
            assert!(
                drift <= 1.0,
                "expected select viewport not to drift across frames after wheel (frame={i}); drift={drift} bounds={viewport_bounds:?} base={viewport_bounds_after:?}"
            );
            assert!(
                viewport_bounds.size.height.0 > 1.0,
                "expected non-zero viewport height after wheel (frame={i}); bounds={viewport_bounds:?}"
            );
        }
    }

    #[test]
    fn select_list_desired_height_clamps_to_tight_max_height() {
        let item_h = Px(32.0);
        let outer_h = Px(600.0);
        let max_h = Px(12.0);

        let desired = super::select_list_desired_height(item_h, 20, max_h, outer_h);
        assert_eq!(desired, max_h);
    }
}
