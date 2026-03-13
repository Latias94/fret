use super::*;
use crate::rules::{DiagnosticSeverity, DiagnosticTarget};

#[test]
fn split_edge_reroute_rejection_toast_uses_first_diagnostic_message() {
    let toast =
        toast::split_edge_reroute_rejection_toast::<NoopNodeGraphCanvasMiddleware>(&[Diagnostic {
            key: "reroute_rejected".into(),
            severity: DiagnosticSeverity::Warning,
            target: DiagnosticTarget::Graph,
            message: "reroute was rejected".into(),
            fixes: Vec::new(),
        }]);

    assert_eq!(toast.0, DiagnosticSeverity::Warning);
    assert_eq!(&*toast.1, "reroute was rejected");
}

#[test]
fn split_edge_reroute_rejection_toast_falls_back_when_message_missing() {
    let toast =
        toast::split_edge_reroute_rejection_toast::<NoopNodeGraphCanvasMiddleware>(&[Diagnostic {
            key: "reroute_rejected".into(),
            severity: DiagnosticSeverity::Info,
            target: DiagnosticTarget::Graph,
            message: String::new(),
            fixes: Vec::new(),
        }]);

    assert_eq!(toast.0, DiagnosticSeverity::Error);
    assert_eq!(&*toast.1, "failed to insert reroute");
}
