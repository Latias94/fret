use std::sync::Arc;

use crate::core::{Graph, NodeId};
use crate::ui::portal_commands::{PortalCommandOutcome, PortalTextCommand};

use super::portal_command_policy::{
    PortalNumberCommandPlan, PortalNumberCommandSubmit, PortalNumberEditSpec,
    PortalNumberEditSubmit, PortalTextCommandPlan, PortalTextEditSpec, PortalTextEditSubmit,
    plan_portal_number_command, plan_portal_text_command,
};

pub(super) trait PortalTextCommandSession {
    fn current_text(&mut self, node: NodeId, initial_text: String) -> String;
    fn write_text(&mut self, node: NodeId, text: String);
    fn set_error(&mut self, node: NodeId, message: Option<Arc<str>>);
}

pub(super) fn handle_portal_text_command_with_session<S, T>(
    graph: &Graph,
    spec: &S,
    session: &mut T,
    command: PortalTextCommand,
) -> PortalCommandOutcome
where
    S: PortalTextEditSpec,
    T: PortalTextCommandSession,
{
    let current_text = match command {
        PortalTextCommand::Submit { node } | PortalTextCommand::Step { node, .. } => {
            Some(session.current_text(node, spec.initial_text(graph, node)))
        }
        PortalTextCommand::Cancel { .. } => None,
    };

    match plan_portal_text_command(graph, spec, command, current_text.as_deref()) {
        PortalTextCommandPlan::NotHandled => PortalCommandOutcome::NotHandled,
        PortalTextCommandPlan::Cancel { node, reset_text } => {
            session.write_text(node, reset_text);
            session.set_error(node, None);
            PortalCommandOutcome::Handled
        }
        PortalTextCommandPlan::Submit { node, submit, .. } => {
            apply_portal_text_submit(session, node, submit)
        }
        PortalTextCommandPlan::StepSubmit { node, text, submit } => {
            session.write_text(node, text);
            session.set_error(node, None);
            apply_portal_text_submit(session, node, submit)
        }
    }
}

fn apply_portal_text_submit<T>(
    session: &mut T,
    node: NodeId,
    submit: PortalTextEditSubmit,
) -> PortalCommandOutcome
where
    T: PortalTextCommandSession,
{
    match submit {
        PortalTextEditSubmit::NotHandled => PortalCommandOutcome::NotHandled,
        PortalTextEditSubmit::Handled { normalized_text } => {
            session.set_error(node, None);
            if let Some(normalized) = normalized_text {
                session.write_text(node, normalized);
            }
            PortalCommandOutcome::Handled
        }
        PortalTextEditSubmit::Error { message } => {
            session.set_error(node, Some(message));
            PortalCommandOutcome::Handled
        }
        PortalTextEditSubmit::Commit {
            tx,
            normalized_text,
        } => {
            session.set_error(node, None);
            if let Some(normalized) = normalized_text {
                session.write_text(node, normalized);
            }
            PortalCommandOutcome::Commit(tx)
        }
    }
}

pub(super) trait PortalNumberCommandSession {
    fn current_text(&mut self, node: NodeId, initial_text: String) -> String;
    fn write_text(&mut self, node: NodeId, text: String);
    fn set_error(&mut self, node: NodeId, message: Option<Arc<str>>);
}

pub(super) fn handle_portal_number_command_with_session<S, T>(
    graph: &Graph,
    spec: &S,
    session: &mut T,
    command: PortalTextCommand,
) -> PortalCommandOutcome
where
    S: PortalNumberEditSpec,
    T: PortalNumberCommandSession,
{
    let current_text = match command {
        PortalTextCommand::Submit { node } | PortalTextCommand::Step { node, .. } => spec
            .initial_value(graph, node)
            .map(|value| session.current_text(node, spec.format_value(value))),
        PortalTextCommand::Cancel { .. } => None,
    };

    match plan_portal_number_command(graph, spec, command, current_text.as_deref()) {
        PortalNumberCommandPlan::NotHandled => PortalCommandOutcome::NotHandled,
        PortalNumberCommandPlan::Handled => PortalCommandOutcome::Handled,
        PortalNumberCommandPlan::Cancel { node, reset_text } => {
            session.write_text(node, reset_text);
            session.set_error(node, None);
            PortalCommandOutcome::Handled
        }
        PortalNumberCommandPlan::Submit { node, submit, .. } => {
            apply_portal_number_submit(session, node, submit)
        }
        PortalNumberCommandPlan::StepSubmit { node, text, submit } => {
            session.write_text(node, text);
            apply_portal_number_submit(session, node, submit)
        }
    }
}

