//! Shared authoring-state helpers for shadcn-style menu families.
//!
//! These keep the prop-style `checked/value + on...Change` surface aligned with the model-backed
//! surface without forcing each recipe to duplicate the same state plumbing.

use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::action::{ActionCx, UiActionHost};
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;

pub(crate) type OnCheckedChange = Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, bool) + 'static>;
pub(crate) type OnValueChange = Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, Arc<str>) + 'static>;

#[derive(Debug, Clone)]
pub enum MenuCheckboxChecked {
    Model(Model<bool>),
    Value(bool),
}

impl MenuCheckboxChecked {
    pub(crate) fn snapshot<H: UiHost>(&self, cx: &mut ElementContext<'_, H>) -> bool {
        match self {
            Self::Model(model) => cx.watch_model(model).copied().unwrap_or(false),
            Self::Value(value) => *value,
        }
    }

    pub(crate) fn toggle(&self, host: &mut dyn UiActionHost) -> bool {
        match self {
            Self::Model(model) => {
                let next = !host.models_mut().get_copied(model).unwrap_or(false);
                let _ = host.models_mut().update(model, |value| *value = next);
                next
            }
            Self::Value(value) => !*value,
        }
    }
}

#[derive(Debug, Clone)]
pub enum MenuRadioValue {
    Model(Model<Option<Arc<str>>>),
    Value(Option<Arc<str>>),
}

impl MenuRadioValue {
    pub(crate) fn snapshot<H: UiHost>(&self, cx: &mut ElementContext<'_, H>) -> Option<Arc<str>> {
        match self {
            Self::Model(model) => cx.watch_model(model).cloned().flatten(),
            Self::Value(value) => value.clone(),
        }
    }

    pub(crate) fn select(&self, host: &mut dyn UiActionHost, value: &Arc<str>) -> Option<Arc<str>> {
        match self {
            Self::Model(model) => {
                let current = host
                    .models_mut()
                    .read(model, |selected| selected.clone())
                    .ok()
                    .flatten();
                if current
                    .as_ref()
                    .is_some_and(|cur| cur.as_ref() == value.as_ref())
                {
                    return None;
                }
                let next = Some(value.clone());
                let _ = host
                    .models_mut()
                    .update(model, |selected| *selected = next.clone());
                next
            }
            Self::Value(current) => {
                if current
                    .as_ref()
                    .is_some_and(|cur| cur.as_ref() == value.as_ref())
                {
                    None
                } else {
                    Some(value.clone())
                }
            }
        }
    }
}
