#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiCommandGatingTraceEntryV1 {
    pub command: String,
    pub enabled: bool,
    pub reason: String,
    #[serde(default)]
    pub scope: String,
    #[serde(default)]
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub menu_path: Option<String>,
    /// Structured explanation of why the command is disabled (multiple blockers may apply).
    #[serde(default)]
    pub blocked_by: Vec<String>,
    /// Best-effort detail fields to make debugging inconsistent gating easier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_available: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_when: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub menu_when: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled_override: Option<bool>,
    #[serde(default)]
    pub command_registered: bool,
}

#[derive(Debug, Clone)]
struct UiCommandGatingTraceCandidate {
    command: fret_runtime::CommandId,
    source: &'static str,
    menu_path: Option<String>,
    menu_when: Option<fret_runtime::WhenExpr>,
}

fn command_gating_trace_for_window(
    app: &App,
    window: AppWindowId,
    max_entries: usize,
) -> Vec<UiCommandGatingTraceEntryV1> {
    let gating = fret_runtime::best_effort_snapshot_for_window(app, window);

    let mut candidates: Vec<UiCommandGatingTraceCandidate> = Vec::new();

    // 1) Explicit gating inputs (useful for verifying that snapshots are being published).
    for (cmd, _) in gating.enabled_overrides() {
        candidates.push(UiCommandGatingTraceCandidate {
            command: cmd.clone(),
            source: "enabled_overrides",
            menu_path: None,
            menu_when: None,
        });
    }
    if let Some(map) = gating.action_availability() {
        for (cmd, _) in map {
            candidates.push(UiCommandGatingTraceCandidate {
                command: cmd.clone(),
                source: "action_availability",
                menu_path: None,
                menu_when: None,
            });
        }
    }

    // 2) Effective OS menubar model (data-only). This is the closest source of truth for
    // "visible menu commands" from the app's perspective.
    if let Some(menu_bar) = fret_app::effective_menu_bar(app) {
        collect_menu_bar_commands(&menu_bar, &mut candidates);
    }

    // 3) Command palette catalog (best-effort). This approximates the set of entries derived from
    // host commands; the actual palette filters further by query/group options.
    for (id, meta) in app.commands().iter() {
        if meta.hidden {
            continue;
        }
        candidates.push(UiCommandGatingTraceCandidate {
            command: id.clone(),
            source: "command_palette_catalog",
            menu_path: None,
            menu_when: None,
        });
    }

    // Always include a core, cross-surface set even if the host didn't publish any snapshot yet.
    for &cmd in &[
        "edit.undo",
        "edit.redo",
        "edit.copy",
        "edit.cut",
        "edit.paste",
        "edit.select_all",
        "focus.menu_bar",
    ] {
        candidates.push(UiCommandGatingTraceCandidate {
            command: fret_runtime::CommandId::from(cmd),
            source: "core",
            menu_path: None,
            menu_when: None,
        });
    }

    // Deduplicate by (command, source, menu_path) so repeated insertions don't explode snapshots.
    let mut seen: HashSet<(String, &'static str, Option<String>)> = HashSet::new();
    candidates.retain(|c| {
        let key = (
            c.command.as_str().to_string(),
            c.source,
            c.menu_path.clone(),
        );
        if seen.contains(&key) {
            return false;
        }
        seen.insert(key);
        true
    });

    candidates.sort_by(|a, b| {
        a.source
            .cmp(b.source)
            .then_with(|| a.menu_path.cmp(&b.menu_path))
            .then_with(|| a.command.as_str().cmp(b.command.as_str()))
    });

    let max_entries = max_entries.min(2000);
    candidates
        .into_iter()
        .take(max_entries)
        .map(|c| {
            let decision =
                command_gating_decision_trace(app, &gating, &c.command, c.menu_when.as_ref());

            UiCommandGatingTraceEntryV1 {
                command: c.command.as_str().to_string(),
                enabled: decision.enabled,
                reason: decision.reason,
                scope: decision.scope,
                source: c.source.to_string(),
                menu_path: c.menu_path,
                blocked_by: decision.blocked_by,
                action_available: decision.action_available,
                command_when: decision.command_when,
                menu_when: decision.menu_when,
                enabled_override: decision.enabled_override,
                command_registered: decision.command_registered,
            }
        })
        .collect()
}

#[derive(Debug, Clone)]
struct UiCommandGatingDecisionTrace {
    enabled: bool,
    reason: String,
    scope: String,
    blocked_by: Vec<String>,
    action_available: Option<bool>,
    command_when: Option<bool>,
    menu_when: Option<bool>,
    enabled_override: Option<bool>,
    command_registered: bool,
}

fn command_gating_decision_trace(
    app: &App,
    gating: &fret_runtime::WindowCommandGatingSnapshot,
    command: &fret_runtime::CommandId,
    menu_when: Option<&fret_runtime::WhenExpr>,
) -> UiCommandGatingDecisionTrace {
    let meta = app.commands().get(command.clone());
    let scope = meta
        .map(|m| format!("{:?}", m.scope))
        .unwrap_or_else(|| "Unknown".to_string());

    let mut blocked_by: Vec<String> = Vec::new();

    let action_available = if let Some(meta) = meta
        && meta.scope == fret_runtime::CommandScope::Widget
        && let Some(map) = gating.action_availability()
        && let Some(is_available) = map.get(command).copied()
    {
        Some(is_available)
    } else {
        None
    };
    if action_available == Some(false) {
        blocked_by.push("action_availability".to_string());
    }

    let command_when = meta.and_then(|m| {
        m.when
            .as_ref()
            .map(|w| w.eval_with_key_contexts(gating.input_ctx(), gating.key_contexts()))
    });
    if command_when == Some(false) {
        blocked_by.push("when".to_string());
    }

    let enabled_override = gating.enabled_overrides().get(command).copied();
    if enabled_override == Some(false) {
        blocked_by.push("enabled_override".to_string());
    }

    let menu_when =
        menu_when.map(|w| w.eval_with_key_contexts(gating.input_ctx(), gating.key_contexts()));
    if menu_when == Some(false) {
        blocked_by.push("menu_when".to_string());
    }

    let command_registered = meta.is_some();
    let enabled = blocked_by.is_empty();

    // Keep a stable "primary reason" string for backwards compatibility / easy grepping.
    let reason = if blocked_by.iter().any(|b| b == "action_availability") {
        "action_unavailable"
    } else if blocked_by.iter().any(|b| b == "when") {
        "when_false"
    } else if blocked_by.iter().any(|b| b == "enabled_override") {
        "disabled_override"
    } else if blocked_by.iter().any(|b| b == "menu_when") {
        "menu_when_false"
    } else if !command_registered {
        "unknown_command"
    } else {
        "enabled"
    }
    .to_string();

    UiCommandGatingDecisionTrace {
        enabled,
        reason,
        scope,
        blocked_by,
        action_available,
        command_when,
        menu_when,
        enabled_override,
        command_registered,
    }
}

fn collect_menu_bar_commands(
    menu_bar: &fret_runtime::MenuBar,
    out: &mut Vec<UiCommandGatingTraceCandidate>,
) {
    for menu in &menu_bar.menus {
        let menu_title = menu.title.as_ref().to_string();
        collect_menu_items(&menu_title, &menu.items, out);
    }
}

fn collect_menu_items(
    prefix: &str,
    items: &[fret_runtime::MenuItem],
    out: &mut Vec<UiCommandGatingTraceCandidate>,
) {
    for item in items {
        match item {
            fret_runtime::MenuItem::Command { command, when, .. } => {
                out.push(UiCommandGatingTraceCandidate {
                    command: command.clone(),
                    source: "menu_bar",
                    menu_path: Some(prefix.to_string()),
                    menu_when: when.clone(),
                });
            }
            fret_runtime::MenuItem::Label { .. } => {}
            fret_runtime::MenuItem::Separator | fret_runtime::MenuItem::SystemMenu { .. } => {}
            fret_runtime::MenuItem::Submenu {
                title,
                when: _,
                items,
            } => {
                let next = format!("{prefix} > {}", title.as_ref());
                collect_menu_items(&next, items, out);
            }
        }
    }
}
