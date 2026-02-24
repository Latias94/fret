#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiOverlaySynthesisKindV1 {
    Modal,
    Popover,
    Hover,
    Tooltip,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiOverlaySynthesisSourceV1 {
    CachedDeclaration,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiOverlaySynthesisOutcomeV1 {
    Synthesized,
    SuppressedMissingTrigger,
    SuppressedTriggerNotLiveInCurrentFrame,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UiOverlaySynthesisEventV1 {
    pub kind: UiOverlaySynthesisKindV1,
    pub id: u64,
    pub source: UiOverlaySynthesisSourceV1,
    pub outcome: UiOverlaySynthesisOutcomeV1,
}

impl UiOverlaySynthesisEventV1 {
    fn from_event(e: fret_ui_kit::OverlaySynthesisEvent) -> Self {
        use fret_ui_kit::OverlaySynthesisKind;
        use fret_ui_kit::OverlaySynthesisOutcome;
        use fret_ui_kit::OverlaySynthesisSource;

        let kind = match e.kind {
            OverlaySynthesisKind::Modal => UiOverlaySynthesisKindV1::Modal,
            OverlaySynthesisKind::Popover => UiOverlaySynthesisKindV1::Popover,
            OverlaySynthesisKind::Hover => UiOverlaySynthesisKindV1::Hover,
            OverlaySynthesisKind::Tooltip => UiOverlaySynthesisKindV1::Tooltip,
        };
        let source = match e.source {
            OverlaySynthesisSource::CachedDeclaration => {
                UiOverlaySynthesisSourceV1::CachedDeclaration
            }
        };
        let outcome = match e.outcome {
            OverlaySynthesisOutcome::Synthesized => UiOverlaySynthesisOutcomeV1::Synthesized,
            OverlaySynthesisOutcome::SuppressedMissingTrigger => {
                UiOverlaySynthesisOutcomeV1::SuppressedMissingTrigger
            }
            OverlaySynthesisOutcome::SuppressedTriggerNotLiveInCurrentFrame => {
                UiOverlaySynthesisOutcomeV1::SuppressedTriggerNotLiveInCurrentFrame
            }
        };

        Self {
            kind,
            id: e.id.0,
            source,
            outcome,
        }
    }
}
