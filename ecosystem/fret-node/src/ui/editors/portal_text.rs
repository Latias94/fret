use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use fret_core::{AppWindowId, Color, Edges, Px, TextStyle};
use fret_runtime::Model;
use fret_ui::action::PressablePointerDownResult;
use fret_ui::element::{
    ColumnProps, InsetStyle, LayoutStyle, Length, PositionStyle, PressableProps, RowProps,
    SizeStyle, TextInputProps, TextProps,
};
use fret_ui::elements::ElementContext;
use fret_ui::{TextInputStyle, ThemeSnapshot, UiHost};

use crate::core::{Graph, NodeId};
use crate::ops::GraphTransaction;
use crate::ui::editors::chrome::{PortalSmallButtonUi, render_pressable_small_button};
use crate::ui::portal::{
    NodeGraphPortalCommandHandler, NodeGraphPortalNodeLayout, PortalCommandOutcome,
    PortalTextCommand, PortalTextStepMode, portal_cancel_text_command,
    portal_step_text_command_with_mode, portal_submit_text_command,
};
use crate::ui::style::NodeGraphStyle;

#[derive(Debug, Clone)]
pub struct PortalTextEditorUi {
    pub max_width: f32,
    pub gap: f32,
    pub stepper_button: PortalSmallButtonUi,

    pub error_color: Color,
    pub error_text_style: TextStyle,
}

impl PortalTextEditorUi {
    pub fn from_theme(theme: ThemeSnapshot) -> Self {
        let font_size = theme.metric_required("metric.font.size").0;

        Self {
            max_width: 180.0,
            gap: 6.0,
            stepper_button: PortalSmallButtonUi::from_theme(theme),

            error_color: theme.color_required("destructive"),
            error_text_style: TextStyle {
                size: Px((font_size - 1.0).max(10.0)),
                ..TextStyle::default()
            },
        }
    }
}

