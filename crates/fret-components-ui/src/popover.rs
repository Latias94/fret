use fret_core::{
    Color, Corners, DrawOrder, Edges, Event, KeyCode, Modifiers, MouseButton, NodeId, Point, Px,
    Rect, SceneOp, SemanticsRole, Size, TextConstraints, TextMetrics, TextOverflow, TextStyle,
    TextWrap,
};
use fret_runtime::{CommandId, Effect};
use fret_ui::{
    Theme, UiHost,
    widget::{EventCx, Invalidation, LayoutCx, PaintCx, SemanticsCx, Widget},
};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};
use std::{collections::HashMap, sync::Arc};

use crate::ChromeRefinement;
use crate::Size as ComponentSize;
use crate::recipes::menu_list::resolve_menu_list_row_chrome;
use crate::recipes::surface::{SurfaceTokenKeys, resolve_surface_chrome};
use crate::{DismissOnEscapeAndClickOutside, EscapeDismissModifiers};
use fret_ui::overlay_placement;

pub(crate) const POPOVER_A11Y_SLOTS: usize = 256;

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct PopoverA11yState {
    slots: Vec<Option<PopoverA11ySlot>>,
}

#[allow(dead_code)]
impl PopoverA11yState {
    pub(crate) fn new(slot_count: usize) -> Self {
        Self {
            slots: vec![None; slot_count],
        }
    }

    fn clear(&mut self) {
        for slot in &mut self.slots {
            *slot = None;
        }
    }

    fn slot(&self, index: usize) -> Option<&PopoverA11ySlot> {
        self.slots.get(index).and_then(|s| s.as_ref())
    }

