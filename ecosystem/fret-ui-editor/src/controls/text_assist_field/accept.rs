//! Text-assist match acceptance owner.

use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::action::{ActionCx, UiActionHost};
use fret_ui_kit::headless::text_assist::TextAssistMatch;

use super::OnTextAssistFieldAccept;

pub(super) fn accept_text_assist_match(
    host: &mut dyn UiActionHost,
    action_cx: ActionCx,
    query_model: &Model<String>,
    dismissed_query_model: &Model<String>,
    active_item_id_model: &Model<Option<Arc<str>>>,
    active: TextAssistMatch,
    on_accept: Option<&OnTextAssistFieldAccept>,
) {
    let next_query = active.label.as_ref().to_string();
    set_string_model_if_changed(host, query_model, &next_query);
    set_string_model_if_changed(host, dismissed_query_model, &next_query);
    set_active_item_if_changed(host, active_item_id_model, &active.item_id);
    if let Some(on_accept) = on_accept {
        on_accept(host, action_cx, active);
    }
    host.request_redraw(action_cx.window);
}

fn set_string_model_if_changed(
    host: &mut dyn UiActionHost,
    model: &Model<String>,
    next: &str,
) -> bool {
    let unchanged = host
        .models_mut()
        .read(model, |value| value == next)
        .unwrap_or(false);
    if unchanged {
        return false;
    }

    host.models_mut()
        .update(model, |value| {
            value.clear();
            value.push_str(next);
        })
        .is_ok()
}

fn set_active_item_if_changed(
    host: &mut dyn UiActionHost,
    model: &Model<Option<Arc<str>>>,
    next: &Arc<str>,
) -> bool {
    let unchanged = host
        .models_mut()
        .read(model, |value| {
            value
                .as_ref()
                .is_some_and(|current| current.as_ref() == next.as_ref())
        })
        .unwrap_or(false);
    if unchanged {
        return false;
    }

    host.models_mut()
        .update(model, |value| *value = Some(next.clone()))
        .is_ok()
}

#[cfg(test)]
mod tests {
    use fret_app::App;
    use fret_core::AppWindowId;
    use fret_ui::GlobalElementId;
    use fret_ui::action::{ActionCx, UiActionHostAdapter};

    use super::*;

    fn text_match(id: &str, label: &str) -> TextAssistMatch {
        TextAssistMatch {
            item_id: Arc::from(id),
            label: Arc::from(label),
            score: 1.0,
            source_index: 0,
            disabled: false,
        }
    }

    #[test]
    fn accept_match_does_not_bump_already_committed_models() {
        let mut app = App::new();
        let query = app.models_mut().insert(String::from("Cube"));
        let dismissed_query = app.models_mut().insert(String::from("Cube"));
        let active_item_id = app.models_mut().insert(Some(Arc::from("cube")));
        let query_revision = query.revision(&app);
        let dismissed_revision = dismissed_query.revision(&app);
        let active_revision = active_item_id.revision(&app);

        {
            let mut host = UiActionHostAdapter { app: &mut app };
            accept_text_assist_match(
                &mut host,
                ActionCx {
                    window: AppWindowId::default(),
                    target: GlobalElementId(7),
                },
                &query,
                &dismissed_query,
                &active_item_id,
                text_match("cube", "Cube"),
                None,
            );
        }

        assert_eq!(query.revision(&app), query_revision);
        assert_eq!(dismissed_query.revision(&app), dismissed_revision);
        assert_eq!(active_item_id.revision(&app), active_revision);
    }

    #[test]
    fn accept_match_keeps_current_active_item_revision() {
        let mut app = App::new();
        let query = app.models_mut().insert(String::from("ca"));
        let dismissed_query = app.models_mut().insert(String::new());
        let active_item_id = app.models_mut().insert(Some(Arc::from("capsule")));
        let query_revision = query.revision(&app);
        let dismissed_revision = dismissed_query.revision(&app);
        let active_revision = active_item_id.revision(&app);

        {
            let mut host = UiActionHostAdapter { app: &mut app };
            accept_text_assist_match(
                &mut host,
                ActionCx {
                    window: AppWindowId::default(),
                    target: GlobalElementId(7),
                },
                &query,
                &dismissed_query,
                &active_item_id,
                text_match("capsule", "Capsule"),
                None,
            );
        }

        assert_ne!(query.revision(&app), query_revision);
        assert_ne!(dismissed_query.revision(&app), dismissed_revision);
        assert_eq!(active_item_id.revision(&app), active_revision);
        assert_eq!(
            app.models_mut().read(&query, Clone::clone).unwrap(),
            "Capsule"
        );
        assert_eq!(
            app.models_mut()
                .read(&dismissed_query, Clone::clone)
                .unwrap(),
            "Capsule"
        );
    }
}
