use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn show_toast<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        severity: DiagnosticSeverity,
        message: impl Into<Arc<str>>,
    ) {
        if let Some(prev) = self.interaction.toast.take() {
            host.push_effect(Effect::CancelTimer { token: prev.timer });
        }

        let timer = host.next_timer_token();
        host.push_effect(Effect::SetTimer {
            window,
            token: timer,
            after: Duration::from_millis(2400),
            repeat: None,
        });

        self.interaction.toast = Some(ToastState {
            timer,
            severity,
            message: message.into(),
        });
    }

    pub(super) fn toast_from_diagnostics(
        diags: &[Diagnostic],
    ) -> Option<(DiagnosticSeverity, Arc<str>)> {
        let first = diags.first()?;
        if first.message.is_empty() {
            return None;
        }
        Some((first.severity, Arc::<str>::from(first.message.clone())))
    }
}
