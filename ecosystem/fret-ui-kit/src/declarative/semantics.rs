use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_ui::element::{AnyElement, SemanticsDecoration};

use crate::UiIntoElement;

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

#[derive(Debug, Clone)]
pub struct UiIntoElementWithTestId<T> {
    inner: T,
    test_id: Arc<str>,
}

impl<T> UiIntoElementWithTestId<T> {
    pub fn new(inner: T, test_id: impl Into<Arc<str>>) -> Self {
        Self {
            inner,
            test_id: test_id.into(),
        }
    }
}

impl<T: UiIntoElement> UiIntoElement for UiIntoElementWithTestId<T> {
    #[track_caller]
    fn into_element<H: fret_ui::UiHost>(
        self,
        cx: &mut fret_ui::ElementContext<'_, H>,
    ) -> fret_ui::element::AnyElement {
        self.inner.into_element(cx).test_id(self.test_id)
    }
}

pub trait UiIntoElementTestIdExt: UiIntoElement + Sized {
    fn test_id(self, id: impl Into<Arc<str>>) -> UiIntoElementWithTestId<Self> {
        UiIntoElementWithTestId::new(self, id)
    }
}

impl<T: UiIntoElement> UiIntoElementTestIdExt for T {}