    fn set_slot(&mut self, index: usize, value: Option<PopoverA11ySlot>) {
        if let Some(dst) = self.slots.get_mut(index) {
            *dst = value;
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct PopoverA11ySlot {
    index: usize,
    bounds: Rect,
    label: Arc<str>,
    enabled: bool,
    selected: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct PopoverA11yItem {
    slot: usize,
    shared: Rc<RefCell<PopoverA11yState>>,
}

#[allow(dead_code)]
impl PopoverA11yItem {
    pub(crate) fn new(slot: usize, shared: Rc<RefCell<PopoverA11yState>>) -> Self {
        Self { slot, shared }
    }
}

#[allow(dead_code)]
impl<H: UiHost> Widget<H> for PopoverA11yItem {
    fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
        false
    }

    fn is_focusable(&self) -> bool {
        let state = self.shared.borrow();
        let Some(slot) = state.slot(self.slot) else {
            return false;
        };
        slot.enabled
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.available
    }

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        let state = self.shared.borrow();
        let Some(slot) = state.slot(self.slot) else {
            cx.set_role(SemanticsRole::Generic);
            cx.set_disabled(true);
            cx.set_focusable(false);
            cx.set_invokable(false);
            return;
        };

        cx.set_role(SemanticsRole::ListItem);
        cx.set_label(slot.label.as_ref().to_string());
        cx.set_disabled(!slot.enabled);
        cx.set_focusable(slot.enabled);
        cx.set_invokable(slot.enabled);
        cx.set_selected(slot.selected);
    }

    fn paint(&mut self, _cx: &mut PaintCx<'_, H>) {}
}

#[derive(Debug, Clone)]
pub struct PopoverItem {
    pub label: Arc<str>,
    pub enabled: bool,
}

impl PopoverItem {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            enabled: true,
        }
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

#[derive(Debug, Clone)]
pub struct PopoverRequest {
    pub owner: NodeId,
    pub anchor: Rect,
    pub items: Vec<PopoverItem>,
    pub selected: Option<usize>,
    /// Whether `WindowOverlays` should move focus to the popover node when opening.
    ///
    /// Default: `true` (menu-like behavior).
    ///
    /// Some interactions (e.g. typeahead comboboxes) may want to keep focus in the text input
    /// while showing an anchored list.
    pub request_focus: bool,
}

#[derive(Debug, Default)]
pub struct PopoverService {
    next_serial: u64,
    by_window: HashMap<fret_core::AppWindowId, PopoverEntry>,
    results: HashMap<(fret_core::AppWindowId, NodeId), usize>,
}

#[derive(Debug)]
struct PopoverEntry {
    serial: u64,
    request: PopoverRequest,
}

impl PopoverService {
    pub fn set_request(&mut self, window: fret_core::AppWindowId, request: PopoverRequest) {
        self.next_serial = self.next_serial.saturating_add(1);
        let serial = self.next_serial;
        self.by_window
            .insert(window, PopoverEntry { serial, request });
    }

    pub fn request(&self, window: fret_core::AppWindowId) -> Option<(u64, &PopoverRequest)> {
        let entry = self.by_window.get(&window)?;
        Some((entry.serial, &entry.request))
    }

    pub fn clear_request(&mut self, window: fret_core::AppWindowId) {
        self.by_window.remove(&window);
    }

    pub fn set_result(&mut self, window: fret_core::AppWindowId, owner: NodeId, selected: usize) {
        self.results.insert((window, owner), selected);
    }

    pub fn take_result(&mut self, window: fret_core::AppWindowId, owner: NodeId) -> Option<usize> {
        self.results.remove(&(window, owner))
    }
}

#[derive(Debug, Clone)]
pub struct PopoverStyle {
    pub background: Color,
    pub shadow: Option<fret_ui::element::ShadowStyle>,
    pub border: Edges,
    pub border_color: Color,
    pub corner_radii: Corners,
    /// Insets the available window bounds so floating shadows don't get clipped at the edges.
    pub window_margin: Px,
    pub row_hover: Color,
    pub row_selected: Color,
    pub text_color: Color,
    pub disabled_text_color: Color,
    pub text_style: TextStyle,
    pub padding_x: Px,
    pub padding_y: Px,
    pub row_height: Px,
    pub gap: Px,
}

impl Default for PopoverStyle {
    fn default() -> Self {
        Self {
            background: Color {
                r: 0.10,
                g: 0.10,
                b: 0.12,
                a: 1.0,
            },
            shadow: None,
            border: Edges::all(Px(1.0)),
            border_color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.40,
            },
            corner_radii: Corners::all(Px(8.0)),
            window_margin: Px(8.0),
            row_hover: Color {
                r: 0.16,
                g: 0.17,
                b: 0.22,
                a: 0.95,
            },
            row_selected: Color {
                r: 0.24,
                g: 0.34,
                b: 0.52,
                a: 0.65,
            },
            text_color: Color {
                r: 0.92,
                g: 0.92,
                b: 0.92,
                a: 1.0,
            },
            disabled_text_color: Color {
                r: 0.92,
                g: 0.92,
                b: 0.92,
                a: 0.45,
            },
            text_style: TextStyle {
                font: fret_core::FontId::default(),
                size: Px(13.0),
                ..Default::default()
            },
            padding_x: Px(10.0),
            padding_y: Px(8.0),
            row_height: Px(22.0),
            gap: Px(2.0),
        }
    }
}

#[derive(Debug)]
struct PreparedRow {
    label: fret_core::TextBlobId,
    metrics: TextMetrics,
    enabled: bool,
    bounds: Rect,
}

#[derive(Debug)]
pub struct Popover {
    style: PopoverStyle,
    style_override: bool,
    size: ComponentSize,
    #[allow(dead_code)]
    a11y: Rc<RefCell<PopoverA11yState>>,
    rows_dirty: bool,
    last_bounds: Rect,
    last_serial: Option<u64>,
    last_theme_revision: Option<u64>,
    #[allow(dead_code)]
    last_scale_factor_bits: Option<u32>,
    hover_row: Option<usize>,
    rows: Vec<PreparedRow>,
    panel_bounds: Rect,
    scroll_offset_y: Px,
    max_scroll_offset_y: Px,
    typeahead: String,
    typeahead_last: Option<Instant>,
    dismiss: DismissOnEscapeAndClickOutside,
}

#[allow(dead_code)]
struct PopoverPrepareCx<'a, H: UiHost> {
    app: &'a mut H,
    services: &'a mut dyn fret_core::UiServices,
    window: Option<fret_core::AppWindowId>,
    bounds: Rect,
    scale_factor: f32,
}

impl Popover {
    pub fn new() -> Self {
        Self::new_with_a11y(Rc::new(RefCell::new(PopoverA11yState::new(
            POPOVER_A11Y_SLOTS,
        ))))
    }

    pub(crate) fn new_with_a11y(a11y: Rc<RefCell<PopoverA11yState>>) -> Self {
        Self {
            style: PopoverStyle::default(),
            style_override: false,
            size: ComponentSize::Small,
            a11y,
            rows_dirty: true,
            last_bounds: Rect::default(),
            last_serial: None,
            last_theme_revision: None,
            last_scale_factor_bits: None,
            hover_row: None,
            rows: Vec::new(),
            panel_bounds: Rect::default(),
            scroll_offset_y: Px(0.0),
            max_scroll_offset_y: Px(0.0),
            typeahead: String::new(),
            typeahead_last: None,
            dismiss: DismissOnEscapeAndClickOutside::new(CommandId::from("popover.close"))
                .escape_modifiers(EscapeDismissModifiers::Any),
        }
    }

    pub fn with_style(mut self, style: PopoverStyle) -> Self {
        self.style = style;
        self.style_override = true;
        self
    }

    pub fn with_size(mut self, size: ComponentSize) -> Self {
        self.size = size;
        self.last_theme_revision = None;
        self
    }

