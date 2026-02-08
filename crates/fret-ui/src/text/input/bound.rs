use fret_core::{Event, KeyCode, Point, Rect, SemanticsRole, Size, TextStyle};
use fret_runtime::{CommandId, Model};

use super::TextInput;
use crate::widget::{
    CommandAvailability, CommandAvailabilityCx, CommandCx, EventCx, LayoutCx, PaintCx,
    PlatformTextInputCx, Widget,
};
use crate::{Invalidation, TextInputStyle, UiHost};

pub struct BoundTextInput {
    model: Model<String>,
    last_revision: Option<u64>,
    dirty_since_sync: bool,
    submit_command: Option<CommandId>,
    cancel_command: Option<CommandId>,
    enabled: bool,
    focusable: bool,
    input: TextInput,
}

impl BoundTextInput {
    pub fn new(model: Model<String>) -> Self {
        Self {
            model,
            last_revision: None,
            dirty_since_sync: false,
            submit_command: None,
            cancel_command: None,
            enabled: true,
            focusable: true,
            input: TextInput::new(),
        }
    }

    pub fn model_id(&self) -> fret_runtime::ModelId {
        self.model.id()
    }

    pub fn set_model(&mut self, model: Model<String>) {
        self.model = model;
        self.last_revision = None;
        self.dirty_since_sync = false;
    }

    pub fn with_submit_command(mut self, command: CommandId) -> Self {
        self.submit_command = Some(command);
        self
    }

    pub fn with_cancel_command(mut self, command: CommandId) -> Self {
        self.cancel_command = Some(command);
        self
    }

    pub fn set_submit_command(&mut self, command: Option<CommandId>) {
        self.submit_command = command;
    }

    pub fn set_cancel_command(&mut self, command: Option<CommandId>) {
        self.cancel_command = command;
    }

    pub fn with_chrome_style(mut self, style: TextInputStyle) -> Self {
        self.input.set_chrome_style(style);
        self
    }

    pub fn with_text_style(mut self, style: TextStyle) -> Self {
        self.input.set_text_style(style);
        self
    }

    pub fn set_placeholder(&mut self, placeholder: Option<std::sync::Arc<str>>) {
        self.input.set_placeholder(placeholder);
    }

    pub fn set_chrome_style(&mut self, style: TextInputStyle) {
        self.input.set_chrome_style(style);
    }

    pub fn set_text_style(&mut self, style: TextStyle) {
        self.input.set_text_style(style);
    }

    pub fn set_a11y_role(&mut self, role: SemanticsRole) {
        self.input.set_a11y_role(role);
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.input.set_enabled(enabled);
    }

    pub fn set_focusable(&mut self, focusable: bool) {
        self.focusable = focusable;
        self.input.set_focusable(focusable);
    }