impl Default for PortalTextEditorUi {
    fn default() -> Self {
        Self {
            max_width: 180.0,
            gap: 6.0,
            stepper_button: PortalSmallButtonUi::default(),

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
pub struct PortalTextEditor {
    root_name: Arc<str>,
}

impl PortalTextEditor {
    pub fn new(root_name: impl Into<Arc<str>>) -> Self {
        Self {
            root_name: root_name.into(),
        }
    }

    pub fn render_text_input_for_node<H: UiHost, S: PortalTextEditSpec>(
        &self,
        ecx: &mut ElementContext<'_, H>,
        graph: &Graph,
        layout: NodeGraphPortalNodeLayout,
        style: &NodeGraphStyle,
        node: NodeId,
        spec: &S,
    ) -> Vec<fret_ui::element::AnyElement> {
        self.sync_session_for_graph(ecx.app, ecx.window, graph);

        let ui = PortalTextEditorUi::from_theme(ecx.theme().snapshot());
        let chrome = TextInputStyle::from_theme(ecx.theme().snapshot());

        let desired_text = spec.initial_text(graph, node);
        let input_model = self.ensure_input_model(ecx.app, ecx.window, graph, node, spec);
        let error_model = self.ensure_error_model(ecx.app, ecx.window, node);

        self.maybe_sync_from_graph(ecx.app, ecx.window, node, &input_model, desired_text);
        self.maybe_clear_error_on_input_change(
            ecx.app,
            ecx.window,
            node,
            &input_model,
            &error_model,
        );

        let error_text = error_model.read_ref(ecx.app, |v| v.clone()).ok().flatten();

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

        vec![ecx.column(column, |cx| {
            let current_text = input_model
                .read_ref(cx.app, |v| v.clone())
                .ok()
                .unwrap_or_default();
            let show_stepper = spec.step_text(graph, node, &current_text, 1).is_some()
                || spec.step_text(graph, node, &current_text, -1).is_some();

            let input_row = if show_stepper {
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
                    btn_col.layout.size.width = Length::Px(Px(ui.stepper_button.size));

                    let mut minus = PressableProps::default();
                    minus.focusable = false;
                    minus.a11y.label = Some(Arc::from("Decrement"));
                    minus.layout.size.width = Length::Px(Px(ui.stepper_button.size));
                    minus.layout.size.height = Length::Px(Px(ui.stepper_button.size));

                    let mut plus = PressableProps::default();
                    plus.focusable = false;
                    plus.a11y.label = Some(Arc::from("Increment"));
                    plus.layout.size.width = Length::Px(Px(ui.stepper_button.size));
                    plus.layout.size.height = Length::Px(Px(ui.stepper_button.size));

                    vec![
                        cx.text_input(props),
                        cx.column(btn_col, |cx| {
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
                                    host.prevent_default(
                                        fret_runtime::DefaultAction::FocusOnPointerDown,
                                    );
                                    PressablePointerDownResult::SkipDefaultAndStopPropagation
                                }));
                                vec![render_pressable_small_button(
                                    cx,
                                    &stepper_ui.stepper_button,
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
                                    host.prevent_default(
                                        fret_runtime::DefaultAction::FocusOnPointerDown,
                                    );
                                    PressablePointerDownResult::SkipDefaultAndStopPropagation
                                }));
                                vec![render_pressable_small_button(
                                    cx,
                                    &stepper_ui.stepper_button,
                                    state,
                                    "+",
                                )]
                            });

                            vec![dec, inc]
                        }),
                    ]
                })
            } else {
                let mut props = TextInputProps::new(input_model);
                props.chrome = chrome;
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

    fn session_key(&self, window: AppWindowId) -> PortalTextEditorSessionKey {
        PortalTextEditorSessionKey {
            window,
            root_name: self.root_name.clone(),
        }
    }

    fn with_session_mut<H: UiHost, R>(
        &self,
        app: &mut H,
        window: AppWindowId,
        f: impl FnOnce(&mut PortalTextEditorSession, &mut H) -> R,
    ) -> R {
        let key = self.session_key(window);
        app.with_global_mut(PortalTextEditorGlobalState::default, |global, app| {
            let session = global.sessions.entry(key).or_default();
            f(session, app)
        })
    }

    fn sync_session_for_graph<H: UiHost>(&self, app: &mut H, window: AppWindowId, graph: &Graph) {
        let live: HashSet<NodeId> = graph.nodes.keys().copied().collect();
        self.with_session_mut(app, window, |session, _app| {
            session.inputs.retain(|k, _| live.contains(k));
            session.errors.retain(|k, _| live.contains(k));
            session.last_synced.retain(|k, _| live.contains(k));
            session.last_seen.retain(|k, _| live.contains(k));
        });
    }

    fn ensure_input_model<H: UiHost, S: PortalTextEditSpec>(
        &self,
        app: &mut H,
        window: AppWindowId,
        graph: &Graph,
        node: NodeId,
        spec: &S,
    ) -> Model<String> {
        self.with_session_mut(app, window, |session, app| {
            session.inputs.entry(node).or_insert_with(|| {
                let text = spec.initial_text(graph, node);
                session.last_synced.insert(node, text.clone());
                session.last_seen.insert(node, text.clone());
                app.models_mut().insert(text)
            });
            session.inputs.get(&node).expect("model exists").clone()
        })
    }

    fn ensure_error_model<H: UiHost>(
        &self,
        app: &mut H,
        window: AppWindowId,
        node: NodeId,
    ) -> Model<Option<Arc<str>>> {
        self.with_session_mut(app, window, |session, app| {
            session
                .errors
                .entry(node)
                .or_insert_with(|| app.models_mut().insert(None))
                .clone()
        })
    }

    fn set_error<H: UiHost>(
        &self,
        app: &mut H,
        window: AppWindowId,
        node: NodeId,
        error: Option<Arc<str>>,
    ) {
        let model = self.ensure_error_model(app, window, node);
        let _ = model.update(app, |v, _cx| {
            *v = error;
        });
    }

    fn reset_input<H: UiHost, S: PortalTextEditSpec>(
        &self,
        app: &mut H,
        window: AppWindowId,
        graph: &Graph,
        node: NodeId,
        spec: &S,
    ) {
        let model = self.ensure_input_model(app, window, graph, node, spec);
        let text = spec.initial_text(graph, node);
        let synced = text.clone();
        let _ = model.update(app, |v, _cx| {
            *v = text;
        });
        self.with_session_mut(app, window, |session, _app| {
            session.last_synced.insert(node, synced.clone());
            session.last_seen.insert(node, synced);
        });
    }

    fn read_input<H: UiHost, S: PortalTextEditSpec>(
        &self,
        app: &mut H,
        window: AppWindowId,
        graph: &Graph,
        node: NodeId,
        spec: &S,
    ) -> String {
        let model = self.ensure_input_model(app, window, graph, node, spec);
        model.read_ref(app, |v| v.clone()).ok().unwrap_or_default()
    }

    fn write_normalized_input<H: UiHost, S: PortalTextEditSpec>(
        &self,
        app: &mut H,
        window: AppWindowId,
        graph: &Graph,
        node: NodeId,
        spec: &S,
        normalized: Option<String>,
    ) {
        let Some(normalized) = normalized else {
            return;
        };
        let synced = normalized.clone();
        let model = self.ensure_input_model(app, window, graph, node, spec);
        let _ = model.update(app, |v, _cx| {
            *v = normalized;
        });
        self.with_session_mut(app, window, |session, _app| {
            session.last_synced.insert(node, synced.clone());
            session.last_seen.insert(node, synced);
        });
    }

    fn maybe_sync_from_graph<H: UiHost>(
        &self,
        app: &mut H,
        window: AppWindowId,
        node: NodeId,
        input_model: &Model<String>,
        desired_text: String,
    ) {
        let current = input_model
            .read_ref(app, |v| v.clone())
            .ok()
            .unwrap_or_default();

        let should_update = self.with_session_mut(app, window, |session, _app| {
            let Some(last) = session.last_synced.get(&node).cloned() else {
                session.last_synced.insert(node, current.clone());
                return false;
            };

            // If the user edited the input (current != last_synced), do not clobber their work.
            if current != last {
                return false;
            }

            // If the graph-derived value hasn't changed, do nothing.
            if desired_text == last {
                return false;
            }

            session.last_synced.insert(node, desired_text.clone());
            true
        });

        if should_update {
            let next = desired_text.clone();
            let _ = input_model.update(app, |v, _cx| {
                *v = desired_text;
            });
            self.with_session_mut(app, window, |session, _app| {
                session.last_seen.insert(node, next);
            });
        }
    }

    fn maybe_clear_error_on_input_change<H: UiHost>(
        &self,
        app: &mut H,
        window: AppWindowId,
        node: NodeId,
        input_model: &Model<String>,
        error_model: &Model<Option<Arc<str>>>,
    ) {
        let current = input_model
            .read_ref(app, |v| v.clone())
            .ok()
            .unwrap_or_default();

        let changed = self.with_session_mut(app, window, |session, _app| {
            if session.last_seen.get(&node).is_some_and(|v| v == &current) {
                return false;
            }
            session.last_seen.insert(node, current);
            true
        });

        if changed {
            let _ = error_model.update(app, |err, _cx| {
                *err = None;
            });
        }
    }
}