    fn sync_style_from_theme(&mut self, theme: &Theme) {
        if self.style_override {
            return;
        }
        if self.last_theme_revision == Some(theme.revision()) {
            return;
        }
        self.last_theme_revision = Some(theme.revision());
        self.rows_dirty = true;

        let surface = resolve_surface_chrome(
            theme,
            &ChromeRefinement::default(),
            SurfaceTokenKeys {
                padding_x: None,
                padding_y: None,
                radius: Some("metric.radius.md"),
                border_width: None,
                bg: Some("popover.background"),
                border: Some("border"),
            },
        );
        self.style.border = Edges::all(surface.border_width);
        self.style.background = surface.background;
        self.style.border_color = surface.border_color;
        self.style.corner_radii = Corners::all(surface.radius);
        self.style.shadow = Some(crate::declarative::style::shadow_md(theme, surface.radius));

        self.style.window_margin = theme
            .metric_by_key("component.popover.window_margin")
            .or_else(|| theme.metric_by_key("component.popover_surface.window_margin"))
            .unwrap_or(Px(8.0));

        let rows = resolve_menu_list_row_chrome(theme, self.size);
        self.style.padding_x = rows.padding_x;
        self.style.padding_y = rows.padding_y;
        self.style.row_hover = rows.row_hover;
        self.style.row_selected = rows.row_selected;
        self.style.text_color = rows.text_color;
        self.style.disabled_text_color = rows.disabled_text_color;
        self.style.text_style = rows.text_style;
        self.style.row_height = rows.row_height;
    }

    fn cleanup(&mut self, services: &mut dyn fret_core::UiServices) {
        for row in self.rows.drain(..) {
            services.text().release(row.label);
        }
    }

    fn clear_a11y(&mut self) {
        self.a11y.borrow_mut().clear();
    }

    fn update_a11y_from_rows(&mut self, request: &PopoverRequest) {
        let mut state = self.a11y.borrow_mut();
        state.clear();

        let active = self.hover_row.or(request.selected);
        for slot in 0..state.slots.len() {
            let Some(item) = request.items.get(slot) else {
                break;
            };

            let bounds = self
                .rows
                .get(slot)
                .map(|r| r.bounds)
                .unwrap_or(self.panel_bounds);
            state.set_slot(
                slot,
                Some(PopoverA11ySlot {
                    index: slot,
                    bounds,
                    label: item.label.clone(),
                    enabled: item.enabled,
                    selected: active == Some(slot),
                }),
            );
        }
    }

    fn update_a11y_flags_only(&mut self, request: &PopoverRequest) {
        let mut state = self.a11y.borrow_mut();
        let active = self.hover_row.or(request.selected);
        for slot in &mut state.slots {
            let Some(s) = slot.as_mut() else {
                continue;
            };
            s.selected = active == Some(s.index);
        }
    }

    fn focused_slot_index(&self, focus: Option<NodeId>, children: &[NodeId]) -> Option<usize> {
        let focus = focus?;
        let idx = children.iter().position(|&c| c == focus)?;
        self.a11y.borrow().slot(idx).map(|s| s.index)
    }

    fn relayout_rows(&mut self) {
        let mut row_y =
            self.panel_bounds.origin.y.0 + self.style.padding_y.0 - self.scroll_offset_y.0;
        for row in &mut self.rows {
            row.bounds = Rect::new(
                Point::new(self.panel_bounds.origin.x, Px(row_y)),
                Size::new(self.panel_bounds.size.width, self.style.row_height),
            );
            row_y += self.style.row_height.0;
        }
    }

    fn ensure_row_visible(&mut self, index: usize) {
        let viewport_h =
            Px((self.panel_bounds.size.height.0 - self.style.padding_y.0 * 2.0).max(0.0));
        if viewport_h.0 <= 0.0 {
            return;
        }

        let row_h = self.style.row_height.0.max(0.0);
        let top = (index as f32) * row_h;
        let bottom = top + row_h;

        let view_top = self.scroll_offset_y.0;
        let view_bottom = view_top + viewport_h.0;

        if top < view_top {
            self.scroll_offset_y = Px(top);
        } else if bottom > view_bottom {
            self.scroll_offset_y = Px((bottom - viewport_h.0).max(0.0));
        }

        self.scroll_offset_y = Px(self
            .scroll_offset_y
            .0
            .max(0.0)
            .min(self.max_scroll_offset_y.0.max(0.0)));
        self.relayout_rows();
    }

    fn ensure_prepared<H: UiHost>(
        &mut self,
        cx: &mut PopoverPrepareCx<'_, H>,
    ) -> Option<fret_core::AppWindowId> {
        if self.last_bounds != cx.bounds {
            self.rows_dirty = true;
        }
        self.last_bounds = cx.bounds;

        let scale_bits = cx.scale_factor.to_bits();
        if self.last_scale_factor_bits != Some(scale_bits) {
            self.last_scale_factor_bits = Some(scale_bits);
            self.rows_dirty = true;
        }

        let window = cx.window?;
        self.sync_style_from_theme(Theme::global(&*cx.app));

        let Some(service) = cx.app.global::<PopoverService>() else {
            self.cleanup(cx.services);
            self.last_serial = None;
            self.panel_bounds = Rect::default();
            self.hover_row = None;
            self.clear_a11y();
            return Some(window);
        };
        let Some((serial, request)) = service.request(window).map(|(s, r)| (s, r.clone())) else {
            self.cleanup(cx.services);
            self.last_serial = None;
            self.panel_bounds = Rect::default();
            self.hover_row = None;
            self.clear_a11y();
            return Some(window);
        };

        if self.last_serial != Some(serial) {
            self.cleanup(cx.services);
            self.last_serial = Some(serial);
            self.hover_row = None;
            self.panel_bounds = Rect::default();
            self.scroll_offset_y = Px(0.0);
            self.max_scroll_offset_y = Px(0.0);
            self.clear_typeahead();
            self.rows_dirty = true;
        }

        if self.rows_dirty || self.rows.is_empty() {
            self.rows_dirty = false;
            self.cleanup(cx.services);
            self.rebuild_rows(cx, &request);
            self.hover_row = request
                .selected
                .filter(|&i| self.rows.get(i).is_some_and(|r| r.enabled));
            if let Some(i) = self.hover_row {
                self.ensure_row_visible(i);
            }
            self.clear_typeahead();
        }

        Some(window)
    }

