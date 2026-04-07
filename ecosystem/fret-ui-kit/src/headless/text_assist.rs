//! UI-kit glue for input-owned text-assist surfaces.
//!
//! This module preserves the existing headless controller/match API from
//! `fret_ui_headless::text_assist` while adding the small amount of UI glue that should live
//! above pure query/filter/navigation math:
//! - input-owned expanded/collapsed policy,
//! - active-descendant / controls-element semantics wiring,
//! - outer keydown arbitration for Arrow/Home/Page/Enter/Escape.

use std::sync::Arc;

use fret_core::{KeyCode, NodeId};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, OnKeyDown, UiFocusActionHost};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

use crate::declarative::active_descendant::{
    active_descendant_for_index, active_element_for_index,
};

pub use fret_ui_headless::text_assist::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct InputOwnedTextAssistSemantics {
    pub active_descendant: Option<NodeId>,
    pub active_descendant_element: Option<u64>,
    pub controls_element: Option<u64>,
    pub expanded: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputOwnedTextAssistKeyOptions {
    pub match_mode: TextAssistMatchMode,
    pub wrap_navigation: bool,
    pub page_step: usize,
}

impl Default for InputOwnedTextAssistKeyOptions {
    fn default() -> Self {
        Self {
            match_mode: TextAssistMatchMode::Prefix,
            wrap_navigation: false,
            page_step: 4,
        }
    }
}

pub type OnTextAssistAccept =
    Arc<dyn Fn(&mut dyn UiFocusActionHost, ActionCx, TextAssistMatch) + 'static>;

pub fn input_owned_text_assist_expanded(
    query: &str,
    dismissed_query: &str,
    visible_count: usize,
) -> bool {
    !query.trim().is_empty() && query != dismissed_query && visible_count > 0
}

pub fn controller_with_active_item_id(
    items: &[TextAssistItem],
    query: &str,
    active_item_id: Option<&Arc<str>>,
    mode: TextAssistMatchMode,
    wrap_navigation: bool,
) -> TextAssistController {
    let mut controller = TextAssistController::new(mode).with_wrap_navigation(wrap_navigation);
    controller.rebuild(items, query);
    if let Some(active_item_id) = active_item_id {
        controller.set_active_item_id(Some(active_item_id.clone()));
    }
    controller
}

pub fn active_match_index(controller: &TextAssistController) -> Option<usize> {
    let active = controller.active_item_id()?;
    controller
        .visible()
        .iter()
        .position(|entry| &entry.item_id == active)
}

pub fn input_owned_text_assist_semantics<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    option_elements: &[GlobalElementId],
    active_index: Option<usize>,
    controls_element: Option<GlobalElementId>,
    expanded: bool,
) -> InputOwnedTextAssistSemantics {
    InputOwnedTextAssistSemantics {
        active_descendant: expanded
            .then(|| active_descendant_for_index(cx, option_elements, active_index))
            .flatten(),
        active_descendant_element: expanded
            .then(|| active_element_for_index(option_elements, active_index))
            .flatten()
            .map(|element| element.0),
        controls_element: controls_element.map(|element| element.0),
        expanded,
    }
}

pub fn input_owned_text_assist_key_handler(
    items: Arc<[TextAssistItem]>,
    query_model: Model<String>,
    dismissed_query_model: Model<String>,
    active_item_id_model: Model<Option<Arc<str>>>,
    options: InputOwnedTextAssistKeyOptions,
    on_accept: OnTextAssistAccept,
) -> OnKeyDown {
    Arc::new(move |host, action_cx, down| {
        if down.repeat || down.ime_composing {
            return false;
        }

        let query = host
            .models_mut()
            .read(&query_model, Clone::clone)
            .ok()
            .unwrap_or_default();
        let dismissed_query = host
            .models_mut()
            .read(&dismissed_query_model, Clone::clone)
            .ok()
            .unwrap_or_default();
        let active_item_id = host
            .models_mut()
            .read(&active_item_id_model, Clone::clone)
            .ok()
            .unwrap_or(None);

        let mut controller = controller_with_active_item_id(
            items.as_ref(),
            &query,
            active_item_id.as_ref(),
            options.match_mode,
            options.wrap_navigation,
        );
        let visible_count = if query.trim().is_empty() {
            0
        } else {
            controller.visible().len()
        };
        let expanded = input_owned_text_assist_expanded(&query, &dismissed_query, visible_count);

        let movement = match down.key {
            KeyCode::ArrowDown => Some(TextAssistMove::Next),
            KeyCode::ArrowUp => Some(TextAssistMove::Previous),
            KeyCode::PageDown => Some(TextAssistMove::PageDown {
                amount: options.page_step.max(1),
            }),
            KeyCode::PageUp => Some(TextAssistMove::PageUp {
                amount: options.page_step.max(1),
            }),
            KeyCode::Home => Some(TextAssistMove::First),
            KeyCode::End => Some(TextAssistMove::Last),
            _ => None,
        };

        if let Some(movement) = movement {
            if query.trim().is_empty() || visible_count == 0 {
                return false;
            }

            if query == dismissed_query {
                let _ = host.models_mut().update(&dismissed_query_model, |value| {
                    value.clear();
                });
            }

            controller.move_active(movement);
            let next_active = controller.active_item_id().cloned();
            let _ = host
                .models_mut()
                .update(&active_item_id_model, |value| *value = next_active);
            host.request_redraw(action_cx.window);
            return true;
        }

        match down.key {
            KeyCode::Enter | KeyCode::NumpadEnter => {
                if !expanded {
                    return false;
                }
                let Some(active) = controller.active_match().cloned() else {
                    return false;
                };
                on_accept(host, action_cx, active);
                true
            }
            KeyCode::Escape => {
                if !expanded {
                    return false;
                }
                let _ = host.models_mut().update(&dismissed_query_model, |value| {
                    value.clear();
                    value.push_str(&query);
                });
                host.request_redraw(action_cx.window);
                true
            }
            _ => false,
        }
    })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::{
        TextAssistItem, TextAssistMatchMode, active_match_index, controller_with_active_item_id,
        input_owned_text_assist_expanded,
    };

    fn sample_items() -> Vec<TextAssistItem> {
        vec![
            TextAssistItem::new("cube", "Cube"),
            TextAssistItem::new("cylinder", "Cylinder"),
            TextAssistItem::new("capsule", "Capsule"),
        ]
    }

    #[test]
    fn expanded_requires_non_empty_visible_and_not_dismissed_query() {
        assert!(!input_owned_text_assist_expanded("", "", 3));
        assert!(!input_owned_text_assist_expanded("c", "c", 3));
        assert!(!input_owned_text_assist_expanded("c", "", 0));
        assert!(input_owned_text_assist_expanded("c", "", 3));
    }

    #[test]
    fn controller_helper_restores_requested_active_item() {
        let items = sample_items();
        let active = Arc::<str>::from("capsule");
        let controller = controller_with_active_item_id(
            &items,
            "c",
            Some(&active),
            TextAssistMatchMode::Prefix,
            false,
        );

        assert_eq!(
            controller.active_item_id().map(|id| id.as_ref()),
            Some("capsule")
        );
    }

    #[test]
    fn active_match_index_tracks_the_resolved_active_row() {
        let items = sample_items();
        let active = Arc::<str>::from("cylinder");
        let controller = controller_with_active_item_id(
            &items,
            "c",
            Some(&active),
            TextAssistMatchMode::Prefix,
            false,
        );

        assert_eq!(active_match_index(&controller), Some(1));
    }
}
