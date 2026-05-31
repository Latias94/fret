use fret_core::Rect;
use fret_ui::GlobalElementId;

use crate::imui::ResponseExt;

/// Optional metadata reported by adapter seams for focus/geometry choreography.
#[derive(Debug, Clone, Copy, Default)]
pub struct AdapterSignalMetadata {
    rect: Option<Rect>,
    focus_restore_target: Option<GlobalElementId>,
}

impl AdapterSignalMetadata {
    pub fn new(rect: Option<Rect>, focus_restore_target: Option<GlobalElementId>) -> Self {
        Self {
            rect,
            focus_restore_target,
        }
    }

    pub fn rect(self) -> Option<Rect> {
        self.rect
    }

    pub fn focus_restore_target(self) -> Option<GlobalElementId> {
        self.focus_restore_target
    }
}

/// A single adapter signal report emitted after rendering a canonical wrapper.
#[derive(Debug, Clone, Copy, Default)]
pub struct AdapterSignalRecord {
    identity: Option<GlobalElementId>,
    response: ResponseExt,
    metadata: AdapterSignalMetadata,
}

impl AdapterSignalRecord {
    pub fn new(
        identity: Option<GlobalElementId>,
        response: ResponseExt,
        metadata: AdapterSignalMetadata,
    ) -> Self {
        Self {
            identity,
            response,
            metadata,
        }
    }

    pub fn identity(self) -> Option<GlobalElementId> {
        self.identity
    }

    pub fn response(self) -> ResponseExt {
        self.response
    }

    pub fn metadata(self) -> AdapterSignalMetadata {
        self.metadata
    }
}

/// Signal reporter callback used by adapter seams.
pub type AdapterSignalReporter<'a> = dyn FnMut(AdapterSignalRecord) + 'a;