    fn hit_test_row(&self, point: Point) -> Option<usize> {
        if !self.panel_bounds.contains(point) {
            return None;
        }
        for (i, row) in self.rows.iter().enumerate() {
            if row.bounds.contains(point) {
                return Some(i);
            }
        }
        None
    }

    fn close_popover<H: UiHost>(&mut self, cx: &mut EventCx<'_, H>) {
        self.cleanup(cx.services);
        cx.dispatch_command(CommandId::from("popover.close"));
        cx.stop_propagation();
    }

    fn activate_row<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        window: fret_core::AppWindowId,
        request: &PopoverRequest,
        index: usize,
    ) {
        let Some(row) = self.rows.get(index) else {
            self.close_popover(cx);
            return;
        };
        if !row.enabled {
            return;
        }

        cx.app
            .with_global_mut(PopoverService::default, |service, _app| {
                service.set_result(window, request.owner, index);
            });
        cx.app.push_effect(Effect::UiInvalidateLayout { window });
        self.cleanup(cx.services);
        cx.dispatch_command(CommandId::from("popover.close"));
        cx.stop_propagation();
    }

    fn rebuild_rows<H: UiHost>(
        &mut self,
        cx: &mut PopoverPrepareCx<'_, H>,
        request: &PopoverRequest,
    ) {
        let text_constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor,
        };

        // Measure widths.
        let mut max_w = Px(120.0);
        let mut prepared: Vec<(fret_core::TextBlobId, TextMetrics, bool)> = Vec::new();
        for item in &request.items {
            let (blob, metrics) = cx.services.text().prepare(
                item.label.as_ref(),
                self.style.text_style,
                text_constraints,
            );
            max_w = Px(max_w.0.max(metrics.size.width.0));
            prepared.push((blob, metrics, item.enabled));
        }

        let panel_w = Px(max_w.0 + self.style.padding_x.0 * 2.0);
        let content_h = Px((request.items.len() as f32) * self.style.row_height.0);
        let panel_h = Px(content_h.0 + self.style.padding_y.0 * 2.0);

        let outer =
            overlay_placement::inset_rect(cx.bounds, Edges::all(Px(self.style.window_margin.0)));
        self.panel_bounds = overlay_placement::anchored_panel_bounds_sized(
            outer,
            request.anchor,
            Size::new(panel_w, panel_h),
            self.style.gap,
            overlay_placement::Side::Bottom,
            overlay_placement::Align::Start,
        );

        let viewport_h =
            Px((self.panel_bounds.size.height.0 - self.style.padding_y.0 * 2.0).max(0.0));
        self.max_scroll_offset_y = Px((content_h.0 - viewport_h.0).max(0.0));
        self.scroll_offset_y = Px(self
            .scroll_offset_y
            .0
            .max(0.0)
            .min(self.max_scroll_offset_y.0));