    pub fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        self.input.queue_release_all_text_blobs();
        self.input.flush_pending_releases(services);
        self.input.text_metrics = None;
        self.input.prefix_metrics = None;
        self.input.suffix_metrics = None;
        self.input.preedit_metrics = None;
        self.input.caret_stops.clear();
    }

    pub fn semantics<H: UiHost>(&mut self, cx: &mut crate::widget::SemanticsCx<'_, H>) {
        self.input.semantics(cx);
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
            self.input.set_text(text.clone());
            self.dirty_since_sync = false;
        }
    }

    fn maybe_update_model<H: UiHost>(&mut self, app: &mut H) {
        let text = self.input.text().to_string();
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

impl<H: UiHost> Widget<H> for BoundTextInput {
    fn is_focusable(&self) -> bool {
        self.enabled && self.focusable
    }

    fn is_text_input(&self) -> bool {
        true
    }

    fn platform_text_input_snapshot(&self) -> Option<fret_runtime::WindowTextInputSnapshot> {
        <TextInput as Widget<H>>::platform_text_input_snapshot(&self.input)
    }

    fn platform_text_input_selected_range_utf16(&self) -> Option<fret_runtime::Utf16Range> {
        <TextInput as Widget<H>>::platform_text_input_selected_range_utf16(&self.input)
    }

    fn platform_text_input_marked_range_utf16(&self) -> Option<fret_runtime::Utf16Range> {
        <TextInput as Widget<H>>::platform_text_input_marked_range_utf16(&self.input)
    }

    fn platform_text_input_text_for_range_utf16(
        &self,
        range: fret_runtime::Utf16Range,
    ) -> Option<String> {
        <TextInput as Widget<H>>::platform_text_input_text_for_range_utf16(&self.input, range)
    }

    fn platform_text_input_bounds_for_range_utf16(
        &mut self,
        cx: &mut PlatformTextInputCx<'_, H>,
        range: fret_runtime::Utf16Range,
    ) -> Option<Rect> {
        <TextInput as Widget<H>>::platform_text_input_bounds_for_range_utf16(
            &mut self.input,
            cx,
            range,
        )
    }

    fn platform_text_input_character_index_for_point_utf16(
        &mut self,
        cx: &mut PlatformTextInputCx<'_, H>,
        point: Point,
    ) -> Option<u32> {
        <TextInput as Widget<H>>::platform_text_input_character_index_for_point_utf16(
            &mut self.input,
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
        let before = self.input.text().to_string();
        let changed = <TextInput as Widget<H>>::platform_text_input_replace_text_in_range_utf16(
            &mut self.input,
            cx,
            range,
            text,
        );
        if changed && self.input.text() != before {
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
        let before = self.input.text().to_string();
        let changed =
            <TextInput as Widget<H>>::platform_text_input_replace_and_mark_text_in_range_utf16(
                &mut self.input,
                cx,
                range,
                text,
                marked,
            );
        if changed && self.input.text() != before {
            self.dirty_since_sync = true;
            self.maybe_update_model(cx.app);
        }
        changed
    }

    fn command(&mut self, cx: &mut CommandCx<'_, H>, command: &CommandId) -> bool {
        if !self.enabled {
            return false;
        }
        let before = self.input.text().to_string();
        let handled = <TextInput as Widget<H>>::command(&mut self.input, cx, command);
        if handled && self.input.text() != before {
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
        command: &CommandId,
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
        match cmd {
            "text.copy" | "text.cut" => {
                if !clipboard_text {
                    return CommandAvailability::Blocked;
                }
                if self.input.has_selection() {
                    CommandAvailability::Available
                } else {
                    CommandAvailability::Blocked
                }
            }
            "text.paste" => {
                if !clipboard_text {
                    return CommandAvailability::Blocked;
                }
                CommandAvailability::Available
            }
            "text.select_all" | "text.clear" => {
                if !self.input.text().is_empty() {
                    CommandAvailability::Available
                } else {
                    CommandAvailability::Blocked
                }
            }
            _ => CommandAvailability::NotHandled,
        }
    }
    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        if cx.focus != Some(cx.node) {
            self.sync_from_model(cx.app, false);
        }

        if !self.enabled {
            return;
        }

        if cx.focus == Some(cx.node)
            && let Event::KeyDown { key, modifiers, .. } = event
            && !modifiers.shift
            && !modifiers.ctrl
            && !modifiers.alt
            && !modifiers.meta
        {
            match key {
                KeyCode::Enter => {
                    if let Some(cmd) = self.submit_command.clone() {
                        cx.dispatch_command(cmd);
                        cx.stop_propagation();
                        return;
                    }
                }
                KeyCode::Escape => {
                    if let Some(cmd) = self.cancel_command.clone() {
                        cx.dispatch_command(cmd);
                        cx.stop_propagation();
                        return;
                    }
                }
                _ => {}
            }
        }

        let before = self.input.text().to_string();
        self.input.event(cx, event);
        if self.input.text() != before {
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
        self.input.layout(cx)
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.input.paint(cx);
    }
}
