use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use fret_core::{AppWindowId, Color, Edges, MouseButton, Px, SemanticsRole, TextStyle};
use fret_runtime::Model;
use fret_ui::action::{PointerDownCx, PointerMoveCx, PointerUpCx};
use fret_ui::element::{
    ColumnProps, HoverRegionProps, InsetStyle, LayoutStyle, Length, PointerRegionProps,
    PositionStyle, PressableProps, RowProps, SemanticsProps, SizeStyle, TextInputProps, TextProps,
};
use fret_ui::elements::ElementContext;
use fret_ui::{TextInputStyle, ThemeSnapshot, UiHost};

use crate::core::{Graph, NodeId};
use crate::ops::GraphTransaction;
use crate::ui::editors::chrome::{
    PortalSmallButtonUi, render_pressable_small_button, render_small_button,
};
use crate::ui::portal::{
    NodeGraphPortalCommandHandler, NodeGraphPortalNodeLayout, PortalCommandOutcome,
    PortalTextCommand, PortalTextStepMode, portal_cancel_text_command,
    portal_step_text_command_with_mode, portal_submit_text_command,
};
use crate::ui::style::NodeGraphStyle;

#[derive(Debug, Clone)]
pub struct PortalNumberEditorUi {
    pub max_width: f32,
    pub gap: f32,
    pub button: PortalSmallButtonUi,
    pub error_color: Color,
    pub error_text_style: TextStyle,
}

impl PortalNumberEditorUi {
    pub fn from_theme(theme: ThemeSnapshot) -> Self {
        let font_size = theme.metric_required("metric.font.size").0;
        Self {
            max_width: 180.0,
            gap: 6.0,
            button: PortalSmallButtonUi::from_theme(theme),
            error_color: theme.color_required("destructive"),
            error_text_style: TextStyle {
                size: Px((font_size - 1.0).max(10.0)),
                ..TextStyle::default()
            },
        }
    }
}

