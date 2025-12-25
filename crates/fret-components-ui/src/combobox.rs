use std::sync::Arc;

use fret_core::{Event, KeyCode, Modifiers, Px, TextStyle};
use fret_runtime::{CommandId, Model};
use fret_ui::primitives::BoundTextInput;
use fret_ui::{EventCx, Invalidation, LayoutCx, PaintCx, UiHost, Widget};

use crate::recipes::input::{InputTokenKeys, resolve_input_chrome};
use crate::style::StyleRefinement;
use crate::{PopoverItem, PopoverRequest, PopoverService, Size};

fn matches_query(label: &str, q: &str) -> bool {
    let q = q.trim();
    if q.is_empty() {
        return true;
    }
    label.to_ascii_lowercase().contains(q)
}

fn visible_indices(items: &[String], query: &str) -> Vec<usize> {
    let q = query.trim().to_ascii_lowercase();
    items
        .iter()
        .enumerate()
        .filter(|(_, s)| matches_query(s.as_str(), &q))
        .map(|(i, _)| i)
        .collect()
}

/// A minimal, shadcn-inspired combobox (typeahead + list) built on top of `TextInput` + `Popover`.
///
/// Design notes:
/// - The popover list is rendered via `fret-ui`'s `Popover` overlay (anchored below the input).
/// - Focus stays in the text input while the list is open, so typing continues to update the query.
/// - Arrow Up/Down and Enter are handled by this widget (cmdk-style), not by the popover.
pub struct Combobox {
    items: Model<Vec<String>>,
    selection: Model<Option<usize>>,
    query: Model<String>,

    size: Size,
    style: StyleRefinement,
    min_height: Px,

    input: BoundTextInput,
    last_bounds: fret_core::Rect,

    last_theme_revision: Option<u64>,
    last_items_revision: Option<u64>,
    last_query_revision: Option<u64>,
    last_opened: bool,
    active_filtered: Option<usize>,
}

impl Combobox {
    pub fn new(
        items: Model<Vec<String>>,
        selection: Model<Option<usize>>,
        query: Model<String>,
    ) -> Self {
        Self {
            items,
            selection,
            query,
            size: Size::Medium,
            style: StyleRefinement::default(),
            min_height: Px(0.0),
            input: BoundTextInput::new(query),
            last_bounds: fret_core::Rect::default(),
            last_theme_revision: None,
            last_items_revision: None,
            last_query_revision: None,
            last_opened: false,
            active_filtered: None,
        }
    }

    pub fn with_size(mut self, size: Size) -> Self {
        self.size = size;
        self.last_theme_revision = None;
        self
    }

    pub fn refine_style(mut self, style: StyleRefinement) -> Self {
        self.style = style;
        self.last_theme_revision = None;
        self
    }

    fn is_plain(mods: Modifiers) -> bool {
        !mods.shift && !mods.ctrl && !mods.alt && !mods.alt_gr && !mods.meta
    }

    fn is_open<H: UiHost>(&self, cx: &EventCx<'_, H>) -> bool {
        let Some(window) = cx.window else {
            return false;
        };
        cx.app
            .global::<PopoverService>()
            .and_then(|s| s.request(window))
            .is_some_and(|(_, req)| req.owner == cx.node)
    }

    fn set_open<H: UiHost>(&mut self, cx: &mut EventCx<'_, H>, open: bool) {
        if open {
            self.sync_popover_request(cx);
            cx.dispatch_command(CommandId::from("popover.open"));
        } else {
            cx.dispatch_command(CommandId::from("popover.close"));
        }
    }

    fn sync_popover_request<H: UiHost>(&mut self, cx: &mut EventCx<'_, H>) {
        let Some(window) = cx.window else {
            return;
        };

        // Keep anchor rect up-to-date even if the tree uses fast-path translations.
        self.last_bounds = cx.bounds;

        let items = cx.app.models().get(self.items).cloned().unwrap_or_default();
        let query = cx.app.models().get(self.query).cloned().unwrap_or_default();
        let visible = visible_indices(&items, &query);

        // Clamp active selection within filtered results.
        if let Some(active) = self.active_filtered {
            if active >= visible.len() {
                self.active_filtered = visible.first().copied();
            }
        } else {
            self.active_filtered = visible.first().copied();
        }

        let popover_items: Vec<PopoverItem> = visible
            .iter()
            .map(|&i| PopoverItem::new(Arc::<str>::from(items[i].as_str())))
            .collect();

        let selected = self
            .active_filtered
            .and_then(|key| visible.iter().position(|&i| i == key));

        cx.app
            .with_global_mut(PopoverService::default, |service, _app| {
                service.set_request(
                    window,
                    PopoverRequest {
                        owner: cx.node,
                        anchor: self.last_bounds,
                        items: popover_items,
                        selected,
                        request_focus: false,
                    },
                );
            });
    }

