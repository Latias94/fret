use super::frame::ElementInstance;
use super::frame::element_record_for_node;
use super::prelude::*;
use crate::widget::{CommandAvailability, CommandAvailabilityCx, CommandCx, MeasureCx};
use fret_runtime::CommandId;

mod event;
mod layout;
mod measure;
mod paint;
mod semantics;

#[derive(Debug, Default, Clone)]
struct TextCache {
    blob: Option<fret_core::TextBlobId>,
    metrics: Option<TextMetrics>,
    prepared_scale_factor_bits: Option<u32>,
    measured_scale_factor_bits: Option<u32>,
    last_text: Option<std::sync::Arc<str>>,
    last_rich: Option<fret_core::AttributedText>,
    last_style: Option<TextStyle>,
    last_wrap: Option<fret_core::TextWrap>,
    last_overflow: Option<TextOverflow>,
    last_width: Option<Px>,
    last_measure_width: Option<Px>,
    last_font_stack_key: Option<u64>,
}

#[derive(Debug, Default, Clone)]
struct SvgCache {
    key: Option<SvgCacheKey>,
    svg: Option<fret_core::SvgId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SvgCacheKey {
    Static { ptr: usize, len: usize },
    Bytes { ptr: usize, len: usize },
}

pub(super) struct ElementHostWidget {
    element: GlobalElementId,
    text_cache: TextCache,
    svg_cache: SvgCache,
    canvas_cache: crate::canvas::CanvasCache,
    render_transform: Option<fret_core::Transform2D>,
    scroll_child_transform: Option<ScrollChildTransform>,
    hit_testable: bool,
    hit_test_children: bool,
    focus_traversal_children: bool,
    semantics_present: bool,
    semantics_children: bool,
    is_focusable: bool,
    is_text_input: bool,
    can_scroll_descendant: bool,
    clips_hit_test: bool,
    clip_hit_test_corner_radii: Option<fret_core::Corners>,
    text_input: Option<BoundTextInput>,
    text_area: Option<crate::text_area::BoundTextArea>,
    resizable_panel_group: Option<crate::resizable_panel_group::BoundResizablePanelGroup>,
}

#[derive(Debug, Clone)]
struct ScrollChildTransform {
    handle: crate::scroll::ScrollHandle,
    axis: crate::element::ScrollAxis,
}

impl ElementHostWidget {
    pub(super) fn new(element: GlobalElementId) -> Self {
        Self {
            element,
            text_cache: TextCache::default(),
            svg_cache: SvgCache::default(),
            canvas_cache: crate::canvas::CanvasCache::default(),
            render_transform: None,
            scroll_child_transform: None,
            hit_testable: true,
            hit_test_children: true,
            focus_traversal_children: true,
            semantics_present: true,
            semantics_children: true,
            is_focusable: false,
            is_text_input: false,
            can_scroll_descendant: false,
            clips_hit_test: true,
            clip_hit_test_corner_radii: None,
            text_input: None,
            text_area: None,
            resizable_panel_group: None,
        }
    }

    fn resolve_svg_for_icon(
        &mut self,
        services: &mut dyn fret_core::UiServices,
        source: &crate::SvgSource,
    ) -> fret_core::SvgId {
        match source {
            crate::SvgSource::Id(id) => *id,
            crate::SvgSource::Static(bytes) => {
                let key = SvgCacheKey::Static {
                    ptr: bytes.as_ptr() as usize,
                    len: bytes.len(),
                };
                if self.svg_cache.key == Some(key)
                    && let Some(svg) = self.svg_cache.svg
                {
                    return svg;
                }

                let svg = services.svg().register_svg(bytes);
                if let Some(old) = self.svg_cache.svg.replace(svg) {
                    let _ = services.svg().unregister_svg(old);
                }
                self.svg_cache.key = Some(key);
                svg
            }
            crate::SvgSource::Bytes(bytes) => {
                let key = SvgCacheKey::Bytes {
                    ptr: bytes.as_ptr() as usize,
                    len: bytes.len(),
                };
                if self.svg_cache.key == Some(key)
                    && let Some(svg) = self.svg_cache.svg
                {
                    return svg;
                }

                let svg = services.svg().register_svg(bytes);
                if let Some(old) = self.svg_cache.svg.replace(svg) {
                    let _ = services.svg().unregister_svg(old);
                }
                self.svg_cache.key = Some(key);
                svg
            }
        }
    }

