use std::sync::Arc;

use super::super::hover::ResponseExt;

/// Aggregated response surface for helper-owned tab bars.
#[derive(Debug, Clone)]
pub struct TabBarResponse {
    pub(crate) selected: Option<Arc<str>>,
    pub(crate) selected_changed: bool,
    pub(crate) triggers: Vec<TabTriggerResponse>,
}

/// Outward trigger response for a single helper-owned tab item.
#[derive(Debug, Clone)]
pub struct TabTriggerResponse {
    pub(crate) id: Arc<str>,
    pub(crate) selected: bool,
    pub(crate) trigger: ResponseExt,
}

impl TabBarResponse {
    pub fn selected_id(&self) -> Option<&str> {
        self.selected.as_deref()
    }

    pub fn selected_changed(&self) -> bool {
        self.selected_changed
    }

    pub fn triggers(&self) -> &[TabTriggerResponse] {
        &self.triggers
    }

    pub fn trigger(&self, id: &str) -> Option<&TabTriggerResponse> {
        self.triggers
            .iter()
            .find(|trigger| trigger.id.as_ref() == id)
    }
}

impl TabTriggerResponse {
    pub fn id(&self) -> &str {
        self.id.as_ref()
    }

    pub fn selected(&self) -> bool {
        self.selected
    }

    pub fn response(&self) -> ResponseExt {
        self.trigger
    }

    pub fn clicked(&self) -> bool {
        self.trigger.clicked()
    }

    pub fn activated(&self) -> bool {
        self.trigger.activated()
    }

    pub fn deactivated(&self) -> bool {
        self.trigger.deactivated()
    }
}