        // Place rows.
        let mut row_y =
            self.panel_bounds.origin.y.0 + self.style.padding_y.0 - self.scroll_offset_y.0;
        self.rows.clear();
        for (blob, metrics, enabled) in prepared {
            let bounds = Rect::new(
                Point::new(self.panel_bounds.origin.x, Px(row_y)),
                Size::new(self.panel_bounds.size.width, self.style.row_height),
            );
            row_y += self.style.row_height.0;
            self.rows.push(PreparedRow {
                label: blob,
                metrics,
                enabled,
                bounds,
            });
        }
    }

    fn first_enabled_row(&self) -> Option<usize> {
        self.rows.iter().position(|r| r.enabled)
    }

    fn last_enabled_row(&self) -> Option<usize> {
        self.rows.iter().rposition(|r| r.enabled)
    }

    fn next_enabled_row(&self, start: usize, dir: i32) -> Option<usize> {
        let len = self.rows.len();
        if len == 0 {
            return None;
        }
        for step in 1..=len {
            let idx = if dir >= 0 {
                (start + step) % len
            } else {
                (start + len - (step % len)) % len
            };
            if self.rows.get(idx).is_some_and(|r| r.enabled) {
                return Some(idx);
            }
        }
        None
    }

    fn page_step(&self, viewport_bounds: Rect) -> usize {
        let view_h = (viewport_bounds.size.height.0 - self.style.padding_y.0 * 2.0).max(0.0);
        let row_h = self.style.row_height.0.max(1.0);
        let page = (view_h / row_h).floor() as usize;
        page.max(1)
    }

    fn page_step_enabled(&self, viewport_bounds: Rect, start: usize, dir: i32) -> Option<usize> {
        let len = self.rows.len();
        if len == 0 {
            return None;
        }
        let step = self.page_step(viewport_bounds);
        let unclamped = if dir >= 0 {
            start.saturating_add(step)
        } else {
            start.saturating_sub(step)
        };
        let mut idx = unclamped.min(len.saturating_sub(1));

        if self.rows.get(idx).is_some_and(|r| r.enabled) {
            return Some(idx);
        }

        if dir >= 0 {
            while idx + 1 < len {
                idx += 1;
                if self.rows.get(idx).is_some_and(|r| r.enabled) {
                    return Some(idx);
                }
            }
        } else {
            while idx > 0 {
                idx -= 1;
                if self.rows.get(idx).is_some_and(|r| r.enabled) {
                    return Some(idx);
                }
            }
        }
        None
    }

    fn typeahead_char(key: KeyCode, modifiers: &Modifiers) -> Option<char> {
        if modifiers.ctrl || modifiers.meta || modifiers.alt || modifiers.alt_gr {
            return None;
        }
        Some(match key {
            KeyCode::KeyA => 'a',
            KeyCode::KeyB => 'b',
            KeyCode::KeyC => 'c',
            KeyCode::KeyD => 'd',
            KeyCode::KeyE => 'e',
            KeyCode::KeyF => 'f',
            KeyCode::KeyG => 'g',
            KeyCode::KeyH => 'h',
            KeyCode::KeyI => 'i',
            KeyCode::KeyJ => 'j',
            KeyCode::KeyK => 'k',
            KeyCode::KeyL => 'l',
            KeyCode::KeyM => 'm',
            KeyCode::KeyN => 'n',
            KeyCode::KeyO => 'o',
            KeyCode::KeyP => 'p',
            KeyCode::KeyQ => 'q',
            KeyCode::KeyR => 'r',
            KeyCode::KeyS => 's',
            KeyCode::KeyT => 't',
            KeyCode::KeyU => 'u',
            KeyCode::KeyV => 'v',
            KeyCode::KeyW => 'w',
            KeyCode::KeyX => 'x',
            KeyCode::KeyY => 'y',
            KeyCode::KeyZ => 'z',
            KeyCode::Digit0 | KeyCode::Numpad0 => '0',
            KeyCode::Digit1 | KeyCode::Numpad1 => '1',
            KeyCode::Digit2 | KeyCode::Numpad2 => '2',
            KeyCode::Digit3 | KeyCode::Numpad3 => '3',
            KeyCode::Digit4 | KeyCode::Numpad4 => '4',
            KeyCode::Digit5 | KeyCode::Numpad5 => '5',
            KeyCode::Digit6 | KeyCode::Numpad6 => '6',
            KeyCode::Digit7 | KeyCode::Numpad7 => '7',
            KeyCode::Digit8 | KeyCode::Numpad8 => '8',
            KeyCode::Digit9 | KeyCode::Numpad9 => '9',
            _ => return None,
        })
    }

    fn typeahead_timeout() -> Duration {
        Duration::from_millis(1000)
    }

    fn clear_typeahead(&mut self) {
        self.typeahead.clear();
        self.typeahead_last = None;
    }

    fn handle_typeahead(&mut self, request: &PopoverRequest, typed: char) -> bool {
        let now = Instant::now();
        if self
            .typeahead_last
            .is_some_and(|t| now.duration_since(t) > Self::typeahead_timeout())
        {
            self.typeahead.clear();
        }
        self.typeahead_last = Some(now);

        let lower = typed.to_ascii_lowercase();
        let cycle_same = self.typeahead.chars().count() == 1 && self.typeahead.starts_with(lower);

        if self.typeahead.is_empty() || !cycle_same {
            self.typeahead.push(lower);
        }

        let query = self.typeahead.as_str();
        let start = self
            .hover_row
            .or(request.selected)
            .map(|i| i.saturating_add(1))
            .unwrap_or(0);

        let find_from = |from: usize| {
            (from..request.items.len()).chain(0..from).find(|&i| {
                request.items.get(i).is_some_and(|it| {
                    it.enabled && it.label.to_ascii_lowercase().starts_with(query)
                })
            })
        };

        if let Some(i) = find_from(start) {
            self.hover_row = Some(i);
            true
        } else if cycle_same && query.len() == 1 {
            // Keep cycling behavior stable if there were no matches for the repeated character.
            false
        } else {
            false
        }
    }
}

impl Default for Popover {
    fn default() -> Self {
        Self::new()
    }
}