    fn instance<H: UiHost>(
        &self,
        app: &mut H,
        window: AppWindowId,
        node: NodeId,
    ) -> Option<ElementInstance> {
        element_record_for_node(app, window, node).map(|r| r.instance)
    }
}

impl<H: UiHost> Widget<H> for ElementHostWidget {
    fn command(&mut self, cx: &mut CommandCx<'_, H>, command: &CommandId) -> bool {
        let Some(window) = cx.window else {
            return false;
        };
        let Some(instance) = self.instance(cx.app, window, cx.node) else {
            return false;
        };

        let hook = crate::elements::with_element_state(
            &mut *cx.app,
            window,
            self.element,
            crate::action::CommandActionHooks::default,
            |hooks| hooks.on_command.clone(),
        );
        if let Some(hook) = hook {
            struct CommandHookHost<'a, H: UiHost> {
                app: &'a mut H,
                window: AppWindowId,
                element: crate::GlobalElementId,
                requested_focus: &'a mut Option<NodeId>,
            }

            impl<H: UiHost> crate::action::UiActionHost for CommandHookHost<'_, H> {
                fn models_mut(&mut self) -> &mut fret_runtime::ModelStore {
                    self.app.models_mut()
                }

                fn push_effect(&mut self, effect: Effect) {
                    match effect {
                        Effect::SetTimer {
                            window: Some(window),
                            token,
                            ..
                        } if window == self.window => {
                            crate::elements::record_timer_target(
                                &mut *self.app,
                                window,
                                token,
                                self.element,
                            );
                        }
                        Effect::CancelTimer { token } => {
                            crate::elements::clear_timer_target(&mut *self.app, self.window, token);
                        }
                        _ => {}
                    }
                    self.app.push_effect(effect);
                }

                fn request_redraw(&mut self, window: AppWindowId) {
                    self.app.request_redraw(window);
                }

                fn next_timer_token(&mut self) -> fret_runtime::TimerToken {
                    self.app.next_timer_token()
                }
            }

            impl<H: UiHost> crate::action::UiFocusActionHost for CommandHookHost<'_, H> {
                fn request_focus(&mut self, target: crate::GlobalElementId) {
                    let Some(node) = crate::elements::with_window_state(
                        &mut *self.app,
                        self.window,
                        |window_state| window_state.node_entry(target).map(|e| e.node),
                    ) else {
                        return;
                    };
                    *self.requested_focus = Some(node);
                }
            }