#[derive(Debug, Clone)]
pub enum PortalTextEditSubmit {
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

pub trait PortalTextEditSpec {
    fn initial_text(&self, graph: &Graph, node: NodeId) -> String;
    fn submit(&self, graph: &Graph, node: NodeId, text: &str) -> PortalTextEditSubmit;

    fn step_text(&self, _graph: &Graph, _node: NodeId, _text: &str, _delta: i32) -> Option<String> {
        None
    }

    fn step_text_with_mode(
        &self,
        graph: &Graph,
        node: NodeId,
        text: &str,
        delta: i32,
        _mode: PortalTextStepMode,
    ) -> Option<String> {
        self.step_text(graph, node, text, delta)
    }
}

#[derive(Debug, Clone)]
pub struct PortalTextEditHandler<S> {
    editor: PortalTextEditor,
    spec: S,
}

impl<S> PortalTextEditHandler<S> {
    pub fn new(root_name: impl Into<Arc<str>>, spec: S) -> Self {
        Self {
            editor: PortalTextEditor::new(root_name),
            spec,
        }
    }
}

impl<H: UiHost, S: PortalTextEditSpec> NodeGraphPortalCommandHandler<H>
    for PortalTextEditHandler<S>
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
                self.editor
                    .reset_input(cx.app, window, graph, node, &self.spec);
                self.editor.set_error(cx.app, window, node, None);
                PortalCommandOutcome::Handled
            }
            PortalTextCommand::Submit { node } => {
                let text = self
                    .editor
                    .read_input(cx.app, window, graph, node, &self.spec);
                self.handle_submit(cx, window, graph, node, text)
            }
            PortalTextCommand::Step { node, delta, mode } => {
                let text = self
                    .editor
                    .read_input(cx.app, window, graph, node, &self.spec);
                let Some(next_text) = self
                    .spec
                    .step_text_with_mode(graph, node, &text, delta, mode)
                else {
                    return PortalCommandOutcome::NotHandled;
                };

                let model = self
                    .editor
                    .ensure_input_model(cx.app, window, graph, node, &self.spec);
                let submit_text = next_text.clone();
                let _ = model.update(cx.app, |v, _cx| {
                    *v = next_text;
                });
                self.editor
                    .with_session_mut(cx.app, window, |session, _app| {
                        session.last_synced.insert(node, submit_text.clone());
                        session.last_seen.insert(node, submit_text.clone());
                    });

                self.editor.set_error(cx.app, window, node, None);
                self.handle_submit(cx, window, graph, node, submit_text)
            }
        }
    }
}

