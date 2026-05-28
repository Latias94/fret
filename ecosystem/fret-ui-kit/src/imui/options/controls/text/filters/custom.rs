use std::sync::Arc;

#[derive(Clone)]
pub struct InputTextCustomFilter {
    filter: Arc<dyn Fn(&str) -> String + 'static>,
}

impl InputTextCustomFilter {
    pub fn new(filter: impl Fn(&str) -> String + 'static) -> Self {
        Self {
            filter: Arc::new(filter),
        }
    }

    pub fn filter_text(&self, text: &str) -> String {
        (self.filter)(text)
    }
}

impl std::fmt::Debug for InputTextCustomFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InputTextCustomFilter")
            .finish_non_exhaustive()
    }
}
