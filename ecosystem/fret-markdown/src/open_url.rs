use std::sync::Arc;

use fret_runtime::Effect;
use fret_ui::action::{ActionCx, ActivateReason, UiActionHost};

use crate::LinkInfo;

pub type OnLinkActivate =
    Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, ActivateReason, LinkInfo) + 'static>;

/// A conservative allowlist for `Effect::OpenUrl` to avoid surprising/suspicious schemes in UI.
///
/// This is intentionally strict:
/// - allow: `http://`, `https://`, `mailto:`
/// - deny: `javascript:`, `data:`, `file:`, empty, whitespace-only
pub fn is_safe_open_url(url: &str) -> bool {
    let url = url.trim();
    if url.is_empty() {
        return false;
    }

    let lower = url.to_ascii_lowercase();
    if lower.starts_with("javascript:")
        || lower.starts_with("data:")
        || lower.starts_with("file:")
        || lower.starts_with("vbscript:")
    {
        return false;
    }

    lower.starts_with("http://") || lower.starts_with("https://") || lower.starts_with("mailto:")
}

/// Convenience: open links via the runner's `Effect::OpenUrl` plumbing (desktop/web).
///
/// Usage:
/// - `components.on_link_activate = Some(fret_markdown::on_link_activate_open_url());`
pub fn on_link_activate_open_url() -> OnLinkActivate {
    Arc::new(|host, _cx, _reason, link| {
        if !is_safe_open_url(&link.href) {
            return;
        }
        host.push_effect(Effect::OpenUrl {
            url: link.href.to_string(),
            target: None,
            rel: None,
        });
    })
}