impl<S: PortalTextEditSpec> PortalTextEditHandler<S> {
    fn handle_submit<H: UiHost>(
        &mut self,
        cx: &mut fret_ui::retained_bridge::CommandCx<'_, H>,
        window: AppWindowId,
        graph: &Graph,
        node: NodeId,
        text: String,
    ) -> PortalCommandOutcome {
        match self.spec.submit(graph, node, &text) {
            PortalTextEditSubmit::NotHandled => PortalCommandOutcome::NotHandled,
            PortalTextEditSubmit::Handled { normalized_text } => {
                self.editor.set_error(cx.app, window, node, None);
                self.editor.write_normalized_input(
                    cx.app,
                    window,
                    graph,
                    node,
                    &self.spec,
                    normalized_text,
                );
                PortalCommandOutcome::Handled
            }
            PortalTextEditSubmit::Error { message } => {
                self.editor.set_error(cx.app, window, node, Some(message));
                PortalCommandOutcome::Handled
            }
            PortalTextEditSubmit::Commit {
                tx,
                normalized_text,
            } => {
                self.editor.set_error(cx.app, window, node, None);
                self.editor.write_normalized_input(
                    cx.app,
                    window,
                    graph,
                    node,
                    &self.spec,
                    normalized_text,
                );
                PortalCommandOutcome::Commit(tx)
            }
        }
    }
}

#[derive(Debug, Default)]
struct PortalTextEditorGlobalState {
    sessions: HashMap<PortalTextEditorSessionKey, PortalTextEditorSession>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct PortalTextEditorSessionKey {
    window: AppWindowId,
    root_name: Arc<str>,
}

#[derive(Debug, Default)]
struct PortalTextEditorSession {
    inputs: HashMap<NodeId, Model<String>>,
    errors: HashMap<NodeId, Model<Option<Arc<str>>>>,
    last_synced: HashMap<NodeId, String>,
    last_seen: HashMap<NodeId, String>,
}