    fn sync_popover_result<H: UiHost>(&mut self, cx: &mut EventCx<'_, H>) -> bool {
        let Some(window) = cx.window else {
            return false;
        };
        let Some(selected_row) = cx
            .app
            .global_mut::<PopoverService>()
            .and_then(|s| s.take_result(window, cx.node))
        else {
            return false;
        };

        let items = cx.app.models().get(self.items).cloned().unwrap_or_default();
        let query = cx.app.models().get(self.query).cloned().unwrap_or_default();
        let visible = visible_indices(&items, &query);
        let Some(&picked) = visible.get(selected_row) else {
            return false;
        };

        self.active_filtered = Some(picked);

        let _ = cx
            .app
            .models_mut()
            .update(self.selection, |v| *v = Some(picked));
        let _ = cx.app.models_mut().update(self.query, |v| {
            *v = items.get(picked).cloned().unwrap_or_default()
        });

        cx.invalidate_self(Invalidation::Layout);
        cx.invalidate_self(Invalidation::Paint);
        cx.request_redraw();
        true
    }

    fn move_active<H: UiHost>(&mut self, cx: &mut EventCx<'_, H>, dir: i32) -> bool {
        let items = cx.app.models().get(self.items).cloned().unwrap_or_default();
        let query = cx.app.models().get(self.query).cloned().unwrap_or_default();
        let visible = visible_indices(&items, &query);
        if visible.is_empty() {
            self.active_filtered = None;
            self.sync_popover_request(cx);
            return false;
        }

        let current_pos = self
            .active_filtered
            .and_then(|k| visible.iter().position(|&i| i == k).map(|p| p as i32));

        let next_pos = match current_pos {
            None => {
                if dir >= 0 {
                    0
                } else {
                    (visible.len() as i32).saturating_sub(1)
                }
            }
            Some(p) => (p + dir).clamp(0, (visible.len() as i32).saturating_sub(1)),
        } as usize;

        let next = visible.get(next_pos).copied();
        self.active_filtered = next;
        self.sync_popover_request(cx);
        cx.request_redraw();
        true
    }

    fn accept_active<H: UiHost>(&mut self, cx: &mut EventCx<'_, H>) -> bool {
        let items = cx.app.models().get(self.items).cloned().unwrap_or_default();
        let Some(picked) = self.active_filtered else {
            return false;
        };
        if picked >= items.len() {
            return false;
        }
        let _ = cx
            .app
            .models_mut()
            .update(self.selection, |v| *v = Some(picked));
        let _ = cx.app.models_mut().update(self.query, |v| {
            *v = items.get(picked).cloned().unwrap_or_default()
        });
        self.set_open(cx, false);
        cx.stop_propagation();
        true
    }

    fn sync_chrome(&mut self, theme: &fret_ui::Theme) {
        if self.last_theme_revision == Some(theme.revision()) {
            return;
        }
        self.last_theme_revision = Some(theme.revision());

        let resolved = resolve_input_chrome(
            theme,
            self.size,
            &self.style,
            InputTokenKeys {
                // Reuse text-field keys as the component-specific override surface.
                padding_x: Some("component.text_field.padding_x"),
                padding_y: Some("component.text_field.padding_y"),
                min_height: Some("component.text_field.min_height"),
                radius: Some("component.text_field.radius"),
                border_width: Some("component.text_field.border_width"),
                bg: Some("component.text_field.bg"),
                border: Some("component.text_field.border"),
                border_focus: Some("component.text_field.border_focus"),
                fg: Some("component.text_field.fg"),
                text_px: Some("component.text_field.text_px"),
                selection: Some("component.text_field.selection"),
            },
        );

        let snap = theme.snapshot();
        let mut chrome = fret_ui::primitives::TextInputStyle::from_theme(snap);

        chrome.padding_x = resolved.padding_x;
        chrome.padding_y = resolved.padding_y;
        chrome.corner_radii = fret_core::geometry::Corners::all(resolved.radius);
        chrome.border = fret_core::geometry::Edges::all(resolved.border_width);
        chrome.background = resolved.background;
        chrome.border_color = resolved.border_color;
        chrome.border_color_focused = resolved.border_color_focused;
        chrome.text_color = resolved.text_color;
        chrome.caret_color = resolved.text_color;
        chrome.selection_color = resolved.selection_color;

        let text_px = resolved.text_px;

        self.input.set_text_style(TextStyle {
            font: fret_core::FontId::default(),
            size: text_px,
            ..Default::default()
        });
        self.input.set_chrome_style(chrome);

        self.min_height = resolved.min_height;
    }
}

