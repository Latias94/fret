use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct TabBarOptions {
    pub selected: Option<fret_runtime::Model<Option<Arc<str>>>>,
    pub gap: crate::MetricRef,
    pub test_id: Option<Arc<str>>,
}

impl Default for TabBarOptions {
    fn default() -> Self {
        Self {
            selected: None,
            gap: crate::MetricRef::space(crate::Space::N1),
            test_id: None,
        }
    }
}

impl TabBarOptions {
    pub fn selected_model(mut self, selected: impl crate::imui::IntoImUiOptionalTextModel) -> Self {
        self.selected = Some(selected.into_imui_optional_text_model());
        self
    }

    pub fn test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_runtime::ModelStore;

    #[test]
    fn tab_bar_options_accepts_narrow_imui_selected_model_bridge() {
        let mut models = ModelStore::default();
        let selected = models.insert(Some(Arc::<str>::from("inspector")));

        let options = TabBarOptions::default()
            .selected_model(&selected)
            .test_id("tabs.root");

        assert_eq!(
            options.selected.as_ref().map(|model| model.id()),
            Some(selected.id())
        );
        assert_eq!(options.test_id.as_deref(), Some("tabs.root"));
    }
}