impl Default for PortalNumberEditorUi {
    fn default() -> Self {
        Self {
            max_width: 180.0,
            gap: 6.0,
            button: PortalSmallButtonUi::default(),
            error_color: Color {
                r: 0.9,
                g: 0.3,
                b: 0.3,
                a: 1.0,
            },
            error_text_style: TextStyle::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum PortalNumberEditSubmit {
    NotHandled,
    Handled {
        normalized_text: Option<String>,
    },
    Error {
        message: Arc<str>,
    },
    Commit {
        tx: GraphTransaction,
        normalized_text: Option<String>,
    },
}

pub trait PortalNumberEditSpec: Clone + 'static {
    /// Returns the current numeric value for the node, or `None` if the node is not editable by this spec.
    fn initial_value(&self, graph: &Graph, node: NodeId) -> Option<f64>;

    fn format_value(&self, value: f64) -> String {
        format!("{value}")
    }

    fn parse_text(&self, text: &str) -> Result<f64, Arc<str>> {
        text.trim()
            .parse::<f64>()
            .map_err(|_| Arc::from("Invalid number"))
    }

    fn clamp_range(&self, _graph: &Graph, _node: NodeId) -> Option<(f64, f64)> {
        None
    }

    fn round_value(&self, _graph: &Graph, _node: NodeId, value: f64) -> f64 {
        value
    }

    fn submit_value(
        &self,
        graph: &Graph,
        node: NodeId,
        value: f64,
        text: &str,
    ) -> PortalNumberEditSubmit;

    fn supports_drag(&self, _graph: &Graph, _node: NodeId) -> bool {
        false
    }

    fn drag_threshold_px(&self, _graph: &Graph, _node: NodeId) -> f32 {
        1.0
    }

    fn drag_sensitivity_per_px(
        &self,
        _graph: &Graph,
        _node: NodeId,
        _mode: PortalTextStepMode,
    ) -> Option<f64> {
        None
    }

    fn drag_value_with_mode(
        &self,
        graph: &Graph,
        node: NodeId,
        start_value: f64,
        dx_px: f32,
        mode: PortalTextStepMode,
    ) -> Option<f64> {
        let sensitivity = self.drag_sensitivity_per_px(graph, node, mode)?;
        let next = start_value + dx_px as f64 * sensitivity;
        Some(self.normalize_value(graph, node, next))
    }

    fn step_size(&self, _graph: &Graph, _node: NodeId, _mode: PortalTextStepMode) -> Option<f64> {
        None
    }

    fn step_value_with_mode(
        &self,
        graph: &Graph,
        node: NodeId,
        value: f64,
        delta: i32,
        mode: PortalTextStepMode,
    ) -> Option<f64> {
        let step = self.step_size(graph, node, mode)?;
        Some(self.normalize_value(graph, node, value + step * delta as f64))
    }

    fn normalize_value(&self, graph: &Graph, node: NodeId, mut value: f64) -> f64 {
        if let Some((min, max)) = self.clamp_range(graph, node) {
            value = value.clamp(min.min(max), max.max(min));
        }
        self.round_value(graph, node, value)
    }
}

#[derive(Debug, Clone)]
pub struct PortalNumberEditor {
    root_name: Arc<str>,
}

impl PortalNumberEditor {
    pub fn new(root_name: impl Into<Arc<str>>) -> Self {
        Self {
            root_name: root_name.into(),
        }
    }

    pub fn render_number_input_for_node<H: UiHost, S: PortalNumberEditSpec>(
        &self,
        ecx: &mut ElementContext<'_, H>,
        graph_model: Model<Graph>,
        graph: &Graph,
        layout: NodeGraphPortalNodeLayout,
        style: &NodeGraphStyle,
        node: NodeId,
        spec: &S,
    ) -> Vec<fret_ui::element::AnyElement> {
        let Some(initial_value) = spec.initial_value(graph, node) else {
            return Vec::new();
        };

        self.sync_session_for_graph(ecx.app, ecx.window, graph);

        let ui = PortalNumberEditorUi::from_theme(ecx.theme().snapshot());
        let chrome = TextInputStyle::from_theme(ecx.theme().snapshot());

        let desired_text = spec.format_value(initial_value);
        let input_model = self.ensure_input_model(ecx.app, ecx.window, node, desired_text.clone());
        let error_model = self.ensure_error_model(ecx.app, ecx.window, node);
        let drag_model = self.ensure_drag_model(ecx.app, ecx.window);

        self.maybe_sync_from_graph(ecx.app, ecx.window, node, &input_model, desired_text);
        self.maybe_clear_error_on_input_change(ecx.app, &input_model, &error_model);

        let error_text = error_model
            .read_ref(ecx.app, |v| v.as_ref().map(|e| e.message.clone()))
            .ok()
            .flatten();

        let max_w = (layout.node_window.size.width.0 - 2.0 * style.node_padding)
            .max(80.0)
            .min(ui.max_width);

        let inset_left = layout.node_window.origin.x.0 + style.node_padding;
        let inset_top =
            layout.node_window.origin.y.0 + style.node_header_height + style.node_padding;

        let mut column = ColumnProps::default();
        column.gap = Px(ui.gap);
        column.layout = LayoutStyle {
            position: PositionStyle::Absolute,
            inset: InsetStyle {
                top: Some(Px(inset_top)),
                left: Some(Px(inset_left)),
                ..Default::default()
            },
            size: SizeStyle {
                width: Length::Px(Px(max_w)),
                height: Length::Auto,
                ..Default::default()
            },
            ..Default::default()
        };

        let submit = portal_submit_text_command(node);
        let cancel = portal_cancel_text_command(node);

        let current_text = input_model
            .read_ref(ecx.app, |v| v.clone())
            .ok()
            .unwrap_or_default();

        let parsed = spec.parse_text(&current_text).ok().unwrap_or(initial_value);
        let show_stepper = spec
            .step_value_with_mode(graph, node, parsed, 1, PortalTextStepMode::Normal)
            .is_some()
            || spec
                .step_value_with_mode(graph, node, parsed, -1, PortalTextStepMode::Normal)
                .is_some();

        let show_drag = spec.supports_drag(graph, node);

        vec![ecx.column(column, |cx| {
            let input_row = if show_stepper || show_drag {
                let mut row = RowProps::default();
                row.gap = Px(ui.gap);
                row.layout.size.width = Length::Fill;

                cx.row(row, |cx| {
                    let mut props = TextInputProps::new(input_model.clone());
                    props.chrome = chrome.clone();
                    props.submit_command = Some(submit.clone());
                    props.cancel_command = Some(cancel.clone());
                    props.layout.size.width = Length::Fill;

                    let mut btn_col = ColumnProps::default();
                    btn_col.gap = Px(2.0);
                    btn_col.padding = Edges::all(Px(0.0));
                    btn_col.layout.size.width = Length::Px(Px(ui.button.size));

                    let drag_button = if show_drag {
                        let cmd_node = node;
                        let drag_ui = ui.clone();
                        let drag_spec = spec.clone();
                        let drag_input = input_model.clone();
                        let drag_state = drag_model.clone();

                        let mut hover = HoverRegionProps::default();
                        hover.layout.size.width = Length::Px(Px(drag_ui.button.size));
                        hover.layout.size.height = Length::Px(Px(drag_ui.button.size));

                        Some(cx.hover_region(hover, move |cx, hovered| {
                            let pressed = is_dragging(cx.app, &drag_state, cmd_node);

                            let mut region = PointerRegionProps::default();
                            region.layout.size.width = Length::Fill;
                            region.layout.size.height = Length::Fill;

                            let mut semantics = SemanticsProps::default();
                            semantics.role = SemanticsRole::Button;
                            semantics.label = Some(Arc::from("Drag to adjust value"));
                            semantics.layout.size.width = Length::Fill;
                            semantics.layout.size.height = Length::Fill;

                            vec![cx.semantics(semantics, |cx| {
                                vec![cx.pointer_region(region, |cx| {
                                    cx.pointer_region_on_pointer_down(Arc::new({
                                        let graph_model = graph_model.clone();
                                        let down_spec = drag_spec.clone();
                                        let down_state = drag_state.clone();
                                        let down_input = drag_input.clone();
                                        move |host, cx, down| {
                                            handle_drag_pointer_down(
                                                host,
                                                cx.window,
                                                &graph_model,
                                                &down_spec,
                                                &down_state,
                                                &down_input,
                                                cmd_node,
                                                down,
                                            )
                                        }
                                    }));

                                    cx.pointer_region_on_pointer_move(Arc::new({
                                        let graph_model = graph_model.clone();
                                        let mv_spec = drag_spec.clone();
                                        let mv_state = drag_state.clone();
                                        let mv_input = drag_input.clone();
                                        move |host, _cx, mv| {
                                            handle_drag_pointer_move(
                                                host,
                                                &graph_model,
                                                &mv_spec,
                                                &mv_state,
                                                &mv_input,
                                                cmd_node,
                                                mv,
                                            )
                                        }
                                    }));

                                    cx.pointer_region_on_pointer_up(Arc::new({
                                        let up_state = drag_state.clone();
                                        move |host, cx, up| {
                                            handle_drag_pointer_up(
                                                host,
                                                cx.window,
                                                &up_state,
                                                cmd_node,
                                                up,
                                            )
                                        }
                                    }));

                                    vec![render_small_button(
                                        cx,
                                        &drag_ui.button,
                                        hovered,
                                        pressed,
                                        "<>",
                                    )]
                                })]
                            })]
                        }))
                    } else {
                        None
                    };

                    let mut minus = PressableProps::default();
                    minus.focusable = false;
                    minus.a11y.label = Some(Arc::from("Decrement"));
                    minus.layout.size.width = Length::Px(Px(ui.button.size));
                    minus.layout.size.height = Length::Px(Px(ui.button.size));

                    let mut plus = PressableProps::default();
                    plus.focusable = false;
                    plus.a11y.label = Some(Arc::from("Increment"));
                    plus.layout.size.width = Length::Px(Px(ui.button.size));
                    plus.layout.size.height = Length::Px(Px(ui.button.size));

                    vec![
                        cx.text_input(props),
                        cx.column(btn_col, |cx| {
                            let mut children = Vec::new();
                            if let Some(drag) = drag_button {
                                children.push(drag);
                            }

                            if show_stepper {
                                let stepper_ui = ui.clone();
                                let dec = cx.pressable(minus, |cx, state| {
                                    let cmd_node = node;
                                    cx.pressable_on_pointer_down(Arc::new(move |host, cx, down| {
                                        let mode = if down.modifiers.shift {
                                            PortalTextStepMode::Coarse
                                        } else if down.modifiers.ctrl || down.modifiers.meta {
                                            PortalTextStepMode::Fine
                                        } else {
                                            PortalTextStepMode::Normal
                                        };

                                     host.dispatch_command(
                                         Some(cx.window),
                                         portal_step_text_command_with_mode(cmd_node, -1, mode),
                                     );
                                     host.prevent_default(fret_runtime::DefaultAction::FocusOnPointerDown);
                                     fret_ui::action::PressablePointerDownResult::SkipDefaultAndStopPropagation
                                 }));
                                 vec![render_pressable_small_button(
                                     cx,
                                     &stepper_ui.button,
                                        state,
                                        "-",
                                    )]
                                });

                                let stepper_ui = ui.clone();
                                let inc = cx.pressable(plus, |cx, state| {
                                    let cmd_node = node;
                                    cx.pressable_on_pointer_down(Arc::new(move |host, cx, down| {
                                        let mode = if down.modifiers.shift {
                                            PortalTextStepMode::Coarse
                                        } else if down.modifiers.ctrl || down.modifiers.meta {
                                            PortalTextStepMode::Fine
                                        } else {
                                            PortalTextStepMode::Normal
                                        };

                                     host.dispatch_command(
                                         Some(cx.window),
                                         portal_step_text_command_with_mode(cmd_node, 1, mode),
                                     );
                                     host.prevent_default(fret_runtime::DefaultAction::FocusOnPointerDown);
                                     fret_ui::action::PressablePointerDownResult::SkipDefaultAndStopPropagation
                                 }));
                                 vec![render_pressable_small_button(
                                     cx,
                                     &stepper_ui.button,
                                        state,
                                        "+",
                                    )]
                                });

                                children.push(dec);
                                children.push(inc);
                            }

                            children
                        }),
                    ]
                })
            } else {
                let mut props = TextInputProps::new(input_model.clone());
                props.chrome = chrome.clone();
                props.submit_command = Some(submit);
                props.cancel_command = Some(cancel);
                props.layout.size.width = Length::Fill;
                cx.text_input(props)
            };

            let mut children = vec![input_row];

            if let Some(err) = error_text {
                let mut text = TextProps::new(err);
                text.color = Some(ui.error_color);
                text.style = Some(ui.error_text_style);
                text.layout.size.width = Length::Fill;
                children.push(cx.text_props(text));
            }

            children
        })]
    }

    fn session_key(&self, window: AppWindowId) -> PortalNumberEditorSessionKey {
        PortalNumberEditorSessionKey {
            window,
            root_name: self.root_name.clone(),
        }
    }

    fn with_session_mut<H: UiHost, R>(
        &self,
        app: &mut H,
        window: AppWindowId,
        f: impl FnOnce(&mut PortalNumberEditorSession, &mut H) -> R,
    ) -> R {
        let key = self.session_key(window);
        app.with_global_mut(PortalNumberEditorGlobalState::default, |global, app| {
            let session = global.sessions.entry(key).or_default();
            f(session, app)
        })
    }

    fn sync_session_for_graph<H: UiHost>(&self, app: &mut H, window: AppWindowId, graph: &Graph) {
        let live: HashSet<NodeId> = graph.nodes.keys().copied().collect();
        self.with_session_mut(app, window, |session, _app| {
            session.inputs.retain(|node, _| live.contains(node));
            session.errors.retain(|node, _| live.contains(node));
            session.last_synced.retain(|node, _| live.contains(node));
        });
    }

    fn ensure_input_model<H: UiHost>(
        &self,
        app: &mut H,
        window: AppWindowId,
        node: NodeId,
        initial_text: String,
    ) -> Model<String> {
        self.with_session_mut(app, window, |session, app| {
            session.inputs.entry(node).or_insert_with(|| {
                session.last_synced.insert(node, initial_text.clone());
                app.models_mut().insert(initial_text)
            });
            session.inputs.get(&node).expect("inserted").clone()
        })
    }

    fn ensure_error_model<H: UiHost>(
        &self,
        app: &mut H,
        window: AppWindowId,
        node: NodeId,
    ) -> Model<Option<PortalNumberEditorError>> {
        self.with_session_mut(app, window, |session, app| {
            session
                .errors
                .entry(node)
                .or_insert_with(|| app.models_mut().insert(None))
                .clone()
        })
    }

    fn ensure_drag_model<H: UiHost>(
        &self,
        app: &mut H,
        window: AppWindowId,
    ) -> Model<Option<PortalNumberDragSession>> {
        self.with_session_mut(app, window, |session, app| {
            session
                .drag
                .get_or_insert_with(|| app.models_mut().insert(None))
                .clone()
        })
    }

    fn maybe_sync_from_graph<H: UiHost>(
        &self,
        app: &mut H,
        window: AppWindowId,
        node: NodeId,
        input_model: &Model<String>,
        desired_text: String,
    ) {
        let should_sync = self.with_session_mut(app, window, |session, app| {
            let current = input_model
                .read_ref(app, |v| v.clone())
                .ok()
                .unwrap_or_default();
            let last = session.last_synced.get(&node).cloned().unwrap_or_default();
            if current != last {
                return false;
            }

            session.last_synced.insert(node, desired_text.clone());
            true
        });

        if should_sync {
            let _ = input_model.update(app, |v, _cx| {
                *v = desired_text;
            });
        }
    }

    fn maybe_clear_error_on_input_change<H: UiHost>(
        &self,
        app: &mut H,
        input_model: &Model<String>,
        error_model: &Model<Option<PortalNumberEditorError>>,
    ) {
        let current = input_model
            .read_ref(app, |v| v.clone())
            .ok()
            .unwrap_or_default();
        let clear = error_model
            .read_ref(app, |v| match v.as_ref() {
                Some(err) => err.last_input.as_str() != current.as_str(),
                None => false,
            })
            .ok()
            .unwrap_or(false);

        if clear {
            let _ = error_model.update(app, |v, _cx| *v = None);
        }
    }

    fn set_error<H: UiHost>(
        &self,
        app: &mut H,
        window: AppWindowId,
        node: NodeId,
        input_model: &Model<String>,
        message: Option<Arc<str>>,
    ) {
        let model = self.ensure_error_model(app, window, node);
        let last_input = input_model
            .read_ref(app, |v| v.clone())
            .ok()
            .unwrap_or_default();
        let _ = model.update(app, |v, _cx| {
            *v = message.map(|message| PortalNumberEditorError {
                last_input,
                message,
            });
        });
    }
}

#[derive(Debug, Clone)]
pub struct PortalNumberEditHandler<S: PortalNumberEditSpec> {
    editor: PortalNumberEditor,
    spec: S,
}

impl<S: PortalNumberEditSpec> PortalNumberEditHandler<S> {
    pub fn new(root_name: impl Into<Arc<str>>, spec: S) -> Self {
        Self {
            editor: PortalNumberEditor::new(root_name),
            spec,
        }
    }

    fn handle_submit<H: UiHost>(
        &mut self,
        cx: &mut fret_ui::retained_bridge::CommandCx<'_, H>,
        window: AppWindowId,
        graph: &Graph,
        node: NodeId,
        text: String,
    ) -> PortalCommandOutcome {
        let Some(initial_value) = self.spec.initial_value(graph, node) else {
            return PortalCommandOutcome::NotHandled;
        };

        let input_model = self.editor.ensure_input_model(
            cx.app,
            window,
            node,
            self.spec.format_value(initial_value),
        );
        let value = match self.spec.parse_text(&text) {
            Ok(v) => v,
            Err(message) => {
                self.editor
                    .set_error(cx.app, window, node, &input_model, Some(message));
                return PortalCommandOutcome::Handled;
            }
        };

        match self.spec.submit_value(graph, node, value, &text) {
            PortalNumberEditSubmit::NotHandled => PortalCommandOutcome::NotHandled,
            PortalNumberEditSubmit::Handled { normalized_text } => {
                self.editor
                    .set_error(cx.app, window, node, &input_model, None);
                if let Some(normalized) = normalized_text {
                    let _ = input_model.update(cx.app, |v, _cx| *v = normalized);
                }
                PortalCommandOutcome::Handled
            }
            PortalNumberEditSubmit::Error { message } => {
                self.editor
                    .set_error(cx.app, window, node, &input_model, Some(message));
                PortalCommandOutcome::Handled
            }
            PortalNumberEditSubmit::Commit {
                tx,
                normalized_text,
            } => {
                self.editor
                    .set_error(cx.app, window, node, &input_model, None);
                if let Some(normalized) = normalized_text {
                    let _ = input_model.update(cx.app, |v, _cx| *v = normalized);
                }
                PortalCommandOutcome::Commit(tx)
            }
        }
    }
}

impl<H: UiHost, S: PortalNumberEditSpec> NodeGraphPortalCommandHandler<H>
    for PortalNumberEditHandler<S>
{
    fn handle_portal_command(
        &mut self,
        cx: &mut fret_ui::retained_bridge::CommandCx<'_, H>,
        graph: &Graph,
        command: PortalTextCommand,
    ) -> PortalCommandOutcome {
        let Some(window) = cx.window else {
            return PortalCommandOutcome::NotHandled;
        };

        match command {
            PortalTextCommand::Cancel { node } => {
                let Some(initial) = self.spec.initial_value(graph, node) else {
                    return PortalCommandOutcome::NotHandled;
                };

                let reset = self.spec.format_value(initial);
                let input_model =
                    self.editor
                        .ensure_input_model(cx.app, window, node, reset.clone());
                let error_model = self.editor.ensure_error_model(cx.app, window, node);

                let _ = input_model.update(cx.app, |v, _cx| *v = reset);
                let _ = error_model.update(cx.app, |v, _cx| *v = None);

                PortalCommandOutcome::Handled
            }
            PortalTextCommand::Submit { node } => {
                let Some(initial_value) = self.spec.initial_value(graph, node) else {
                    return PortalCommandOutcome::NotHandled;
                };

                let input_model = self.editor.ensure_input_model(
                    cx.app,
                    window,
                    node,
                    self.spec.format_value(initial_value),
                );
                let text = input_model
                    .read_ref(cx.app, |v| v.clone())
                    .ok()
                    .unwrap_or_default();
                self.handle_submit(cx, window, graph, node, text)
            }
            PortalTextCommand::Step { node, delta, mode } => {
                let Some(initial_value) = self.spec.initial_value(graph, node) else {
                    return PortalCommandOutcome::NotHandled;
                };

                let input_model = self.editor.ensure_input_model(
                    cx.app,
                    window,
                    node,
                    self.spec.format_value(initial_value),
                );
                let current_text = input_model
                    .read_ref(cx.app, |v| v.clone())
                    .ok()
                    .unwrap_or_default();
                let base = self
                    .spec
                    .parse_text(&current_text)
                    .ok()
                    .unwrap_or(initial_value);

                let Some(next_value) = self
                    .spec
                    .step_value_with_mode(graph, node, base, delta, mode)
                else {
                    return PortalCommandOutcome::Handled;
                };

                let next_text = self.spec.format_value(next_value);
                let _ = input_model.update(cx.app, |v, _cx| *v = next_text.clone());
                self.handle_submit(cx, window, graph, node, next_text)
            }
        }
    }
}

#[derive(Debug, Clone)]
struct PortalNumberEditorError {
    last_input: String,
    message: Arc<str>,
}

#[derive(Debug, Clone, Copy)]
struct PortalNumberDragSession {
    node: NodeId,
    start_pos: fret_core::Point,
    start_value: f64,
    mode: PortalTextStepMode,
    threshold_px: f32,
    started: bool,
}

fn is_dragging<H: UiHost>(
    app: &mut H,
    drag: &Model<Option<PortalNumberDragSession>>,
    node: NodeId,
) -> bool {
    drag.read_ref(app, |v| v.as_ref().is_some_and(|s| s.node == node))
        .ok()
        .unwrap_or(false)
}

fn step_mode_for_down(down: PointerDownCx) -> PortalTextStepMode {
    if down.modifiers.shift {
        PortalTextStepMode::Coarse
    } else if down.modifiers.ctrl || down.modifiers.meta {
        PortalTextStepMode::Fine
    } else {
        PortalTextStepMode::Normal
    }
}

fn handle_drag_pointer_down<S: PortalNumberEditSpec>(
    host: &mut dyn fret_ui::action::UiPointerActionHost,
    window: AppWindowId,
    graph_model: &Model<Graph>,
    spec: &S,
    drag_state: &Model<Option<PortalNumberDragSession>>,
    input_model: &Model<String>,
    node: NodeId,
    down: PointerDownCx,
) -> bool {
    if down.button != MouseButton::Left {
        return false;
    }

    let mode = step_mode_for_down(down);
    let current_text = host
        .models_mut()
        .read(input_model, |v| v.clone())
        .ok()
        .unwrap_or_default();

    let (start_value, threshold_px) = host
        .models_mut()
        .read(graph_model, |graph| {
            let initial = spec.initial_value(graph, node)?;
            let parsed = spec.parse_text(&current_text).ok().unwrap_or(initial);
            Some((parsed, spec.drag_threshold_px(graph, node)))
        })
        .ok()
        .flatten()
        .unwrap_or((0.0, 1.0));

    let _ = host.models_mut().update(drag_state, |v| {
        *v = Some(PortalNumberDragSession {
            node,
            start_pos: down.position,
            start_value,
            mode,
            threshold_px: threshold_px.max(0.0),
            started: false,
        });
    });

    host.capture_pointer();
    host.set_cursor_icon(fret_core::CursorIcon::ColResize);
    host.request_redraw(window);
    true
}

fn handle_drag_pointer_move<S: PortalNumberEditSpec>(
    host: &mut dyn fret_ui::action::UiPointerActionHost,
    graph_model: &Model<Graph>,
    spec: &S,
    drag_state: &Model<Option<PortalNumberDragSession>>,
    input_model: &Model<String>,
    node: NodeId,
    mv: PointerMoveCx,
) -> bool {
    let active = host.models_mut().read(drag_state, |v| *v).ok().flatten();

    let Some(active) = active else {
        return false;
    };
    if active.node != node {
        return false;
    }

    let threshold = active.threshold_px.max(0.0);
    let dx0 = mv.position.x.0 - active.start_pos.x.0;
    let dy0 = mv.position.y.0 - active.start_pos.y.0;

    let mut active = active;
    if !active.started {
        let dist2 = dx0 * dx0 + dy0 * dy0;
        if dist2 <= threshold * threshold {
            return true;
        }

        let sign = if dx0 >= 0.0 { 1.0 } else { -1.0 };
        let adjusted = Px(active.start_pos.x.0 + sign * threshold);
        active.started = true;
        active.start_pos.x = adjusted;

        let _ = host.models_mut().update(drag_state, |v| {
            if let Some(s) = v.as_mut() {
                if s.node == node {
                    s.started = true;
                    s.start_pos.x = adjusted;
                }
            }
        });
    }

    let dx = mv.position.x.0 - active.start_pos.x.0;
    let next = host
        .models_mut()
        .read(graph_model, |graph| {
            spec.drag_value_with_mode(graph, node, active.start_value, dx, active.mode)
        })
        .ok()
        .flatten();

    let Some(next_value) = next else {
        return true;
    };

    let next_text = spec.format_value(next_value);
    let _ = host.models_mut().update(input_model, |v| *v = next_text);
    true
}

fn handle_drag_pointer_up(
    host: &mut dyn fret_ui::action::UiPointerActionHost,
    window: AppWindowId,
    drag_state: &Model<Option<PortalNumberDragSession>>,
    node: NodeId,
    up: PointerUpCx,
) -> bool {
    if up.button != MouseButton::Left {
        return false;
    }

    let ended = host
        .models_mut()
        .update(drag_state, |v| {
            let ended = v.take();
            ended
        })
        .ok()
        .flatten();

    let Some(ended) = ended else {
        return false;
    };
    if ended.node != node {
        return false;
    }

    host.release_pointer_capture();
    host.set_cursor_icon(fret_core::CursorIcon::Default);
    if ended.started {
        host.dispatch_command(Some(window), portal_submit_text_command(node));
    }
    host.request_redraw(window);
    true
}

#[derive(Debug, Default)]
struct PortalNumberEditorGlobalState {
    sessions: HashMap<PortalNumberEditorSessionKey, PortalNumberEditorSession>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct PortalNumberEditorSessionKey {
    window: AppWindowId,
    root_name: Arc<str>,
}

#[derive(Debug, Default)]
struct PortalNumberEditorSession {
    inputs: HashMap<NodeId, Model<String>>,
    errors: HashMap<NodeId, Model<Option<PortalNumberEditorError>>>,
    last_synced: HashMap<NodeId, String>,
    drag: Option<Model<Option<PortalNumberDragSession>>>,
}