fn apply_portal_number_submit<T>(
    session: &mut T,
    node: NodeId,
    submit: PortalNumberCommandSubmit,
) -> PortalCommandOutcome
where
    T: PortalNumberCommandSession,
{
    match submit {
        PortalNumberCommandSubmit::ParseError { message } => {
            session.set_error(node, Some(message));
            PortalCommandOutcome::Handled
        }
        PortalNumberCommandSubmit::Submit(submit) => match submit {
            PortalNumberEditSubmit::NotHandled => PortalCommandOutcome::NotHandled,
            PortalNumberEditSubmit::Handled { normalized_text } => {
                session.set_error(node, None);
                if let Some(normalized) = normalized_text {
                    session.write_text(node, normalized);
                }
                PortalCommandOutcome::Handled
            }
            PortalNumberEditSubmit::Error { message } => {
                session.set_error(node, Some(message));
                PortalCommandOutcome::Handled
            }
            PortalNumberEditSubmit::Commit {
                tx,
                normalized_text,
            } => {
                session.set_error(node, None);
                if let Some(normalized) = normalized_text {
                    session.write_text(node, normalized);
                }
                PortalCommandOutcome::Commit(tx)
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::core::{Graph, NodeId};
    use crate::ops::GraphTransaction;
    use crate::ui::editors::portal_command_policy::{
        PortalNumberEditSpec, PortalNumberEditSubmit, PortalTextEditSpec, PortalTextEditSubmit,
    };
    use crate::ui::portal_commands::{PortalCommandOutcome, PortalTextCommand, PortalTextStepMode};

    use std::collections::HashMap;

    use super::{
        PortalNumberCommandSession, PortalTextCommandSession,
        handle_portal_number_command_with_session, handle_portal_text_command_with_session,
    };

    fn test_node() -> NodeId {
        NodeId::from_u128(0x12345678123456781234567812345678)
    }

    #[derive(Debug, Default)]
    struct MemoryTextSession {
        inputs: HashMap<NodeId, String>,
        errors: HashMap<NodeId, Option<Arc<str>>>,
    }

    impl PortalTextCommandSession for MemoryTextSession {
        fn current_text(&mut self, node: NodeId, initial_text: String) -> String {
            self.inputs.entry(node).or_insert(initial_text).clone()
        }

        fn write_text(&mut self, node: NodeId, text: String) {
            self.inputs.insert(node, text);
        }

        fn set_error(&mut self, node: NodeId, message: Option<Arc<str>>) {
            self.errors.insert(node, message);
        }
    }

    #[derive(Debug)]
    struct TextSpec;

    impl PortalTextEditSpec for TextSpec {
        fn initial_text(&self, _graph: &Graph, _node: NodeId) -> String {
            "initial".to_string()
        }

        fn submit(&self, _graph: &Graph, _node: NodeId, text: &str) -> PortalTextEditSubmit {
            match text {
                "error" => PortalTextEditSubmit::Error {
                    message: Arc::from("text error"),
                },
                "commit" => PortalTextEditSubmit::Commit {
                    tx: GraphTransaction::new().with_label("text commit"),
                    normalized_text: Some("committed".to_string()),
                },
                other => PortalTextEditSubmit::Handled {
                    normalized_text: Some(other.to_ascii_uppercase()),
                },
            }
        }
    }

    #[test]
    fn portal_text_session_applies_commands_without_retained_command_cx() {
        let graph = Graph::default();
        let node = test_node();
        let mut session = MemoryTextSession::default();

        let handled = handle_portal_text_command_with_session(
            &graph,
            &TextSpec,
            &mut session,
            PortalTextCommand::Submit { node },
        );
        assert!(matches!(handled, PortalCommandOutcome::Handled));
        assert_eq!(
            session.inputs.get(&node).map(String::as_str),
            Some("INITIAL")
        );
        assert_eq!(session.errors.get(&node), Some(&None));

        session.inputs.insert(node, "error".to_string());
        let error = handle_portal_text_command_with_session(
            &graph,
            &TextSpec,
            &mut session,
            PortalTextCommand::Submit { node },
        );
        assert!(matches!(error, PortalCommandOutcome::Handled));
        assert_eq!(
            session.errors.get(&node).and_then(|err| err.as_deref()),
            Some("text error")
        );

        let cancel = handle_portal_text_command_with_session(
            &graph,
            &TextSpec,
            &mut session,
            PortalTextCommand::Cancel { node },
        );
        assert!(matches!(cancel, PortalCommandOutcome::Handled));
        assert_eq!(
            session.inputs.get(&node).map(String::as_str),
            Some("initial")
        );
        assert_eq!(session.errors.get(&node), Some(&None));

        session.inputs.insert(node, "commit".to_string());
        let commit = handle_portal_text_command_with_session(
            &graph,
            &TextSpec,
            &mut session,
            PortalTextCommand::Submit { node },
        );
        let PortalCommandOutcome::Commit(tx) = commit else {
            panic!("commit submit should return a transaction outcome");
        };
        assert_eq!(tx.label.as_deref(), Some("text commit"));
        assert_eq!(
            session.inputs.get(&node).map(String::as_str),
            Some("committed")
        );
        assert_eq!(session.errors.get(&node), Some(&None));
    }

    #[derive(Debug, Default)]
    struct MemoryNumberSession {
        inputs: HashMap<NodeId, String>,
        errors: HashMap<NodeId, Option<Arc<str>>>,
    }

    impl PortalNumberCommandSession for MemoryNumberSession {
        fn current_text(&mut self, node: NodeId, initial_text: String) -> String {
            self.inputs.entry(node).or_insert(initial_text).clone()
        }

        fn write_text(&mut self, node: NodeId, text: String) {
            self.inputs.insert(node, text);
        }

        fn set_error(&mut self, node: NodeId, message: Option<Arc<str>>) {
            self.errors.insert(node, message);
        }
    }

    #[derive(Debug, Clone)]
    struct NumberSpec;

    impl PortalNumberEditSpec for NumberSpec {
        fn initial_value(&self, _graph: &Graph, _node: NodeId) -> Option<f64> {
            Some(10.0)
        }

        fn format_value(&self, value: f64) -> String {
            format!("{value:.1}")
        }

        fn parse_text(&self, text: &str) -> Result<f64, Arc<str>> {
            text.trim()
                .parse::<f64>()
                .map_err(|_| Arc::from("number error"))
        }

        fn submit_value(
            &self,
            _graph: &Graph,
            _node: NodeId,
            value: f64,
            _text: &str,
        ) -> PortalNumberEditSubmit {
            if value >= 20.0 {
                PortalNumberEditSubmit::Commit {
                    tx: GraphTransaction::new().with_label("number commit"),
                    normalized_text: Some(format!("{value:.1}")),
                }
            } else {
                PortalNumberEditSubmit::Handled {
                    normalized_text: Some(format!("{value:.1}")),
                }
            }
        }

        fn step_size(
            &self,
            _graph: &Graph,
            _node: NodeId,
            mode: PortalTextStepMode,
        ) -> Option<f64> {
            Some(match mode {
                PortalTextStepMode::Fine => 0.5,
                PortalTextStepMode::Normal => 1.0,
                PortalTextStepMode::Coarse => 10.0,
            })
        }
    }

    #[test]
    fn portal_number_session_applies_commands_without_retained_command_cx() {
        let graph = Graph::default();
        let node = test_node();
        let mut session = MemoryNumberSession::default();

        session.inputs.insert(node, "bad".to_string());
        let parse_error = handle_portal_number_command_with_session(
            &graph,
            &NumberSpec,
            &mut session,
            PortalTextCommand::Submit { node },
        );
        assert!(matches!(parse_error, PortalCommandOutcome::Handled));
        assert_eq!(
            session.errors.get(&node).and_then(|err| err.as_deref()),
            Some("number error")
        );

        let cancel = handle_portal_number_command_with_session(
            &graph,
            &NumberSpec,
            &mut session,
            PortalTextCommand::Cancel { node },
        );
        assert!(matches!(cancel, PortalCommandOutcome::Handled));
        assert_eq!(session.inputs.get(&node).map(String::as_str), Some("10.0"));
        assert_eq!(session.errors.get(&node), Some(&None));

        let step = handle_portal_number_command_with_session(
            &graph,
            &NumberSpec,
            &mut session,
            PortalTextCommand::Step {
                node,
                delta: 2,
                mode: PortalTextStepMode::Coarse,
            },
        );
        let PortalCommandOutcome::Commit(tx) = step else {
            panic!("coarse step should commit the normalized number");
        };
        assert_eq!(tx.label.as_deref(), Some("number commit"));
        assert_eq!(session.inputs.get(&node).map(String::as_str), Some("30.0"));
        assert_eq!(session.errors.get(&node), Some(&None));
    }
}
