//! Node graph controls overlay (UI-only).

use fret_core::{
    Color, Corners, CursorIcon, DrawOrder, Edges, Event, MouseButton, Point, Px, Rect, SceneOp,
    Size, TextBlobId, TextConstraints, TextOverflow, TextWrap,
};
use fret_runtime::Model;
use fret_ui::{UiHost, retained_bridge::*};

use crate::io::{NodeGraphEditorConfig, NodeGraphViewState};
use crate::ui::NodeGraphStyle;

use super::OverlayPlacement;
use super::controls_layout::{
    ControlsLayout, compute_controls_layout, controls_button_at, controls_panel_size,
};
use super::controls_policy::{
    ControlsButton, NodeGraphControlsBindings, controls_button_a11y_label, controls_button_label,
    controls_buttons, resolve_controls_command_id,
};
use super::panel_button_paint::paint_panel_button;
use super::panel_item_state::{
    clear_panel_item_state, panel_item_visual_state, select_panel_keyboard_item,
};
use super::panel_navigation_policy::{PanelKeyboardAction, panel_keyboard_action};
use super::panel_pointer_policy::{begin_panel_press, release_panel_press, sync_panel_hover};

pub struct NodeGraphControlsOverlay {
    canvas_node: fret_core::NodeId,
    view_state: Model<NodeGraphViewState>,
    editor_config: Model<NodeGraphEditorConfig>,
    style: NodeGraphStyle,
    bindings: NodeGraphControlsBindings,
    hovered: Option<ControlsButton>,
    pressed: Option<ControlsButton>,
    keyboard_active: Option<ControlsButton>,
    text_blobs: Vec<TextBlobId>,
    placement: OverlayPlacement,
}

impl NodeGraphControlsOverlay {
    pub fn new(
        canvas_node: fret_core::NodeId,
        view_state: Model<NodeGraphViewState>,
        editor_config: Model<NodeGraphEditorConfig>,
        style: NodeGraphStyle,
    ) -> Self {
        Self {
            canvas_node,
            view_state,
            editor_config,
            style,
            bindings: NodeGraphControlsBindings::default(),
            hovered: None,
            pressed: None,
            keyboard_active: None,
            text_blobs: Vec::new(),
            placement: OverlayPlacement::FloatingInCanvas,
        }
    }

    pub fn with_bindings(mut self, bindings: NodeGraphControlsBindings) -> Self {
        self.bindings = bindings;
        self
    }

    /// Switches to "panel bounds" mode for `NodeGraphPanel` composition.
    pub fn in_panel_bounds(mut self) -> Self {
        self.placement = OverlayPlacement::PanelBounds;
        self
    }

    fn compute_layout(&self, bounds: Rect) -> ControlsLayout {
        compute_controls_layout(&self.style, self.placement, bounds)
    }

    fn button_at(&self, bounds: Rect, position: Point) -> Option<ControlsButton> {
        let layout = self.compute_layout(bounds);
        controls_button_at(&layout, position)
    }

    fn dispatch_button<H: UiHost>(&self, cx: &mut EventCx<'_, H>, btn: ControlsButton) {
        cx.request_focus(self.canvas_node);

        if let Some(id) = resolve_controls_command_id(&self.bindings, btn) {
            cx.dispatch_command(id);
        }
    }

    fn clear_item_state(&mut self) {
        clear_panel_item_state(
            &mut self.hovered,
            &mut self.pressed,
            &mut self.keyboard_active,
        );
    }

    fn activate_button<H: UiHost>(&mut self, cx: &mut EventCx<'_, H>, btn: ControlsButton) {
        self.clear_item_state();
        self.dispatch_button(cx, btn);
    }
}

impl<H: UiHost> Widget<H> for NodeGraphControlsOverlay {
    fn is_focusable(&self) -> bool {
        true
    }

    fn measure(&mut self, _cx: &mut MeasureCx<'_, H>) -> Size {
        controls_panel_size(&self.style)
    }

    fn hit_test(&self, bounds: Rect, position: Point) -> bool {
        let layout = self.compute_layout(bounds);
        layout.panel.contains(position)
    }

    fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        for id in self.text_blobs.drain(..) {
            services.text().release(id);
        }
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        match event {
            Event::KeyDown { key, repeat: _, .. } => {
                match panel_keyboard_action(*key, self.keyboard_active, controls_buttons()) {
                    PanelKeyboardAction::Select(button) => {
                        select_panel_keyboard_item(
                            &mut self.hovered,
                            &mut self.pressed,
                            &mut self.keyboard_active,
                            button,
                        );
                        crate::ui::retained_event_tail::finish_paint_event(cx);
                    }
                    PanelKeyboardAction::Activate(button) => {
                        self.activate_button(cx, button);
                        crate::ui::retained_event_tail::finish_paint_event(cx);
                    }
                    PanelKeyboardAction::FocusCanvas => {
                        self.clear_item_state();
                        crate::ui::retained_event_tail::focus_canvas_and_finish_paint_event(
                            cx,
                            self.canvas_node,
                        );
                    }
                    PanelKeyboardAction::Ignore => {}
                }
            }
            Event::Pointer(fret_core::PointerEvent::Move { position, .. }) => {
                let hovered = self.button_at(cx.bounds, *position);
                if hovered.is_some() {
                    cx.set_cursor_icon(CursorIcon::Pointer);
                }
                if sync_panel_hover(&mut self.hovered, hovered) {
                    crate::ui::retained_event_tail::request_paint_repaint(cx);
                }
            }
            Event::Pointer(fret_core::PointerEvent::Down {
                position, button, ..
            }) => {
                if *button != MouseButton::Left {
                    return;
                }
                let button = self.button_at(cx.bounds, *position);
                begin_panel_press(cx, &mut self.keyboard_active, &mut self.pressed, button);
            }
            Event::Pointer(fret_core::PointerEvent::Up {
                position, button, ..
            }) => {
                if *button != MouseButton::Left {
                    return;
                }
                let released_on = self.button_at(cx.bounds, *position);
                let release = release_panel_press(&mut self.pressed, released_on);
                cx.release_pointer_capture();
                if release.had_pressed {
                    crate::ui::retained_event_tail::finish_paint_event(cx);
                }
                let Some(pressed) = release.activate else {
                    return;
                };
                self.activate_button(cx, pressed);
            }
            _ => {}
        }
    }

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        cx.set_role(fret_core::SemanticsRole::Panel);
        cx.set_label("Controls");
        cx.set_test_id("node_graph.controls");
        cx.set_focusable(true);

        let active = self
            .keyboard_active
            .or_else(|| controls_buttons().first().copied())
            .expect("controls buttons");
        cx.set_value(controls_button_a11y_label(active));
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        for id in self.text_blobs.drain(..) {
            cx.services.text().release(id);
        }

        cx.observe_model(&self.view_state, Invalidation::Paint);
        cx.observe_model(&self.editor_config, Invalidation::Paint);
        let mode = self
            .editor_config
            .read_ref(cx.app, |state| state.interaction.connection_mode)
            .expect("controls overlay editor-config model must stay readable");

        let layout = self.compute_layout(cx.bounds);
        let bg = self.style.paint.context_menu_background;
        let border = self.style.paint.context_menu_border;
        let corner = self.style.paint.context_menu_corner_radius;

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(21_000),
            rect: layout.panel,
            background: fret_core::Paint::Solid(bg).into(),

            border: Edges::all(Px(1.0)),
            border_paint: fret_core::Paint::Solid(border).into(),

            corner_radii: Corners::all(Px(corner)),
        });

        let text_style = self.style.paint.controls_text_style.clone();
        let constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: cx.scale_factor,
        };

        for (btn, rect) in &layout.buttons {
            let state = panel_item_visual_state(
                *btn,
                self.hovered,
                self.pressed,
                self.keyboard_active,
                cx.focus == Some(cx.node),
                true,
            );
            let button_bg = if state.pressed {
                self.style.paint.controls_active_background
            } else if state.hovered || state.keyboard {
                self.style.paint.controls_hover_background
            } else {
                Color::TRANSPARENT
            };

            let label = controls_button_label(*btn, mode);
            paint_panel_button(
                cx,
                &mut self.text_blobs,
                *rect,
                label,
                &text_style,
                constraints,
                button_bg,
                self.style.paint.controls_text,
                corner,
                DrawOrder(21_001),
                DrawOrder(21_002),
            );
        }
    }
}
