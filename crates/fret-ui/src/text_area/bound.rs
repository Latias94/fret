use fret_core::{Event, Point, Px, Rect, Size, TextStyle};
use fret_runtime::Model;

use crate::widget::{
    CommandAvailability, CommandAvailabilityCx, CommandCx, EventCx, LayoutCx, PaintCx,
    PlatformTextInputCx, Widget,
};
use crate::{Invalidation, UiHost};

use super::{TextArea, TextAreaStyle};

pub struct BoundTextArea {
    model: Model<String>,
    last_revision: Option<u64>,
    dirty_since_sync: bool,
    enabled: bool,
    focusable: bool,
    area: TextArea,
}

impl BoundTextArea {
    pub fn new(model: Model<String>) -> Self {
        Self {
            model,
            last_revision: None,
            dirty_since_sync: false,
            enabled: true,
            focusable: true,
            area: TextArea::default(),
        }
    }

    pub fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        self.area.queue_release_blob();
        self.area.flush_pending_releases(services);
        self.area.metrics = None;
        self.area.prepared_key = None;
    }

    pub fn with_text_style(mut self, style: TextStyle) -> Self {
        self.area.text_style = style;
        self.area.text_style_override = true;
        self.area.last_text_style_theme_revision = None;
        self.area.text_dirty = true;
        self
    }

    pub fn set_text_style(&mut self, style: TextStyle) {
        self.area.text_style = style;
        self.area.text_style_override = true;
        self.area.last_text_style_theme_revision = None;
        self.area.text_dirty = true;
    }

    pub fn with_min_height(mut self, min_height: Px) -> Self {
        self.area.min_height = min_height;
        self
    }

    pub fn set_min_height(&mut self, min_height: Px) {
        self.area.min_height = min_height;
    }

    pub fn with_style(mut self, style: TextAreaStyle) -> Self {
        self.area.style = style;
        self.area.style_override = true;
        self.area.last_theme_revision = None;
        self
    }

    pub fn set_style(&mut self, style: TextAreaStyle) {
        self.area.style = style;
        self.area.style_override = true;
        self.area.last_theme_revision = None;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.area.set_enabled(enabled);
    }

    pub fn set_focusable(&mut self, focusable: bool) {
        self.focusable = focusable;
        self.area.set_focusable(focusable);
    }

    pub fn model_id(&self) -> fret_runtime::ModelId {
        self.model.id()
    }

    pub fn set_model(&mut self, model: Model<String>) {
        if self.model.id() == model.id() {
            return;
        }
        self.model = model;
        self.last_revision = None;
        self.dirty_since_sync = false;
    }

    fn sync_from_model<H: UiHost>(&mut self, app: &H, force: bool) {
        let revision = app.models().revision(&self.model);
        if revision == self.last_revision {
            return;
        }
        self.last_revision = revision;

        let Some(text) = app.models().get_cloned(&self.model) else {
            return;
        };

        if force || !self.dirty_since_sync {
            self.area.set_text(text.clone());
            self.dirty_since_sync = false;
        }
    }

    fn maybe_update_model<H: UiHost>(&mut self, app: &mut H) {
        let text = self.area.text.clone();
        if app
            .models_mut()
            .update(&self.model, move |v| *v = text)
            .is_ok()
        {
            self.dirty_since_sync = false;
            self.last_revision = app.models().revision(&self.model);
        }
    }
}

impl<H: UiHost> Widget<H> for BoundTextArea {
    fn is_focusable(&self) -> bool {
        self.enabled && self.focusable
    }

    fn is_text_input(&self) -> bool {
        true
    }

    fn platform_text_input_snapshot(&self) -> Option<fret_runtime::WindowTextInputSnapshot> {
        <TextArea as Widget<H>>::platform_text_input_snapshot(&self.area)
    }

    fn platform_text_input_selected_range_utf16(&self) -> Option<fret_runtime::Utf16Range> {
        <TextArea as Widget<H>>::platform_text_input_selected_range_utf16(&self.area)
    }

    fn platform_text_input_marked_range_utf16(&self) -> Option<fret_runtime::Utf16Range> {
        <TextArea as Widget<H>>::platform_text_input_marked_range_utf16(&self.area)
    }

    fn platform_text_input_text_for_range_utf16(
        &self,
        range: fret_runtime::Utf16Range,
    ) -> Option<String> {
        <TextArea as Widget<H>>::platform_text_input_text_for_range_utf16(&self.area, range)
    }

    fn platform_text_input_bounds_for_range_utf16(
        &mut self,
        cx: &mut PlatformTextInputCx<'_, H>,
        range: fret_runtime::Utf16Range,
    ) -> Option<Rect> {
        <TextArea as Widget<H>>::platform_text_input_bounds_for_range_utf16(
            &mut self.area,
            cx,
            range,
        )
    }

