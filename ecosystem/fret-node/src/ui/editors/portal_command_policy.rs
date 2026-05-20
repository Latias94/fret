use std::sync::Arc;

use crate::core::{Graph, NodeId};
use crate::ops::GraphTransaction;
use crate::ui::portal_commands::{PortalTextCommand, PortalTextStepMode};

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
pub enum PortalTextCommandPlan {
    NotHandled,
    Cancel {
        node: NodeId,
        reset_text: String,
    },
    Submit {
        node: NodeId,
        text: String,
        submit: PortalTextEditSubmit,
    },
    StepSubmit {
        node: NodeId,
        text: String,
        submit: PortalTextEditSubmit,
    },
}

pub fn plan_portal_text_command<S: PortalTextEditSpec>(
    graph: &Graph,
    spec: &S,
    command: PortalTextCommand,
    current_text: Option<&str>,
) -> PortalTextCommandPlan {
    match command {
        PortalTextCommand::Cancel { node } => PortalTextCommandPlan::Cancel {
            node,
            reset_text: spec.initial_text(graph, node),
        },
        PortalTextCommand::Submit { node } => {
            let Some(text) = current_text else {
                return PortalTextCommandPlan::NotHandled;
            };
            let text = text.to_string();
            let submit = spec.submit(graph, node, &text);
            PortalTextCommandPlan::Submit { node, text, submit }
        }
        PortalTextCommand::Step { node, delta, mode } => {
            let Some(text) = current_text else {
                return PortalTextCommandPlan::NotHandled;
            };
            let Some(next_text) = spec.step_text_with_mode(graph, node, text, delta, mode) else {
                return PortalTextCommandPlan::NotHandled;
            };
            let submit = spec.submit(graph, node, &next_text);
            PortalTextCommandPlan::StepSubmit {
                node,
                text: next_text,
                submit,
            }
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
pub enum PortalNumberCommandSubmit {
    ParseError { message: Arc<str> },
    Submit(PortalNumberEditSubmit),
}

#[derive(Debug, Clone)]
pub enum PortalNumberCommandPlan {
    NotHandled,
    Handled,
    Cancel {
        node: NodeId,
        reset_text: String,
    },
    Submit {
        node: NodeId,
        text: String,
        submit: PortalNumberCommandSubmit,
    },
    StepSubmit {
        node: NodeId,
        text: String,
        submit: PortalNumberCommandSubmit,
    },
}

pub fn plan_portal_number_command<S: PortalNumberEditSpec>(
    graph: &Graph,
    spec: &S,
    command: PortalTextCommand,
    current_text: Option<&str>,
) -> PortalNumberCommandPlan {
    let Some(initial) = (match command {
        PortalTextCommand::Cancel { node }
        | PortalTextCommand::Submit { node }
        | PortalTextCommand::Step { node, .. } => spec.initial_value(graph, node),
    }) else {
        return PortalNumberCommandPlan::NotHandled;
    };

    match command {
        PortalTextCommand::Cancel { node } => PortalNumberCommandPlan::Cancel {
            node,
            reset_text: spec.format_value(initial),
        },
        PortalTextCommand::Submit { node } => {
            let fallback = spec.format_value(initial);
            let text = current_text.unwrap_or(&fallback).to_string();
            let submit = plan_number_submit(graph, spec, node, &text);
            PortalNumberCommandPlan::Submit { node, text, submit }
        }
        PortalTextCommand::Step { node, delta, mode } => {
            let fallback = spec.format_value(initial);
            let text = current_text.unwrap_or(&fallback);
            let base = spec.parse_text(text).ok().unwrap_or(initial);
            let Some(next_value) = spec.step_value_with_mode(graph, node, base, delta, mode) else {
                return PortalNumberCommandPlan::Handled;
            };
            let next_text = spec.format_value(next_value);
            let submit = plan_number_submit(graph, spec, node, &next_text);
            PortalNumberCommandPlan::StepSubmit {
                node,
                text: next_text,
                submit,
            }
        }
    }
}

fn plan_number_submit<S: PortalNumberEditSpec>(
    graph: &Graph,
    spec: &S,
    node: NodeId,
    text: &str,
) -> PortalNumberCommandSubmit {
    match spec.parse_text(text) {
        Ok(value) => PortalNumberCommandSubmit::Submit(spec.submit_value(graph, node, value, text)),
        Err(message) => PortalNumberCommandSubmit::ParseError { message },
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::core::{Graph, NodeId};
    use crate::ui::portal_commands::{PortalTextCommand, PortalTextStepMode};

    use super::{
        PortalNumberCommandPlan, PortalNumberCommandSubmit, PortalNumberEditSpec,
        PortalNumberEditSubmit, PortalTextCommandPlan, PortalTextEditSpec, PortalTextEditSubmit,
        plan_portal_number_command, plan_portal_text_command,
    };

    fn test_node() -> NodeId {
        NodeId::from_u128(0x12345678123456781234567812345678)
    }

    #[derive(Debug)]
    struct TextSpec;

    impl PortalTextEditSpec for TextSpec {
        fn initial_text(&self, _graph: &Graph, _node: NodeId) -> String {
            "initial".to_string()
        }

        fn submit(&self, _graph: &Graph, _node: NodeId, text: &str) -> PortalTextEditSubmit {
            PortalTextEditSubmit::Handled {
                normalized_text: Some(text.to_ascii_uppercase()),
            }
        }

        fn step_text_with_mode(
            &self,
            _graph: &Graph,
            _node: NodeId,
            text: &str,
            delta: i32,
            mode: PortalTextStepMode,
        ) -> Option<String> {
            Some(format!("{text}:{delta}:{}", mode.as_str()))
        }
    }

    #[test]
    fn portal_text_command_policy_plans_cancel_submit_and_step_without_retained_cx() {
        let graph = Graph::default();
        let node = test_node();

        let cancel =
            plan_portal_text_command(&graph, &TextSpec, PortalTextCommand::Cancel { node }, None);
        let PortalTextCommandPlan::Cancel {
            node: got,
            reset_text,
        } = cancel
        else {
            panic!("cancel should produce a reset plan");
        };
        assert_eq!(got, node);
        assert_eq!(reset_text, "initial");

        let submit = plan_portal_text_command(
            &graph,
            &TextSpec,
            PortalTextCommand::Submit { node },
            Some("value"),
        );
        let PortalTextCommandPlan::Submit {
            node: got,
            text,
            submit,
        } = submit
        else {
            panic!("submit should produce a submit plan");
        };
        assert_eq!(got, node);
        assert_eq!(text, "value");
        let PortalTextEditSubmit::Handled { normalized_text } = submit else {
            panic!("submit result should be handled");
        };
        assert_eq!(normalized_text.as_deref(), Some("VALUE"));

        let step = plan_portal_text_command(
            &graph,
            &TextSpec,
            PortalTextCommand::Step {
                node,
                delta: -2,
                mode: PortalTextStepMode::Fine,
            },
            Some("value"),
        );
        let PortalTextCommandPlan::StepSubmit {
            node: got,
            text,
            submit,
        } = step
        else {
            panic!("step should produce a submit plan for the stepped text");
        };
        assert_eq!(got, node);
        assert_eq!(text, "value:-2:fine");
        let PortalTextEditSubmit::Handled { normalized_text } = submit else {
            panic!("step submit result should be handled");
        };
        assert_eq!(normalized_text.as_deref(), Some("VALUE:-2:FINE"));
    }

    #[derive(Debug, Clone)]
    struct NumberSpec {
        step: Option<f64>,
    }

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
                .map_err(|_| Arc::from("not a number"))
        }

        fn submit_value(
            &self,
            _graph: &Graph,
            _node: NodeId,
            value: f64,
            _text: &str,
        ) -> PortalNumberEditSubmit {
            PortalNumberEditSubmit::Handled {
                normalized_text: Some(self.format_value(value)),
            }
        }

        fn step_size(
            &self,
            _graph: &Graph,
            _node: NodeId,
            mode: PortalTextStepMode,
        ) -> Option<f64> {
            let step = self.step?;
            Some(match mode {
                PortalTextStepMode::Fine => step / 10.0,
                PortalTextStepMode::Normal => step,
                PortalTextStepMode::Coarse => step * 10.0,
            })
        }
    }

    #[test]
    fn portal_number_command_policy_plans_cancel_submit_parse_error_and_step_without_retained_cx() {
        let graph = Graph::default();
        let node = test_node();
        let spec = NumberSpec { step: Some(2.0) };

        let cancel = plan_portal_number_command(
            &graph,
            &spec,
            PortalTextCommand::Cancel { node },
            Some("12.0"),
        );
        let PortalNumberCommandPlan::Cancel {
            node: got,
            reset_text,
        } = cancel
        else {
            panic!("cancel should produce a reset plan");
        };
        assert_eq!(got, node);
        assert_eq!(reset_text, "10.0");

        let submit = plan_portal_number_command(
            &graph,
            &spec,
            PortalTextCommand::Submit { node },
            Some("12"),
        );
        let PortalNumberCommandPlan::Submit {
            node: got,
            text,
            submit,
        } = submit
        else {
            panic!("submit should produce a submit plan");
        };
        assert_eq!(got, node);
        assert_eq!(text, "12");
        let PortalNumberCommandSubmit::Submit(PortalNumberEditSubmit::Handled { normalized_text }) =
            submit
        else {
            panic!("submit should parse and call the spec");
        };
        assert_eq!(normalized_text.as_deref(), Some("12.0"));

        let parse_error = plan_portal_number_command(
            &graph,
            &spec,
            PortalTextCommand::Submit { node },
            Some("nope"),
        );
        let PortalNumberCommandPlan::Submit { submit, .. } = parse_error else {
            panic!("invalid submit should still be handled by the policy");
        };
        let PortalNumberCommandSubmit::ParseError { message } = submit else {
            panic!("invalid submit should produce a parse-error plan");
        };
        assert_eq!(&*message, "not a number");

        let step = plan_portal_number_command(
            &graph,
            &spec,
            PortalTextCommand::Step {
                node,
                delta: 2,
                mode: PortalTextStepMode::Fine,
            },
            Some("10"),
        );
        let PortalNumberCommandPlan::StepSubmit {
            node: got,
            text,
            submit,
        } = step
        else {
            panic!("supported step should submit the stepped value");
        };
        assert_eq!(got, node);
        assert_eq!(text, "10.4");
        let PortalNumberCommandSubmit::Submit(PortalNumberEditSubmit::Handled { normalized_text }) =
            submit
        else {
            panic!("step should submit the stepped value");
        };
        assert_eq!(normalized_text.as_deref(), Some("10.4"));

        let unsupported_step = plan_portal_number_command(
            &graph,
            &NumberSpec { step: None },
            PortalTextCommand::Step {
                node,
                delta: 1,
                mode: PortalTextStepMode::Normal,
            },
            Some("10"),
        );
        assert!(matches!(unsupported_step, PortalNumberCommandPlan::Handled));
    }
}
