use std::sync::Arc;

use fret_core::{
    Color, Corners, CursorIcon, DrawOrder, Edges, Event, FontId, KeyCode, MouseButton, Point, Px,
    Rect, SceneOp, SemanticsRole, Size, TextConstraints, TextOverflow, TextStyle, TextWrap,
};
use fret_runtime::{CommandId, Model};
use fret_ui::widget::{EventCx, Invalidation, LayoutCx, PaintCx, SemanticsCx, Widget};
use fret_ui::{Theme, UiHost};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Date {
    pub year: i32,
    pub month: u8,
    pub day: u8,
}

impl Date {
    pub fn new(year: i32, month: u8, day: u8) -> Self {
        Self { year, month, day }
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

/// Weekday for a Gregorian date (0=Sunday..6=Saturday).
fn weekday_sun0(year: i32, month: u8, day: u8) -> u8 {
    // Zeller's congruence (Gregorian).
    // h = 0 => Saturday, 1 => Sunday, 2 => Monday, ...
    let q = day as i32;
    let mut m = month as i32;
    let mut y = year;
    if m <= 2 {
        m += 12;
        y -= 1;
    }
    let k = y.rem_euclid(100);
    let j = y.div_euclid(100);
    let h = (q + (13 * (m + 1)) / 5 + k + k / 4 + j / 4 + 5 * j).rem_euclid(7);
    // Convert: Zeller Sunday=1 -> 0, Monday=2 -> 1, ..., Saturday=0 -> 6.
    ((h + 6) % 7) as u8
}

fn month_name(month: u8) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "Month",
    }
}

