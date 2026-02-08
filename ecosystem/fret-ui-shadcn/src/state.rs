use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use crate::{Alert, AlertDescription, AlertTitle, AlertVariant, Badge, BadgeVariant};

#[cfg(feature = "state-query")]
use fret_query::{QueryState, QueryStatus};
#[cfg(feature = "state-selector")]
use fret_selector::ui::SelectorElementContextExt as _;

#[cfg(feature = "state-selector")]
use std::any::Any;

#[cfg(feature = "state-selector")]
#[track_caller]
pub fn use_selector_badge<H, Deps, TValue>(
    cx: &mut ElementContext<'_, H>,
    variant: BadgeVariant,
    deps: impl FnOnce(&mut ElementContext<'_, H>) -> Deps,
    compute: impl FnOnce(&mut ElementContext<'_, H>) -> TValue,
) -> AnyElement
where
    H: UiHost,
    Deps: Any + PartialEq,
    TValue: Any + Clone + ToString,
{
    let value = cx.use_selector(deps, compute);
    Badge::new(value.to_string())
        .variant(variant)
        .into_element(cx)
}

#[cfg(feature = "state-query")]
pub fn query_status_badge<H: UiHost, T>(
    cx: &mut ElementContext<'_, H>,
    state: &QueryState<T>,
) -> AnyElement {
    let (variant, label) = match state.status {
        QueryStatus::Idle => (BadgeVariant::Secondary, "Idle"),
        QueryStatus::Loading => (BadgeVariant::Secondary, "Loading"),
        QueryStatus::Success => (BadgeVariant::Default, "Ready"),
        QueryStatus::Error => (BadgeVariant::Destructive, "Error"),
    };

    Badge::new(label).variant(variant).into_element(cx)
}

#[cfg(feature = "state-query")]
pub fn query_error_alert<H: UiHost, T>(
    cx: &mut ElementContext<'_, H>,
    state: &QueryState<T>,
) -> Option<AnyElement> {
    let error = state.error.as_ref()?;

    let title = AlertTitle::new("Request failed").into_element(cx);
    let description = AlertDescription::new(error.to_string()).into_element(cx);

    Some(
        Alert::new([title, description])
            .variant(AlertVariant::Destructive)
            .into_element(cx),
    )
}