            let mut host = CommandHookHost {
                app: &mut *cx.app,
                window,
                element: self.element,
                requested_focus: &mut cx.requested_focus,
            };
            if hook(
                &mut host,
                crate::action::ActionCx {
                    window,
                    target: self.element,
                },
                command.clone(),
            ) {
                cx.stop_propagation();
                return true;
            }
        }

        match instance {
            ElementInstance::SelectableText(props) => {
                if cx.focus != Some(cx.node) {
                    return false;
                }
                if matches!(command.as_str(), "text.copy" | "edit.copy")
                    && !cx.input_ctx.caps.clipboard.text
                {
                    return false;
                }
                let (outcome, range) = crate::elements::with_element_state(
                    &mut *cx.app,
                    window,
                    self.element,
                    crate::element::SelectableTextState::default,
                    |state| {
                        let outcome = crate::text_surface::apply_selectable_text_command(
                            &props.rich.text,
                            state,
                            command.as_str(),
                        );
                        let range = match outcome {
                            crate::text_surface::SelectableTextCommandOutcome::Handled {
                                copy_range: Some(r),
                                ..
                            } => Some(r),
                            _ => None,
                        };
                        (outcome, range)
                    },
                );

                let crate::text_surface::SelectableTextCommandOutcome::Handled {
                    needs_repaint,
                    copy_range: _,
                } = outcome
                else {
                    return false;
                };

                if let Some((start, end)) = range
                    && end <= props.rich.text.len()
                    && let Some(sel) = props.rich.text.get(start..end)
                {
                    cx.app.push_effect(Effect::ClipboardSetText {
                        text: sel.to_string(),
                    });
                }

                if needs_repaint {
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                }

                cx.stop_propagation();
                true
            }
            ElementInstance::TextInput(props) => {
                let model = props.model.clone();
                let model_id = model.id();
                match self.text_input.as_mut() {
                    None => {
                        self.text_input = Some(crate::text_input::BoundTextInput::new(model));
                    }
                    Some(input) => {
                        if input.model_id() != model_id {
                            input.set_model(model);
                        }
                    }
                }
                let input = self.text_input.as_mut().expect("text input");
                input.set_chrome_style(props.chrome);
                input.set_text_style(props.text_style);
                input.set_placeholder(props.placeholder);
                input.set_submit_command(props.submit_command);
                input.set_cancel_command(props.cancel_command);
                <crate::text_input::BoundTextInput as Widget<H>>::command(input, cx, command)
            }
            ElementInstance::TextArea(props) => {
                let model = props.model.clone();
                let model_id = model.id();
                match self.text_area.as_mut() {
                    None => {
                        self.text_area = Some(crate::text_area::BoundTextArea::new(model));
                    }
                    Some(area) => {
                        if area.model_id() != model_id {
                            area.set_model(model);
                        }
                    }
                }
                let area = self.text_area.as_mut().expect("text area");
                area.set_style(props.chrome);
                area.set_text_style(props.text_style);
                area.set_min_height(props.min_height);
                <crate::text_area::BoundTextArea as Widget<H>>::command(area, cx, command)
            }
            ElementInstance::FocusScope(props) if props.trap_focus => {
                let forward = match command.as_str() {
                    "focus.next" => Some(true),
                    "focus.previous" => Some(false),
                    _ => None,
                };
                let Some(forward) = forward else {
                    return false;
                };

                cx.tree
                    .focus_traverse_in_roots(cx.app, &[cx.node], forward, Some(cx.node));
                cx.stop_propagation();
                true
            }
            _ => false,
        }
    }

    fn command_availability(
        &self,
        cx: &mut CommandAvailabilityCx<'_, H>,
        command: &CommandId,
    ) -> CommandAvailability {
        let Some(window) = cx.window else {
            return CommandAvailability::NotHandled;
        };
        let Some(instance) = self.instance(cx.app, window, cx.node) else {
            return CommandAvailability::NotHandled;
        };

        let hook = crate::elements::with_element_state(
            &mut *cx.app,
            window,
            self.element,
            crate::action::CommandAvailabilityActionHooks::default,
            |hooks| hooks.on_command_availability.clone(),
        );
        if let Some(hook) = hook {
            struct AvailabilityHookHost<'a, H: UiHost> {
                app: &'a mut H,
            }

            impl<H: UiHost> crate::action::UiCommandAvailabilityActionHost for AvailabilityHookHost<'_, H> {
                fn models_mut(&mut self) -> &mut fret_runtime::ModelStore {
                    self.app.models_mut()
                }
            }

            let focus_in_subtree = cx
                .focus
                .map(|focus| cx.tree.is_descendant(cx.node, focus))
                .unwrap_or(false);
            let mut host = AvailabilityHookHost { app: &mut *cx.app };
            let availability = hook(
                &mut host,
                crate::action::CommandAvailabilityActionCx {
                    window,
                    target: self.element,
                    node: cx.node,
                    focus: cx.focus,
                    focus_in_subtree,
                    input_ctx: cx.input_ctx.clone(),
                },
                command.clone(),
            );
            if availability != CommandAvailability::NotHandled {
                return availability;
            }
        }

        match instance {
            ElementInstance::SelectableText(props) => {
                if cx.focus != Some(cx.node) {
                    return CommandAvailability::NotHandled;
                }
                match command.as_str() {
                    "text.select_all" | "edit.select_all" => {
                        // A focused selectable text surface should always be able to select its full
                        // content (if non-empty), even though it is not an editable text input.
                        let has_any_text = !props.rich.text.is_empty();
                        if has_any_text {
                            return CommandAvailability::Available;
                        }
                        return CommandAvailability::Blocked;
                    }
                    "text.copy" | "edit.copy" => {
                        if !cx.input_ctx.caps.clipboard.text {
                            return CommandAvailability::Blocked;
                        }
                    }
                    _ => return CommandAvailability::NotHandled,
                }
                let has_selection = crate::elements::with_element_state(
                    &mut *cx.app,
                    window,
                    self.element,
                    crate::element::SelectableTextState::default,
                    |state| state.selection_anchor != state.caret,
                );
                if has_selection {
                    CommandAvailability::Available
                } else {
                    CommandAvailability::Blocked
                }
            }
            ElementInstance::TextInput(_props) => self
                .text_input
                .as_ref()
                .map(|input| {
                    <crate::text_input::BoundTextInput as Widget<H>>::command_availability(
                        input, cx, command,
                    )
                })
                .unwrap_or(CommandAvailability::NotHandled),
            ElementInstance::TextArea(_props) => self
                .text_area
                .as_ref()
                .map(|area| {
                    <crate::text_area::BoundTextArea as Widget<H>>::command_availability(
                        area, cx, command,
                    )
                })
                .unwrap_or(CommandAvailability::NotHandled),
            ElementInstance::FocusScope(props) if props.trap_focus => match command.as_str() {
                "focus.next" | "focus.previous" => CommandAvailability::Available,
                _ => CommandAvailability::NotHandled,
            },
            _ => CommandAvailability::NotHandled,
        }
    }

    fn clips_hit_test(&self, _bounds: Rect) -> bool {
        self.clips_hit_test
    }

    fn render_transform(&self, _bounds: Rect) -> Option<fret_core::Transform2D> {
        self.render_transform
    }

    fn children_render_transform(&self, _bounds: Rect) -> Option<fret_core::Transform2D> {
        let scroll = self.scroll_child_transform.as_ref()?;
        let offset = scroll.handle.offset();
        let offset_x = if scroll.axis.scroll_x() {
            Px(-offset.x.0)
        } else {
            Px(0.0)
        };
        let offset_y = if scroll.axis.scroll_y() {
            Px(-offset.y.0)
        } else {
            Px(0.0)
        };

        let transform = fret_core::Transform2D::translation(Point::new(offset_x, offset_y));
        (transform != fret_core::Transform2D::IDENTITY).then_some(transform)
    }

    fn clip_hit_test_corner_radii(&self, _bounds: Rect) -> Option<fret_core::Corners> {
        self.clip_hit_test_corner_radii
    }

    fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
        self.hit_testable
    }

    fn hit_test_children(&self, _bounds: Rect, _position: Point) -> bool {
        if !self.hit_test_children {
            return false;
        }
        true
    }

    fn focus_traversal_children(&self) -> bool {
        self.focus_traversal_children
    }

    fn semantics_present(&self) -> bool {
        self.semantics_present
    }

    fn semantics_children(&self) -> bool {
        self.semantics_children
    }

    fn is_focusable(&self) -> bool {
        self.is_focusable
    }

    fn is_text_input(&self) -> bool {
        self.is_text_input
    }

    fn can_scroll_descendant_into_view(&self) -> bool {
        self.can_scroll_descendant
    }

    fn scroll_descendant_into_view(
        &mut self,
        cx: &mut crate::widget::ScrollIntoViewCx<'_, H>,
        descendant_bounds: Rect,
    ) -> crate::widget::ScrollIntoViewResult {
        let Some(window) = cx.window else {
            return crate::widget::ScrollIntoViewResult::NotHandled;
        };
        let Some(instance) = self.instance(cx.app, window, cx.node) else {
            return crate::widget::ScrollIntoViewResult::NotHandled;
        };

        match instance {
            ElementInstance::Scroll(props) => {
                let handle = if let Some(handle) = props.scroll_handle.as_ref() {
                    handle.clone()
                } else {
                    crate::elements::with_element_state(
                        &mut *cx.app,
                        window,
                        self.element,
                        crate::element::ScrollState::default,
                        |state| state.scroll_handle.clone(),
                    )
                };

                crate::widget::ScrollIntoViewResult::Handled {
                    did_scroll: {
                        // Scroll content is translated at paint/input time (children-only transform),
                        // so `descendant_bounds` is expressed in the unscrolled content coordinate
                        // space. Map the viewport into that same space before computing the delta.
                        let offset = handle.offset();
                        let viewport_in_content = Rect::new(
                            Point::new(
                                Px(cx.bounds.origin.x.0 + offset.x.0),
                                Px(cx.bounds.origin.y.0 + offset.y.0),
                            ),
                            cx.bounds.size,
                        );
                        scroll_handle_into_view_y(&handle, viewport_in_content, descendant_bounds)
                    },
                }
            }
            ElementInstance::VirtualList(props) => crate::widget::ScrollIntoViewResult::Handled {
                did_scroll: scroll_handle_into_view_y(
                    props.scroll_handle.base_handle(),
                    cx.bounds,
                    descendant_bounds,
                ),
            },
            _ => crate::widget::ScrollIntoViewResult::NotHandled,
        }
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        self.event_impl(cx, event);
    }

    fn event_observer(&mut self, cx: &mut crate::widget::ObserverCx<'_, H>, event: &Event) {
        self.event_observer_impl(cx, event);
    }

    fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        if let Some(blob) = self.text_cache.blob.take() {
            services.text().release(blob);
        }
        self.text_cache.prepared_scale_factor_bits = None;
        self.text_cache.metrics = None;
        self.text_cache.last_text = None;
        self.text_cache.last_rich = None;
        if let Some(svg) = self.svg_cache.svg.take() {
            let _ = services.svg().unregister_svg(svg);
        }
        self.svg_cache.key = None;
        self.canvas_cache.cleanup_resources(services);
        if let Some(input) = self.text_input.as_mut() {
            input.cleanup_resources(services);
        }
        if let Some(area) = self.text_area.as_mut() {
            area.cleanup_resources(services);
        }
        if let Some(group) = self.resizable_panel_group.as_mut() {
            group.cleanup_resources(services);
        }
    }

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        self.semantics_impl(cx);
    }

    fn measure(&mut self, cx: &mut MeasureCx<'_, H>) -> Size {
        self.measure_impl(cx)
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        self.layout_impl(cx)
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.paint_impl(cx);
    }
}

fn scroll_handle_into_view_y(
    handle: &crate::scroll::ScrollHandle,
    viewport: Rect,
    child: Rect,
) -> bool {
    let viewport_h = viewport.size.height.0.max(0.0);
    if viewport_h <= 0.0 {
        return false;
    }

    let view_top = viewport.origin.y.0;
    let view_bottom = view_top + viewport_h;
    let child_top = child.origin.y.0;
    let child_bottom = child_top + child.size.height.0.max(0.0);

    let delta = if child_top < view_top {
        child_top - view_top
    } else if child_bottom > view_bottom {
        child_bottom - view_bottom
    } else {
        0.0
    };

    if delta.abs() <= 0.01 {
        return false;
    }

    let prev = handle.offset();
    handle.set_offset(Point::new(prev.x, Px(prev.y.0 + delta)));

    let next = handle.offset();
    (prev.y.0 - next.y.0).abs() > 0.01
}
