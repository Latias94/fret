//! Optional `fret-ui` sugar for reading hot literals from an `ElementContext`.

use fret_ui::{ElementContext, Invalidation, UiHost};

use super::HotLiterals;

pub trait HotLiteralsElementContextExt {
    fn hot_literal(&mut self, key: &str) -> Option<String>;
    fn hot_literal_or(&mut self, key: &str, fallback: &str) -> String;
}

impl<H: UiHost> HotLiteralsElementContextExt for ElementContext<'_, H> {
    fn hot_literal(&mut self, key: &str) -> Option<String> {
        self.observe_global::<HotLiterals>(Invalidation::Paint);
        HotLiterals::global(&*self.app)
            .get(key)
            .map(|v| v.to_string())
    }

    fn hot_literal_or(&mut self, key: &str, fallback: &str) -> String {
        self.hot_literal(key)
            .unwrap_or_else(|| fallback.to_string())
    }
}
