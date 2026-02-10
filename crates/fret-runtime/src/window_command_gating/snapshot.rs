/// Window-scoped snapshot that aggregates data-only command gating inputs.
use std::collections::HashMap;
use std::sync::Arc;

use crate::{CommandId, CommandMeta, CommandScope, InputContext, WhenExpr};

///
/// This is a consumption-focused seam intended for runner/platform and UI-kit layers:
/// menus, command palette, shortcut help, etc.
#[derive(Debug, Default, Clone)]
pub struct WindowCommandGatingSnapshot {
    input_ctx: InputContext,
    enabled_overrides: HashMap<CommandId, bool>,
    action_availability: Option<Arc<HashMap<CommandId, bool>>>,
}

impl WindowCommandGatingSnapshot {
    pub fn new(input_ctx: InputContext, enabled_overrides: HashMap<CommandId, bool>) -> Self {
        Self {
            input_ctx,
            enabled_overrides,
            action_availability: None,
        }
    }

    pub fn input_ctx(&self) -> &InputContext {
        &self.input_ctx
    }

    pub fn with_input_ctx(mut self, input_ctx: InputContext) -> Self {
        self.input_ctx = input_ctx;
        self
    }

    pub fn enabled_overrides(&self) -> &HashMap<CommandId, bool> {
        &self.enabled_overrides
    }

    pub fn action_availability(&self) -> Option<&HashMap<CommandId, bool>> {
        self.action_availability.as_deref()
    }

    /// GPUI naming parity: query the latest published dispatch-path availability, if present.
    ///
    /// This is only meaningful for `CommandScope::Widget` commands; other scopes are not modeled
    /// as dispatch-path availability entries today.
    pub fn is_action_available(&self, command: &CommandId) -> Option<bool> {
        self.action_availability
            .as_ref()
            .and_then(|map| map.get(command).copied())
    }

    pub fn with_action_availability(
        mut self,
        action_availability: Option<Arc<HashMap<CommandId, bool>>>,
    ) -> Self {
        self.action_availability = action_availability;
        self
    }

    pub fn is_enabled_for_meta(
        &self,
        command: &CommandId,
        scope: CommandScope,
        when: Option<&WhenExpr>,
    ) -> bool {
        if scope == CommandScope::Widget
            && let Some(map) = self.action_availability.as_ref()
            && let Some(is_available) = map.get(command).copied()
            && !is_available
        {
            return false;
        }
        if when.is_some_and(|w| !w.eval(&self.input_ctx)) {
            return false;
        }
        self.enabled_overrides.get(command).copied().unwrap_or(true)
    }

    pub fn is_enabled_for_command(&self, command: &CommandId, meta: &CommandMeta) -> bool {
        self.is_enabled_for_meta(command, meta.scope, meta.when.as_ref())
    }
}
