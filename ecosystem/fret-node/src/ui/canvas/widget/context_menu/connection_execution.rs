use crate::ui::canvas::widget::*;

#[derive(Debug)]
pub(super) enum ConnectionInsertMenuPlan {
    Apply(workflow::WireDropInsertPlan),
    Reject(DiagnosticSeverity, Arc<str>),
    Ignore,
}

#[derive(Debug)]
pub(super) enum ConnectionConversionMenuPlan {
    Apply(Vec<GraphOp>),
    Reject(DiagnosticSeverity, Arc<str>),
    Ignore,
}
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
