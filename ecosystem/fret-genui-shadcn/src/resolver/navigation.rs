use std::sync::Arc;

use fret_genui_core::spec::ElementKey;
use fret_ui::action::OnActivate;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};
use serde_json::{Map, Value, json};

use super::ShadcnResolver;

fn parse_u32(v: Option<&Value>) -> Option<u32> {
    v.and_then(|v| v.as_u64())
        .and_then(|n| u32::try_from(n).ok())
}

fn clamp_page(page: u32, total: u32) -> u32 {
    if total == 0 {
        return 1;
    }
    page.clamp(1, total)
}

impl ShadcnResolver {
    pub(super) fn render_pagination<H: UiHost>(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        key: &ElementKey,
        resolved_props: &Map<String, Value>,
    ) -> AnyElement {
        let current = parse_u32(resolved_props.get("currentPage")).unwrap_or(1);
        let total = parse_u32(resolved_props.get("totalPages"))
            .unwrap_or(1)
            .max(1);
        let current = clamp_page(current, total);

        let action = resolved_props
            .get("onPageChange")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(Arc::<str>::from);

        let scope = Self::genui_scope(cx);
        let element_key: Arc<str> = Arc::from(key.0.as_str());

        let mk_activate = |page: u32| -> Option<OnActivate> {
            let Some(scope) = scope.clone() else {
                return None;
            };
            let Some(action) = action.clone() else {
                return None;
            };
            let element_key = element_key.clone();
            Some(Arc::new(move |host, acx, _reason| {
                let params = json!({ "page": page });
                ShadcnResolver::emit_action_invocation_action(
                    host,
                    acx,
                    scope.action_queue.as_ref(),
                    scope.state.as_ref(),
                    scope.auto_apply_standard_actions,
                    &element_key,
                    "pageChange",
                    &action,
                    params,
                );
            }))
        };

        let mut buttons: Vec<AnyElement> = Vec::new();

        // Previous
        {
            let target = current.saturating_sub(1).max(1);
            let disabled = current == 1 || action.is_none();
            let mut b = fret_ui_shadcn::Button::new(Arc::<str>::from("Prev"))
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .size(fret_ui_shadcn::ButtonSize::Sm)
                .disabled(disabled);
            if let Some(on_activate) = mk_activate(target) {
                b = b.on_activate(on_activate);
            }
            buttons.push(b.into_element(cx));
        }

        // Page links (simple window around current)
        let window: u32 = 2;
        let start = current.saturating_sub(window).max(1);
        let end = (current + window).min(total);
        for page in start..=end {
            let active = page == current;
            let disabled = action.is_none();
            let label = Arc::<str>::from(page.to_string());
            let variant = if active {
                fret_ui_shadcn::ButtonVariant::Secondary
            } else {
                fret_ui_shadcn::ButtonVariant::Ghost
            };
            let mut b = fret_ui_shadcn::Button::new(label)
                .variant(variant)
                .size(fret_ui_shadcn::ButtonSize::Sm)
                .disabled(disabled);
            if let Some(on_activate) = mk_activate(page) {
                b = b.on_activate(on_activate);
            }
            buttons.push(b.into_element(cx));
        }

        // Next
        {
            let target = (current + 1).min(total);
            let disabled = current >= total || action.is_none();
            let mut b = fret_ui_shadcn::Button::new(Arc::<str>::from("Next"))
                .variant(fret_ui_shadcn::ButtonVariant::Outline)
                .size(fret_ui_shadcn::ButtonSize::Sm)
                .disabled(disabled);
            if let Some(on_activate) = mk_activate(target) {
                b = b.on_activate(on_activate);
            }
            buttons.push(b.into_element(cx));
        }

        fret_ui_kit::ui::h_flex(cx, move |_cx| buttons)
            .gap(fret_ui_kit::Space::N1)
            .items_center()
            .justify_center()
            .into_element(cx)
    }
}
