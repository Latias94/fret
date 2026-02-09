use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_ui::element::{AnyElement, SemanticsDecoration};

pub trait AnyElementSemanticsExt {
    fn a11y(self, decoration: SemanticsDecoration) -> AnyElement;
    fn a11y_role(self, role: SemanticsRole) -> AnyElement;
    fn a11y_label(self, label: impl Into<Arc<str>>) -> AnyElement;
    fn test_id(self, id: impl Into<Arc<str>>) -> AnyElement;
    fn a11y_value(self, value: impl Into<Arc<str>>) -> AnyElement;
    fn a11y_disabled(self, disabled: bool) -> AnyElement;
    fn a11y_selected(self, selected: bool) -> AnyElement;
    fn a11y_expanded(self, expanded: bool) -> AnyElement;
    fn a11y_checked(self, checked: Option<bool>) -> AnyElement;
}

impl AnyElementSemanticsExt for AnyElement {
    fn a11y(self, decoration: SemanticsDecoration) -> AnyElement {
        self.attach_semantics(decoration)
    }

    fn a11y_role(self, role: SemanticsRole) -> AnyElement {
        self.a11y(SemanticsDecoration::default().role(role))
    }

    fn a11y_label(self, label: impl Into<Arc<str>>) -> AnyElement {
        self.a11y(SemanticsDecoration::default().label(label))
    }

    fn test_id(self, id: impl Into<Arc<str>>) -> AnyElement {
        self.a11y(SemanticsDecoration::default().test_id(id))
    }

    fn a11y_value(self, value: impl Into<Arc<str>>) -> AnyElement {
        self.a11y(SemanticsDecoration::default().value(value))
    }

    fn a11y_disabled(self, disabled: bool) -> AnyElement {
        self.a11y(SemanticsDecoration::default().disabled(disabled))
    }

    fn a11y_selected(self, selected: bool) -> AnyElement {
        self.a11y(SemanticsDecoration::default().selected(selected))
    }

    fn a11y_expanded(self, expanded: bool) -> AnyElement {
        self.a11y(SemanticsDecoration::default().expanded(expanded))
    }

    fn a11y_checked(self, checked: Option<bool>) -> AnyElement {
        self.a11y(SemanticsDecoration::default().checked(checked))
    }
}
