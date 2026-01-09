use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use fret_core::{AppWindowId, Color, Px, TextStyle};
use fret_runtime::Model;
use fret_ui::element::{
    ColumnProps, InsetStyle, LayoutStyle, Length, PositionStyle, SizeStyle, TextInputProps,
    TextProps,
};
use fret_ui::elements::ElementContext;
use fret_ui::{TextInputStyle, ThemeSnapshot, UiHost};

use crate::core::{Graph, NodeId};
use crate::ops::GraphTransaction;
use crate::ui::portal::{
    NodeGraphPortalCommandHandler, NodeGraphPortalNodeLayout, PortalCommandOutcome,
    PortalTextCommand, portal_cancel_text_command, portal_submit_text_command,
};
use crate::ui::style::NodeGraphStyle;

#[derive(Debug, Clone)]
pub struct PortalTextEditorUi {
    pub max_width: f32,
    pub gap: f32,
    pub error_color: Color,
    pub error_text_style: TextStyle,
}

impl PortalTextEditorUi {
    pub fn from_theme(theme: ThemeSnapshot) -> Self {
        let font_size = theme.metric_required("metric.font.size").0;

        Self {
            max_width: 180.0,
            gap: 6.0,
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

        let input_model = self.ensure_input_model(ecx.app, ecx.window, graph, node, spec);
        let error_model = self.ensure_error_model(ecx.app, ecx.window, node);
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
            let mut props = TextInputProps::new(input_model);
            props.chrome = chrome;
            props.submit_command = Some(submit);
            props.cancel_command = Some(cancel);
            props.layout.size.width = Length::Fill;

            let mut children = vec![cx.text_input(props)];

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
            session
                .inputs
                .entry(node)
                .or_insert_with(|| app.models_mut().insert(spec.initial_text(graph, node)))
                .clone()
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
        let _ = model.update(app, |v, _cx| {
            *v = text;
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
        let model = self.ensure_input_model(app, window, graph, node, spec);
        let _ = model.update(app, |v, _cx| {
            *v = normalized;
        });
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

        let (node, is_submit) = match command {
            PortalTextCommand::Submit { node } => (node, true),
            PortalTextCommand::Cancel { node } => (node, false),
        };

        if !is_submit {
            self.editor
                .reset_input(cx.app, window, graph, node, &self.spec);
            self.editor.set_error(cx.app, window, node, None);
            return PortalCommandOutcome::Handled;
        }

        let text = self
            .editor
            .read_input(cx.app, window, graph, node, &self.spec);
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
}