impl<H: UiHost> Widget<H> for Popover {
    fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        self.cleanup(services);
        self.last_serial = None;
        self.last_scale_factor_bits = None;
        self.hover_row = None;
        self.panel_bounds = Rect::default();
        self.scroll_offset_y = Px(0.0);
        self.max_scroll_offset_y = Px(0.0);
        self.rows_dirty = true;
        self.clear_a11y();
    }

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        cx.set_role(SemanticsRole::List);
        cx.set_label("Popover");
        cx.set_focusable(true);
        cx.set_invokable(false);

        if let Some(window) = cx.window
            && let Some((_, request)) = cx
                .app
                .global::<PopoverService>()
                .and_then(|s| s.request(window))
            && let Some(active) = self.hover_row.or(request.selected)
            && let Some(item) = request.items.get(active)
        {
            cx.set_value(item.label.as_ref().to_string());
        }
    }

    fn hit_test(&self, _bounds: Rect, position: Point) -> bool {
        self.panel_bounds.contains(position)
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        let Some(window) = cx.window else {
            return;
        };

        self.sync_style_from_theme(cx.theme());

        let Some((_, request)) = cx
            .app
            .global::<PopoverService>()
            .and_then(|s| s.request(window))
            .map(|(serial, request)| (serial, request.clone()))
        else {
            return;
        };

        if let Some(slot) = self.focused_slot_index(cx.focus, cx.children)
            && request.items.get(slot).is_some()
        {
            if self.hover_row != Some(slot) {
                self.hover_row = Some(slot);
                self.clear_typeahead();
            }
            self.update_a11y_flags_only(&request);
        }

        match event {
            Event::Pointer(fret_core::PointerEvent::Move { position, .. }) => {
                let hovered = self.hit_test_row(*position);
                if hovered != self.hover_row {
                    self.hover_row = hovered;
                    self.clear_typeahead();
                    self.update_a11y_flags_only(&request);
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                }
            }
            Event::Pointer(fret_core::PointerEvent::Wheel {
                position, delta, ..
            }) => {
                if !self.panel_bounds.contains(*position) {
                    return;
                }
                if self.max_scroll_offset_y.0 <= 0.0 {
                    return;
                }
                let next = Px((self.scroll_offset_y.0 - delta.y.0)
                    .max(0.0)
                    .min(self.max_scroll_offset_y.0));
                if next != self.scroll_offset_y {
                    self.scroll_offset_y = next;
                    self.relayout_rows();
                    cx.invalidate_self(Invalidation::Layout);
                    cx.request_redraw();
                    cx.stop_propagation();
                }
            }
            Event::Pointer(fret_core::PointerEvent::Down {
                position, button, ..
            }) => {
                if *button != MouseButton::Left {
                    return;
                }
                if self
                    .dismiss
                    .should_dismiss(event, self.panel_bounds, false, true)
                {
                    self.close_popover(cx);
                } else if self.panel_bounds.contains(*position) {
                    cx.capture_pointer(cx.node);
                }
            }
            Event::Pointer(fret_core::PointerEvent::Up {
                position, button, ..
            }) => {
                if *button != MouseButton::Left {
                    return;
                }
                cx.release_pointer_capture();
                if let Some(i) = self.hit_test_row(*position) {
                    self.activate_row(cx, window, &request, i);
                }
            }
            Event::KeyDown { key, modifiers, .. } => {
                if self
                    .dismiss
                    .should_dismiss(event, self.panel_bounds, true, false)
                {
                    self.clear_typeahead();
                    self.close_popover(cx);
                    return;
                }
                if let Some(c) = Self::typeahead_char(*key, modifiers) {
                    if self.handle_typeahead(&request, c) {
                        if let Some(i) = self.hover_row {
                            self.ensure_row_visible(i);
                        }
                        self.update_a11y_flags_only(&request);
                        cx.invalidate_self(Invalidation::Layout);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                    return;
                }

                match key {
                    KeyCode::Escape => {
                        self.clear_typeahead();
                        self.close_popover(cx);
                    }
                    KeyCode::Enter | KeyCode::NumpadEnter | KeyCode::Space => {
                        self.clear_typeahead();
                        if let Some(i) = self.hover_row.or(request.selected) {
                            self.activate_row(cx, window, &request, i);
                        }
                    }
                    KeyCode::ArrowDown => {
                        self.clear_typeahead();
                        let base = self
                            .hover_row
                            .or(request.selected)
                            .or_else(|| self.first_enabled_row())
                            .unwrap_or(0);
                        if let Some(next) = self.next_enabled_row(base, 1) {
                            self.hover_row = Some(next);
                            self.ensure_row_visible(next);
                        }
                        self.update_a11y_flags_only(&request);
                        cx.invalidate_self(Invalidation::Layout);
                        cx.request_redraw();
                    }
                    KeyCode::ArrowUp => {
                        self.clear_typeahead();
                        let base = self
                            .hover_row
                            .or(request.selected)
                            .or_else(|| self.first_enabled_row())
                            .unwrap_or(0);
                        if let Some(next) = self.next_enabled_row(base, -1) {
                            self.hover_row = Some(next);
                            self.ensure_row_visible(next);
                        }
                        self.update_a11y_flags_only(&request);
                        cx.invalidate_self(Invalidation::Layout);
                        cx.request_redraw();
                    }
                    KeyCode::PageDown => {
                        self.clear_typeahead();
                        let base = self
                            .hover_row
                            .or(request.selected)
                            .or_else(|| self.first_enabled_row())
                            .unwrap_or(0);
                        if let Some(next) = self.page_step_enabled(self.panel_bounds, base, 1) {
                            self.hover_row = Some(next);
                            self.ensure_row_visible(next);
                        }
                        self.update_a11y_flags_only(&request);
                        cx.invalidate_self(Invalidation::Layout);
                        cx.request_redraw();
                    }
                    KeyCode::PageUp => {
                        self.clear_typeahead();
                        let base = self
                            .hover_row
                            .or(request.selected)
                            .or_else(|| self.first_enabled_row())
                            .unwrap_or(0);
                        if let Some(next) = self.page_step_enabled(self.panel_bounds, base, -1) {
                            self.hover_row = Some(next);
                            self.ensure_row_visible(next);
                        }
                        self.update_a11y_flags_only(&request);
                        cx.invalidate_self(Invalidation::Layout);
                        cx.request_redraw();
                    }
                    KeyCode::Home => {
                        self.clear_typeahead();
                        if let Some(i) = self.first_enabled_row() {
                            self.hover_row = Some(i);
                            self.ensure_row_visible(i);
                        }
                        self.update_a11y_flags_only(&request);
                        cx.invalidate_self(Invalidation::Layout);
                        cx.request_redraw();
                    }
                    KeyCode::End => {
                        self.clear_typeahead();
                        if let Some(i) = self.last_enabled_row() {
                            self.hover_row = Some(i);
                            self.ensure_row_visible(i);
                        }
                        self.update_a11y_flags_only(&request);
                        cx.invalidate_self(Invalidation::Layout);
                        cx.request_redraw();
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        let Some(window) = cx.window else {
            self.clear_a11y();
            return cx.available;
        };

        {
            let mut prep = PopoverPrepareCx {
                app: cx.app,
                services: cx.services,
                window: cx.window,
                bounds: cx.bounds,
                scale_factor: cx.scale_factor,
            };
            let _ = self.ensure_prepared(&mut prep);
        }

        let Some((_, request)) = cx
            .app
            .global::<PopoverService>()
            .and_then(|s| s.request(window))
            .map(|(serial, req)| (serial, req.clone()))
        else {
            self.clear_a11y();
            return cx.available;
        };

        if let Some(slot) = self.focused_slot_index(cx.focus, cx.children)
            && request.items.get(slot).is_some()
            && self.hover_row != Some(slot)
        {
            self.hover_row = Some(slot);
            self.ensure_row_visible(slot);
        }

        self.update_a11y_from_rows(&request);

        for (slot, &child) in cx.children.iter().enumerate() {
            let rect = self
                .a11y
                .borrow()
                .slot(slot)
                .map(|s| s.bounds)
                .unwrap_or_default();
            cx.layout_in(child, rect);
        }

        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        let Some(window) = cx.window else {
            return;
        };

        let has_request = cx
            .app
            .global::<PopoverService>()
            .and_then(|s| s.request(window))
            .is_some();
        if !has_request {
            return;
        }

        let Some(_) = ({
            let mut prep = PopoverPrepareCx {
                app: cx.app,
                services: cx.services,
                window: cx.window,
                bounds: cx.bounds,
                scale_factor: cx.scale_factor,
            };
            self.ensure_prepared(&mut prep)
        }) else {
            return;
        };

        let Some((_, request)) = cx
            .app
            .global::<PopoverService>()
            .and_then(|s| s.request(window))
            .map(|(serial, req)| (serial, req.clone()))
        else {
            return;
        };

        if request.items.is_empty() {
            return;
        }

        if let Some(shadow) = self.style.shadow {
            fret_ui::paint::paint_shadow(cx.scene, DrawOrder(0), self.panel_bounds, shadow);
        }
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: self.panel_bounds,
            background: self.style.background,
            border: self.style.border,
            border_color: self.style.border_color,
            corner_radii: self.style.corner_radii,
        });

        cx.scene.push(SceneOp::PushClipRRect {
            rect: self.panel_bounds,
            corner_radii: self.style.corner_radii,
        });

        let panel_top = self.panel_bounds.origin.y.0 + self.style.padding_y.0;
        let panel_bottom =
            self.panel_bounds.origin.y.0 + self.panel_bounds.size.height.0 - self.style.padding_y.0;

        for (i, row) in self.rows.iter().enumerate() {
            let y = row.bounds.origin.y.0;
            let h = row.bounds.size.height.0.max(0.0);
            if y + h < panel_top {
                continue;
            }
            if y > panel_bottom {
                break;
            }

            let selected = request.selected == Some(i);
            let hovered = self.hover_row == Some(i);
            let bg = if selected {
                self.style.row_selected
            } else if hovered {
                self.style.row_hover
            } else {
                Color {
                    a: 0.0,
                    ..self.style.background
                }
            };

            if selected || hovered {
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(1),
                    rect: row.bounds,
                    background: bg,
                    border: Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: Corners::all(Px(0.0)),
                });
            }

            let text_x = Px(row.bounds.origin.x.0 + self.style.padding_x.0);
            let inner_y = row.bounds.origin.y.0
                + ((row.bounds.size.height.0 - row.metrics.size.height.0) * 0.5);
            let text_y = Px(inner_y + row.metrics.baseline.0);
            cx.scene.push(SceneOp::Text {
                order: DrawOrder(2),
                origin: Point::new(text_x, text_y),
                text: row.label,
                color: if row.enabled {
                    self.style.text_color
                } else {
                    self.style.disabled_text_color
                },
            });
        }

        cx.scene.push(SceneOp::PopClip);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{
        AppWindowId, PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle,
        Point, Px, Rect, Scene, SceneOp, Size, TextService,
    };
    use fret_ui::UiTree;

    #[derive(Default)]
    struct FakeServices(());

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _text: &str,
            _style: fret_core::TextStyle,
            _constraints: fret_core::TextConstraints,
        ) -> (fret_core::TextBlobId, fret_core::TextMetrics) {
            (
                fret_core::TextBlobId::default(),
                fret_core::TextMetrics {
                    size: Size::new(Px(80.0), Px(12.0)),
                    baseline: Px(10.0),
                },
            )
        }

        fn release(&mut self, _blob: fret_core::TextBlobId) {}
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

    impl fret_core::SvgService for FakeServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
            fret_core::SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
            false
        }
    }

    fn clip_rect(scene: &Scene) -> Option<Rect> {
        scene.ops().iter().find_map(|op| match op {
            SceneOp::PushClipRRect { rect, .. } => Some(*rect),
            _ => None,
        })
    }

    #[test]
    fn popover_flips_to_top_near_bottom_and_does_not_overflow() {
        let mut host = App::new();
        let mut services = FakeServices::default();

        let window = AppWindowId::default();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let popover = ui.create_node(Popover::new());
        ui.set_root(popover);

        let outer = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(200.0)),
        );

        let anchor = Rect::new(
            Point::new(Px(10.0), Px(190.0)),
            Size::new(Px(40.0), Px(10.0)),
        );

        let items: Vec<PopoverItem> = (0..32)
            .map(|i| PopoverItem::new(format!("Item {i}")))
            .collect();

        host.with_global_mut(PopoverService::default, |service, _app| {
            service.set_request(
                window,
                PopoverRequest {
                    owner: popover,
                    anchor,
                    items,
                    selected: None,
                    request_focus: true,
                },
            );
        });

        ui.layout_all(&mut host, &mut services, outer, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut host, &mut services, outer, &mut scene, 1.0);

        let Some(bounds) = clip_rect(&scene) else {
            panic!("expected a popover clip rect to be emitted");
        };

        // Must stay within the window.
        assert!(bounds.origin.x.0 >= 0.0);
        assert!(bounds.origin.y.0 >= 0.0);
        assert!(bounds.origin.x.0 + bounds.size.width.0 <= 200.0);
        assert!(bounds.origin.y.0 + bounds.size.height.0 <= 200.0);

        // Near the bottom edge, prefer flipping above the anchor rather than overflowing below.
        assert!(bounds.origin.y.0 + bounds.size.height.0 <= anchor.origin.y.0);
    }

    #[test]
    fn popover_uses_window_margin_to_avoid_shadow_clipping_at_bottom_edge() {
        let mut host = App::new();
        let mut services = FakeServices::default();

        let window = AppWindowId::default();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let style = PopoverStyle {
            padding_x: Px(0.0),
            padding_y: Px(0.0),
            row_height: Px(10.0),
            gap: Px(2.0),
            window_margin: Px(8.0),
            ..PopoverStyle::default()
        };

        let popover = ui.create_node(Popover::new().with_style(style));
        ui.set_root(popover);

        let outer = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(200.0)),
        );

        // This placement would fit exactly at the bottom edge without a margin:
        // anchor_bottom (188) + gap (2) + panel_h (10) == 200.
        // With a window margin of 8px, we expect the solver to flip above instead.
        let anchor = Rect::new(
            Point::new(Px(10.0), Px(178.0)),
            Size::new(Px(40.0), Px(10.0)),
        );

        host.with_global_mut(PopoverService::default, |service, _app| {
            service.set_request(
                window,
                PopoverRequest {
                    owner: popover,
                    anchor,
                    items: vec![PopoverItem::new("Item")],
                    selected: None,
                    request_focus: true,
                },
            );
        });

        ui.layout_all(&mut host, &mut services, outer, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut host, &mut services, outer, &mut scene, 1.0);

        let Some(bounds) = clip_rect(&scene) else {
            panic!("expected a popover clip rect to be emitted");
        };

        // With margin accounted for, we should flip above the anchor.
        assert!(bounds.origin.y.0 + bounds.size.height.0 <= anchor.origin.y.0);

        // And we should stay within the inset window bounds.
        let inset_bottom = 200.0 - 8.0;
        assert!(bounds.origin.y.0 + bounds.size.height.0 <= inset_bottom);
    }
}
