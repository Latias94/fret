use super::*;

impl CodeEditorHandle {
    pub fn feature_payload_snapshot(&self) -> CodeEditorFeaturePayloadSnapshot {
        self.state.borrow().feature_payloads.snapshot()
    }

    pub fn diagnostic_line_summaries(&self) -> Vec<DiagnosticLineSummary> {
        self.state
            .borrow()
            .feature_payloads
            .diagnostic_line_summaries()
            .to_vec()
    }

    pub fn set_diagnostic_spans(
        &self,
        spans: Vec<DiagnosticSpan>,
    ) -> Result<(), DiagnosticSpanError> {
        let mut st = self.state.borrow_mut();
        let normalized = normalized_diagnostic_spans(&st.buffer, &spans)?;
        let summaries = diagnostic_line_summaries(&st.buffer, &normalized)?;
        let buffer_revision = st.buffer.revision();
        if st
            .feature_payloads
            .set_diagnostic_spans(buffer_revision, normalized, summaries)
        {
            st.invalidate_feature_payload_paint_caches();
        }
        Ok(())
    }

    pub fn clear_diagnostic_spans(&self) {
        let _ = self.set_diagnostic_spans(Vec::new());
    }

    pub fn set_range_decorations(
        &self,
        decorations: Vec<RangeDecoration>,
    ) -> Result<(), RangeDecorationError> {
        let mut st = self.state.borrow_mut();
        let normalized = normalized_range_decorations(&st.buffer, &decorations)?;
        let buffer_revision = st.buffer.revision();
        if st
            .feature_payloads
            .set_range_decorations(buffer_revision, normalized)
        {
            st.invalidate_feature_payload_paint_caches();
        }
        Ok(())
    }

    pub fn clear_range_decorations(&self) {
        let _ = self.set_range_decorations(Vec::new());
    }

    pub fn set_gutter_markers(&self, markers: Vec<GutterMarker>) -> Result<(), GutterMarkerError> {
        let mut st = self.state.borrow_mut();
        validate_gutter_markers(&st.buffer, Some(&st.display_map), &markers)?;
        let normalized = normalized_gutter_markers(&markers);
        let buffer_revision = st.buffer.revision();
        let display_map_epoch = st.display_map_epoch;
        if st
            .feature_payloads
            .set_gutter_markers(buffer_revision, display_map_epoch, normalized)
        {
            st.invalidate_feature_payload_paint_caches();
        }
        Ok(())
    }

    pub fn clear_gutter_markers(&self) {
        let _ = self.set_gutter_markers(Vec::new());
    }

    pub fn set_semantic_tokens(
        &self,
        tokens: Vec<SemanticToken>,
    ) -> Result<(), SemanticTokenError> {
        let mut st = self.state.borrow_mut();
        let normalized = normalized_semantic_tokens(&st.buffer, &tokens)?;
        let buffer_revision = st.buffer.revision();
        if st
            .feature_payloads
            .set_semantic_tokens(buffer_revision, normalized)
        {
            st.invalidate_feature_payload_paint_caches();
        }
        Ok(())
    }

    pub fn clear_semantic_tokens(&self) {
        let _ = self.set_semantic_tokens(Vec::new());
    }
}