fn title_style(theme: &Theme) -> TextStyle {
    let size = theme
        .metric_by_key("component.calendar.title_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or(theme.metrics.font_size);
    let line_height = theme
        .metric_by_key("component.calendar.title_line_height")
        .or_else(|| theme.metric_by_key("font.line_height"))
        .unwrap_or(theme.metrics.font_line_height);
    TextStyle {
        font: FontId::default(),
        size,
        line_height: Some(line_height),
        ..Default::default()
    }
}

fn cell_style(theme: &Theme) -> TextStyle {
    let size = theme
        .metric_by_key("component.calendar.cell_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or(theme.metrics.font_size);
    let line_height = theme
        .metric_by_key("component.calendar.cell_line_height")
        .or_else(|| theme.metric_by_key("font.line_height"))
        .unwrap_or(theme.metrics.font_line_height);
    TextStyle {
        font: FontId::default(),
        size,
        line_height: Some(line_height),
        ..Default::default()
    }
}

fn cell_size(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.calendar.cell_size")
        .unwrap_or(Px(40.0))
}

fn gap(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.calendar.gap")
        .or_else(|| theme.metric_by_key("component.space.1"))
        .unwrap_or(Px(4.0))
}

fn radius(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.calendar.radius")
        .or_else(|| theme.metric_by_key("component.radius.md"))
        .unwrap_or(theme.metrics.radius_md)
}

fn fg(theme: &Theme) -> Color {
    theme
        .color_by_key("foreground")
        .unwrap_or(theme.colors.text_primary)
}

fn muted_fg(theme: &Theme) -> Color {
    theme
        .color_by_key("muted.foreground")
        .or_else(|| theme.color_by_key("muted-foreground"))
        .unwrap_or(theme.colors.text_muted)
}

fn accent_bg(theme: &Theme) -> Color {
    theme
        .color_by_key("accent")
        .or_else(|| theme.color_by_key("accent.background"))
        .unwrap_or(theme.colors.hover_background)
}

fn accent_fg(theme: &Theme) -> Color {
    theme
        .color_by_key("accent.foreground")
        .or_else(|| theme.color_by_key("accent-foreground"))
        .unwrap_or(theme.colors.text_primary)
}

fn primary_bg(theme: &Theme) -> Color {
    theme
        .color_by_key("primary")
        .or_else(|| theme.color_by_key("primary.background"))
        .unwrap_or(theme.colors.accent)
}

fn primary_fg(theme: &Theme) -> Color {
    theme
        .color_by_key("primary.foreground")
        .or_else(|| theme.color_by_key("primary-foreground"))
        .unwrap_or(theme.colors.text_primary)
}

fn border(theme: &Theme) -> Color {
    theme
        .color_by_key("border")
        .unwrap_or(theme.colors.panel_border)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HitTarget {
    Prev,
    Next,
    Day(Date, bool /* in_month */),
}

/// shadcn/ui `Calendar` (prototype).
///
/// This is a self-contained month grid widget that updates a `Model<Option<Date>>`.
///
/// Known gaps vs shadcn:
/// - no multi-month, range, or disabled-days rules yet,
/// - no per-cell semantics nodes (retained widget draws the grid),
/// - minimal keyboard navigation (click + arrows are not implemented).
pub struct Calendar {
    model: Model<Option<Date>>,
    on_select: Option<CommandId>,
    disabled: bool,
    show_outside_days: bool,

    view_year: i32,
    view_month: u8,

    last_bounds: Rect,
    hovered: Option<HitTarget>,
    pressed: Option<HitTarget>,
}

impl Calendar {
    pub fn new(model: Model<Option<Date>>) -> Self {
        Self {
            model,
            on_select: None,
            disabled: false,
            show_outside_days: true,
            view_year: 2025,
            view_month: 1,
            last_bounds: Rect::default(),
            hovered: None,
            pressed: None,
        }
    }

    pub fn on_select(mut self, command: impl Into<CommandId>) -> Self {
        self.on_select = Some(command.into());
        self
    }

    pub fn month(mut self, year: i32, month: u8) -> Self {
        self.view_year = year;
        self.view_month = month.clamp(1, 12);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn show_outside_days(mut self, show: bool) -> Self {
        self.show_outside_days = show;
        self
    }

    fn read_value<H: UiHost>(&self, app: &H) -> Option<Date> {
        app.models().get(self.model).copied().flatten()
    }

    fn write_value<H: UiHost>(&self, app: &mut H, next: Option<Date>) {
        let _ = app.models_mut().update(self.model, |v| *v = next);
    }

    fn prev_month(&mut self) {
        if self.view_month <= 1 {
            self.view_month = 12;
            self.view_year = self.view_year.saturating_sub(1);
        } else {
            self.view_month = self.view_month.saturating_sub(1);
        }
    }

    fn next_month(&mut self) {
        if self.view_month >= 12 {
            self.view_month = 1;
            self.view_year = self.view_year.saturating_add(1);
        } else {
            self.view_month = self.view_month.saturating_add(1);
        }
    }

    fn hit_test(&self, theme: &Theme, p: Point) -> Option<HitTarget> {
        if !self.last_bounds.contains(p) {
            return None;
        }

        let cell = cell_size(theme);
        let gap = gap(theme);
        let header_h = cell;
        let header = Rect::new(
            self.last_bounds.origin,
            Size::new(self.last_bounds.size.width, header_h),
        );

        // Prev/Next buttons: small squares on the right of the header.
        let btn = cell;
        let next_rect = Rect::new(
            Point::new(
                Px(header.origin.x.0 + header.size.width.0 - btn.0),
                header.origin.y,
            ),
            Size::new(btn, btn),
        );
        let prev_rect = Rect::new(
            Point::new(
                Px(header.origin.x.0 + header.size.width.0 - btn.0 * 2.0 - gap.0),
                header.origin.y,
            ),
            Size::new(btn, btn),
        );
        if prev_rect.contains(p) {
            return Some(HitTarget::Prev);
        }
        if next_rect.contains(p) {
            return Some(HitTarget::Next);
        }

        // Grid.
        let grid_origin = Point::new(
            self.last_bounds.origin.x,
            Px(self.last_bounds.origin.y.0 + header_h.0 + gap.0),
        );
        let first_weekday = weekday_sun0(self.view_year, self.view_month, 1) as i32;
        let month_days = days_in_month(self.view_year, self.view_month) as i32;

        let (prev_year, prev_month) = if self.view_month == 1 {
            (self.view_year - 1, 12)
        } else {
            (self.view_year, self.view_month - 1)
        };
        let prev_days = days_in_month(prev_year, prev_month) as i32;

        for row in 0..6 {
            for col in 0..7 {
                let i = row * 7 + col;
                let x = Px(grid_origin.x.0 + (col as f32) * (cell.0 + gap.0));
                let y = Px(grid_origin.y.0 + (row as f32) * (cell.0 + gap.0));
                let rect = Rect::new(Point::new(x, y), Size::new(cell, cell));
                if !rect.contains(p) {
                    continue;
                }

                let day_index = i as i32 - first_weekday + 1;
                if day_index >= 1 && day_index <= month_days {
                    return Some(HitTarget::Day(
                        Date::new(self.view_year, self.view_month, day_index as u8),
                        true,
                    ));
                }

                if !self.show_outside_days {
                    return None;
                }

                if day_index < 1 {
                    let d = prev_days + day_index;
                    if d >= 1 && d <= prev_days {
                        return Some(HitTarget::Day(
                            Date::new(prev_year, prev_month, d as u8),
                            false,
                        ));
                    }
                } else {
                    // Next month.
                    let (next_year, next_month) = if self.view_month == 12 {
                        (self.view_year + 1, 1)
                    } else {
                        (self.view_year, self.view_month + 1)
                    };
                    let d = day_index - month_days;
                    let next_days = days_in_month(next_year, next_month) as i32;
                    if d >= 1 && d <= next_days {
                        return Some(HitTarget::Day(
                            Date::new(next_year, next_month, d as u8),
                            false,
                        ));
                    }
                }

                return None;
            }
        }

        None
    }
}

impl<H: UiHost> Widget<H> for Calendar {
    fn is_focusable(&self) -> bool {
        !self.disabled
    }

    fn hit_test(&self, bounds: Rect, position: Point) -> bool {
        if self.disabled {
            return false;
        }
        bounds.contains(position)
    }

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        cx.set_role(SemanticsRole::Panel);
        cx.set_disabled(self.disabled);
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.observe_model(self.model, Invalidation::Layout);
        self.last_bounds = cx.bounds;

        let theme = cx.theme();
        let cell = cell_size(theme);
        let gap = gap(theme);
        let header_h = cell;

        let width = Px((cell.0 * 7.0 + gap.0 * 6.0).min(cx.available.width.0.max(0.0)));
        let height =
            Px((header_h.0 + gap.0 + cell.0 * 6.0 + gap.0 * 5.0)
                .min(cx.available.height.0.max(0.0)));
        Size::new(width, height)
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        let Some(window) = cx.window else {
            return;
        };

        if self.disabled {
            return;
        }

        let theme = cx.theme().clone();
        match event {
            Event::Pointer(pe) => match pe {
                fret_core::PointerEvent::Move { position, .. } => {
                    let hit = self.hit_test(&theme, *position);
                    if hit != self.hovered {
                        self.hovered = hit;
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                    }

                    if hit.is_some() || cx.captured == Some(cx.node) {
                        cx.set_cursor_icon(CursorIcon::Pointer);
                    }
                }
                fret_core::PointerEvent::Down {
                    position, button, ..
                } => {
                    if *button != MouseButton::Left {
                        return;
                    }
                    let hit = self.hit_test(&theme, *position);
                    if hit.is_none() {
                        return;
                    }
                    self.pressed = hit;
                    cx.capture_pointer(cx.node);
                    cx.request_focus(cx.node);
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                }
                fret_core::PointerEvent::Up {
                    position, button, ..
                } => {
                    if *button != MouseButton::Left {
                        return;
                    }
                    cx.release_pointer_capture();
                    let pressed = self.pressed.take();
                    let hit = self.hit_test(&theme, *position);
                    self.hovered = hit;

                    if pressed.is_some() && pressed == hit {
                        match pressed.expect("pressed exists") {
                            HitTarget::Prev => {
                                self.prev_month();
                                cx.invalidate_self(Invalidation::Layout);
                                cx.invalidate_self(Invalidation::Paint);
                                cx.request_redraw();
                            }
                            HitTarget::Next => {
                                self.next_month();
                                cx.invalidate_self(Invalidation::Layout);
                                cx.invalidate_self(Invalidation::Paint);
                                cx.request_redraw();
                            }
                            HitTarget::Day(date, in_month) => {
                                if in_month {
                                    self.write_value(cx.app, Some(date));
                                    if let Some(command) = self.on_select.clone() {
                                        cx.dispatch_command(command);
                                    }
                                    cx.invalidate_self(Invalidation::Paint);
                                    cx.request_redraw();
                                }
                            }
                        }
                    } else {
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                    }

                    cx.stop_propagation();
                }
                _ => {}
            },
            Event::KeyDown { key, repeat, .. } => {
                if *repeat {
                    return;
                }
                if cx.focus != Some(cx.node) {
                    return;
                }
                match key {
                    KeyCode::ArrowLeft => {
                        self.prev_month();
                        cx.invalidate_self(Invalidation::Layout);
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                    KeyCode::ArrowRight => {
                        self.next_month();
                        cx.invalidate_self(Invalidation::Layout);
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        // Keep `window` used to avoid warnings in case event patterns change.
        let _ = window;
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        cx.observe_model(self.model, Invalidation::Paint);
        self.last_bounds = cx.bounds;

        let theme = cx.theme().clone();
        let cell = cell_size(&theme);
        let gap = gap(&theme);
        let r = radius(&theme);

        let selected = self.read_value(cx.app);

        // Header.
        let header_h = cell;
        let header = Rect::new(cx.bounds.origin, Size::new(cx.bounds.size.width, header_h));

        let title = format!("{} {}", month_name(self.view_month), self.view_year);
        let title_style = title_style(&theme);
        let title_constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor,
        };
        let (title_blob, title_metrics) =
            cx.services
                .text()
                .prepare(&title, title_style, title_constraints);
        let title_x = header.origin.x.0;
        let title_top =
            header.origin.y.0 + (header.size.height.0 - title_metrics.size.height.0) * 0.5;
        let title_y = title_top + title_metrics.baseline.0;
        cx.scene.push(SceneOp::Text {
            order: DrawOrder(0),
            origin: Point::new(Px(title_x), Px(title_y)),
            text: title_blob,
            color: fg(&theme),
        });
        cx.services.text().release(title_blob);

        // Prev/Next buttons.
        let btn = cell;
        let next_rect = Rect::new(
            Point::new(
                Px(header.origin.x.0 + header.size.width.0 - btn.0),
                header.origin.y,
            ),
            Size::new(btn, btn),
        );
        let prev_rect = Rect::new(
            Point::new(
                Px(header.origin.x.0 + header.size.width.0 - btn.0 * 2.0 - gap.0),
                header.origin.y,
            ),
            Size::new(btn, btn),
        );

        let draw_icon =
            |cx: &mut PaintCx<'_, H>, rect: Rect, glyph: &str, hovered: bool, pressed: bool| {
                let mut bg = accent_bg(&theme);
                bg.a *= if pressed {
                    0.8
                } else if hovered {
                    0.5
                } else {
                    0.0
                };
                if bg.a > 0.0 {
                    cx.scene.push(SceneOp::Quad {
                        order: DrawOrder(1),
                        rect,
                        background: bg,
                        border: Edges::all(Px(0.0)),
                        border_color: Color::TRANSPARENT,
                        corner_radii: Corners::all(r),
                    });
                }

                let (blob, metrics) = cx.services.text().prepare(
                    glyph,
                    cell_style(&theme),
                    TextConstraints {
                        max_width: None,
                        wrap: TextWrap::None,
                        overflow: TextOverflow::Clip,
                        scale_factor: cx.scale_factor,
                    },
                );
                let gx = rect.origin.x.0 + (rect.size.width.0 - metrics.size.width.0) * 0.5;
                let top = rect.origin.y.0 + (rect.size.height.0 - metrics.size.height.0) * 0.5;
                let gy = top + metrics.baseline.0;
                cx.scene.push(SceneOp::Text {
                    order: DrawOrder(2),
                    origin: Point::new(Px(gx), Px(gy)),
                    text: blob,
                    color: fg(&theme),
                });
                cx.services.text().release(blob);
            };

        let hovered = self.hovered;
        let pressed = self.pressed;
        draw_icon(
            cx,
            prev_rect,
            "‹",
            hovered == Some(HitTarget::Prev),
            pressed == Some(HitTarget::Prev),
        );
        draw_icon(
            cx,
            next_rect,
            "›",
            hovered == Some(HitTarget::Next),
            pressed == Some(HitTarget::Next),
        );

        // Grid origin.
        let grid_origin = Point::new(
            cx.bounds.origin.x,
            Px(cx.bounds.origin.y.0 + header_h.0 + gap.0),
        );

        let first_weekday = weekday_sun0(self.view_year, self.view_month, 1) as i32;
        let month_days = days_in_month(self.view_year, self.view_month) as i32;

        let (prev_year, prev_month) = if self.view_month == 1 {
            (self.view_year - 1, 12)
        } else {
            (self.view_year, self.view_month - 1)
        };
        let prev_days = days_in_month(prev_year, prev_month) as i32;

        let (next_year, next_month) = if self.view_month == 12 {
            (self.view_year + 1, 1)
        } else {
            (self.view_year, self.view_month + 1)
        };

        let cell_style = cell_style(&theme);
        let cell_constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor,
        };

        for row in 0..6 {
            for col in 0..7 {
                let i = row * 7 + col;
                let x = Px(grid_origin.x.0 + (col as f32) * (cell.0 + gap.0));
                let y = Px(grid_origin.y.0 + (row as f32) * (cell.0 + gap.0));
                let rect = Rect::new(Point::new(x, y), Size::new(cell, cell));

                let day_index = i as i32 - first_weekday + 1;
                let (date, in_month) = if day_index >= 1 && day_index <= month_days {
                    (
                        Date::new(self.view_year, self.view_month, day_index as u8),
                        true,
                    )
                } else if !self.show_outside_days {
                    continue;
                } else if day_index < 1 {
                    let d = prev_days + day_index;
                    if d < 1 {
                        continue;
                    }
                    (Date::new(prev_year, prev_month, d as u8), false)
                } else {
                    let d = day_index - month_days;
                    let next_days = days_in_month(next_year, next_month) as i32;
                    if d < 1 || d > next_days {
                        continue;
                    }
                    (Date::new(next_year, next_month, d as u8), false)
                };

                let mut bg = Color::TRANSPARENT;
                let mut fg = if in_month {
                    fg(&theme)
                } else {
                    muted_fg(&theme)
                };

                let hovered_day = matches!(hovered, Some(HitTarget::Day(d, _)) if d == date);
                let selected_day = selected == Some(date);

                if selected_day {
                    bg = primary_bg(&theme);
                    fg = primary_fg(&theme);
                } else if hovered_day && in_month {
                    bg = accent_bg(&theme);
                    fg = accent_fg(&theme);
                }

                if bg.a > 0.0 {
                    cx.scene.push(SceneOp::Quad {
                        order: DrawOrder(10),
                        rect,
                        background: bg,
                        border: Edges::all(Px(0.0)),
                        border_color: Color::TRANSPARENT,
                        corner_radii: Corners::all(r),
                    });
                }

                // Day number.
                let label = Arc::<str>::from(format!("{}", date.day));
                let (blob, metrics) =
                    cx.services
                        .text()
                        .prepare(&label, cell_style, cell_constraints);
                let gx = rect.origin.x.0 + (rect.size.width.0 - metrics.size.width.0) * 0.5;
                let top = rect.origin.y.0 + (rect.size.height.0 - metrics.size.height.0) * 0.5;
                let gy = top + metrics.baseline.0;
                cx.scene.push(SceneOp::Text {
                    order: DrawOrder(11),
                    origin: Point::new(Px(gx), Px(gy)),
                    text: blob,
                    color: fg,
                });
                cx.services.text().release(blob);

                // Outside days: a faint border hint.
                if !in_month {
                    cx.scene.push(SceneOp::Quad {
                        order: DrawOrder(9),
                        rect,
                        background: Color::TRANSPARENT,
                        border: Edges::all(Px(1.0)),
                        border_color: border(&theme),
                        corner_radii: Corners::all(r),
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::{
        MouseButton, NodeId, PathCommand, PathConstraints, PathId, PathMetrics, PathService,
        PathStyle, Point, Px, Rect, Size, SvgId, SvgService, TextBlobId, TextConstraints,
        TextMetrics, TextService, TextStyle,
    };
    use fret_runtime::{CommandId, Effect, InputContext, Platform};
    use fret_ui::{Theme, widget::EventCx};

    #[test]
    fn weekday_is_stable_for_known_dates() {
        // 2024-01-01 was a Monday.
        assert_eq!(weekday_sun0(2024, 1, 1), 1);
        // 2024-12-25 was a Wednesday.
        assert_eq!(weekday_sun0(2024, 12, 25), 3);
    }

    #[test]
    fn days_in_month_handles_leap_years() {
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2025, 2), 28);
    }

    #[derive(Default)]
    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _text: &str,
            _style: TextStyle,
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
            false
        }
    }

    #[test]
    fn selecting_a_day_dispatches_on_select_command() {
        let mut host = crate::test_host::TestHost::default();
        let window = fret_core::AppWindowId::default();
        let model = host.models_mut().insert(None::<Date>);

        let mut calendar = Calendar::new(model)
            .month(2025, 1)
            .on_select(CommandId::from("popover_surface.close"));

        // Establish bounds for hit testing.
        calendar.last_bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(400.0)),
        );

        let theme = Theme::global(&host).clone();
        let cell = cell_size(&theme);
        let gap = gap(&theme);
        let header_h = cell;
        let grid_origin = Point::new(Px(0.0), Px(header_h.0 + gap.0));

        let first_weekday = weekday_sun0(2025, 1, 1) as f32;
        let col = first_weekday;
        let pos = Point::new(
            Px(grid_origin.x.0 + col * (cell.0 + gap.0) + 1.0),
            Px(grid_origin.y.0 + 1.0),
        );

        let mut services = FakeServices::default();
        let mut cx = EventCx {
            app: &mut host,
            services: &mut services,
            node: NodeId::default(),
            window: Some(window),
            input_ctx: InputContext {
                platform: Platform::Linux,
                caps: fret_core::PlatformCapabilities::default(),
                ui_has_modal: false,
                focus_is_text_input: false,
            },
            children: &[],
            focus: None,
            captured: None,
            bounds: calendar.last_bounds,
            invalidations: Vec::new(),
            requested_focus: None,
            requested_capture: None,
            requested_cursor: None,
            stop_propagation: false,
        };

        calendar.event(
            &mut cx,
            &Event::Pointer(fret_core::PointerEvent::Down {
                position: pos,
                button: MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );
        calendar.event(
            &mut cx,
            &Event::Pointer(fret_core::PointerEvent::Up {
                position: pos,
                button: MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );

        let dispatched = host.effects().iter().any(|e| {
            matches!(
                e,
                Effect::Command { window: Some(w), command }
                    if *w == window && command.as_str() == "popover_surface.close"
            )
        });
        assert!(
            dispatched,
            "expected on_select to dispatch popover_surface.close"
        );
    }
}
