use super::TextSystem;
use fret_core::{AttributedText, TextConstraints, TextMetrics, TextStyle};

impl TextSystem {
    pub fn measure(
        &mut self,
        text: &str,
        style: &TextStyle,
        constraints: TextConstraints,
    ) -> TextMetrics {
        self.layout_cache.measure.measure_plain(
            &mut self.parley_shaper,
            text,
            style,
            constraints,
            self.font_runtime.font_stack_key,
        )
    }

    pub fn measure_attributed(
        &mut self,
        rich: &AttributedText,
        base_style: &TextStyle,
        constraints: TextConstraints,
    ) -> TextMetrics {
        self.layout_cache.measure.measure_attributed(
            &mut self.parley_shaper,
            rich,
            base_style,
            constraints,
            self.font_runtime.font_stack_key,
        )
    }
}