impl<H: UiHost> Widget<H> for Combobox {
    fn is_focusable(&self) -> bool {
        true
    }

    fn is_text_input(&self) -> bool {
        true
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        self.sync_chrome(cx.theme());
        self.last_bounds = cx.bounds;

        if self.sync_popover_result(cx) {
            return;
        }

        let open = self.is_open(cx);

        if let Event::KeyDown { key, modifiers, .. } = event
            && cx.focus == Some(cx.node)
            && Self::is_plain(*modifiers)
        {
            match key {
                KeyCode::ArrowDown => {
                    if !open {
                        self.set_open(cx, true);
                    }
                    if self.move_active(cx, 1) {
                        cx.stop_propagation();
                        return;
                    }
                }
                KeyCode::ArrowUp => {
                    if !open {
                        self.set_open(cx, true);
                    }
                    if self.move_active(cx, -1) {
                        cx.stop_propagation();
                        return;
                    }
                }
                KeyCode::Enter => {
                    if open && self.accept_active(cx) {
                        return;
                    }
                }
                KeyCode::Escape => {
                    if open {
                        self.set_open(cx, false);
                        cx.stop_propagation();
                        return;
                    }
                }
                _ => {}
            }
        }

        // Delegate to input for editing/selection/caret behavior.
        self.input.event(cx, event);

        // If we're open, keep the popover request up-to-date when models change.
        // (Typing updates the query model via `BoundTextInput`.)
        if open {
            self.sync_popover_request(cx);
        }

        // Toggle open on click.
        if let Event::Pointer(fret_core::PointerEvent::Up {
            position, button, ..
        }) = event
            && *button == fret_core::MouseButton::Left
            && self.last_bounds.contains(*position)
        {
            self.set_open(cx, !open);
            cx.stop_propagation();
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> fret_core::Size {
        self.sync_chrome(cx.theme());
        self.last_bounds = cx.bounds;

        cx.observe_model(self.items, Invalidation::Layout);
        cx.observe_model(self.query, Invalidation::Layout);
        cx.observe_model(self.selection, Invalidation::Paint);

        // Keep `active_filtered` stable as items/query change.
        let items_rev = cx.app.models().revision(self.items);
        let query_rev = cx.app.models().revision(self.query);
        if self.last_items_revision != items_rev || self.last_query_revision != query_rev {
            self.last_items_revision = items_rev;
            self.last_query_revision = query_rev;
            self.active_filtered = None;
        }

        // Detect open/close transitions to initialize popover selection.
        let open_now = cx
            .app
            .global::<PopoverService>()
            .and_then(|s| cx.window.and_then(|w| s.request(w)))
            .is_some_and(|(_, req)| req.owner == cx.node);
        if open_now && !self.last_opened {
            self.active_filtered = cx.app.models().get(self.selection).copied().unwrap_or(None);
        }
        self.last_opened = open_now;

        // `BoundTextInput` is the entire widget surface.
        let inner = self.input.layout(cx);
        let min_h = self.min_height.0.max(0.0);
        let h = inner.height.0.max(min_h).min(cx.available.height.0);
        fret_core::Size::new(inner.width, Px(h))
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.sync_chrome(cx.theme());
        self.last_bounds = cx.bounds;
        self.input.paint(cx);
    }

    fn semantics(&mut self, cx: &mut fret_ui::widget::SemanticsCx<'_, H>) {
        self.input.semantics(cx);
    }
}