    fn platform_text_input_character_index_for_point_utf16(
        &mut self,
        cx: &mut PlatformTextInputCx<'_, H>,
        point: Point,
    ) -> Option<u32> {
        <TextArea as Widget<H>>::platform_text_input_character_index_for_point_utf16(
            &mut self.area,
            cx,
            point,
        )
    }

    fn platform_text_input_replace_text_in_range_utf16(
        &mut self,
        cx: &mut PlatformTextInputCx<'_, H>,
        range: fret_runtime::Utf16Range,
        text: &str,
    ) -> bool {
        let before = self.area.text.clone();
        let changed = <TextArea as Widget<H>>::platform_text_input_replace_text_in_range_utf16(
            &mut self.area,
            cx,
            range,
            text,
        );
        if changed && self.area.text != before {
            self.dirty_since_sync = true;
            self.maybe_update_model(cx.app);
        }
        changed
    }

    fn platform_text_input_replace_and_mark_text_in_range_utf16(
        &mut self,
        cx: &mut PlatformTextInputCx<'_, H>,
        range: fret_runtime::Utf16Range,
        text: &str,
        marked: Option<fret_runtime::Utf16Range>,
    ) -> bool {
        let before = self.area.text.clone();
        let changed =
            <TextArea as Widget<H>>::platform_text_input_replace_and_mark_text_in_range_utf16(
                &mut self.area,
                cx,
                range,
                text,
                marked,
            );
        if changed && self.area.text != before {
            self.dirty_since_sync = true;
            self.maybe_update_model(cx.app);
        }
        changed
    }

    fn command(&mut self, cx: &mut CommandCx<'_, H>, command: &fret_runtime::CommandId) -> bool {
        if !self.enabled {
            return false;
        }
        let before = self.area.text.clone();
        let handled = <TextArea as Widget<H>>::command(&mut self.area, cx, command);
        if handled && self.area.text != before {
            self.dirty_since_sync = true;
            self.maybe_update_model(cx.app);
            cx.invalidate_self(Invalidation::Layout);
            cx.invalidate_self(Invalidation::Paint);
            cx.request_redraw();
        }
        handled
    }

    fn command_availability(
        &self,
        cx: &mut CommandAvailabilityCx<'_, H>,
        command: &fret_runtime::CommandId,
    ) -> CommandAvailability {
        if !self.enabled {
            return CommandAvailability::NotHandled;
        }
        if cx.focus != Some(cx.node) {
            return CommandAvailability::NotHandled;
        }

        let cmd = match command.as_str() {
            "edit.copy" => "text.copy",
            "edit.cut" => "text.cut",
            "edit.paste" => "text.paste",
            "edit.select_all" => "text.select_all",
            other => other,
        };
        if !cmd.starts_with("text.") {
            return CommandAvailability::NotHandled;
        }

        let clipboard_text = cx.input_ctx.caps.clipboard.text;
        let (start, end) = self.area.selection_range();
        let has_selection = start != end;

        match cmd {
            "text.copy" | "text.cut" => {
                if !clipboard_text {
                    return CommandAvailability::Blocked;
                }
                has_selection
                    .then_some(CommandAvailability::Available)
                    .unwrap_or(CommandAvailability::Blocked)
            }
            "text.paste" => {
                if !clipboard_text {
                    return CommandAvailability::Blocked;
                }
                CommandAvailability::Available
            }
            "text.select_all" | "text.clear" => (!self.area.text().is_empty())
                .then_some(CommandAvailability::Available)
                .unwrap_or(CommandAvailability::Blocked),
            _ => CommandAvailability::NotHandled,
        }
    }
    fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        <TextArea as Widget<H>>::cleanup_resources(&mut self.area, services);
    }

    fn semantics(&mut self, cx: &mut crate::widget::SemanticsCx<'_, H>) {
        self.area.semantics(cx);
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        if cx.focus != Some(cx.node) {
            self.sync_from_model(cx.app, false);
        }

        if !self.enabled {
            return;
        }

        let before = self.area.text.clone();
        self.area.event(cx, event);
        if self.area.text != before {
            self.dirty_since_sync = true;
            self.maybe_update_model(cx.app);
            cx.invalidate_self(Invalidation::Layout);
            cx.invalidate_self(Invalidation::Paint);
            cx.request_redraw();
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.observe_model(&self.model, Invalidation::Layout);
        cx.observe_model(&self.model, Invalidation::Paint);
        let force = !self.dirty_since_sync;
        self.sync_from_model(cx.app, force);
        self.area.layout(cx)
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.area.paint(cx);
    }
}
