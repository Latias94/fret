use fret_runtime::PlatformCapabilities;
use fret_ui::{Theme, UiHost};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DegradationReason {
    UnsupportedCapability,
    BudgetExceeded,
    ReducedMotion,
    ReducedTransparency,
    InvalidInput,
}

#[derive(Debug, Clone, Copy)]
pub struct ResolveCtx<'a> {
    pub theme: &'a Theme,
    pub caps: &'a PlatformCapabilities,
    pub prefers_reduced_motion: Option<bool>,
    pub prefers_reduced_transparency: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecipeDegradedEvent {
    pub label: &'static str,
    pub reason: DegradationReason,
}

#[derive(Debug, Default, Clone)]
pub struct RecipeDiagnostics {
    pub degraded: Vec<RecipeDegradedEvent>,
}

pub fn report_recipe_degraded<H: UiHost>(app: &mut H, event: RecipeDegradedEvent) {
    app.with_global_mut_untracked(RecipeDiagnostics::default, |diag, _app| {
        diag.degraded.push(event);
    });
}

#[derive(Debug, Clone)]
pub struct ResolvedWithFallback<T> {
    pub value: T,
    pub degraded: bool,
    pub reason: Option<DegradationReason>,
    pub label: Option<&'static str>,
}

impl<T> ResolvedWithFallback<T> {
    pub fn ok(value: T) -> Self {
        Self {
            value,
            degraded: false,
            reason: None,
            label: None,
        }
    }

    pub fn degraded(value: T, label: &'static str, reason: DegradationReason) -> Self {
        Self {
            value,
            degraded: true,
            reason: Some(reason),
            label: Some(label),
        }
    }

    pub fn report_if_degraded<H: UiHost>(&self, app: &mut H) {
        if !self.degraded {
            return;
        }
        let (Some(label), Some(reason)) = (self.label, self.reason) else {
            return;
        };
        report_recipe_degraded(app, RecipeDegradedEvent { label, reason });
    }
}
