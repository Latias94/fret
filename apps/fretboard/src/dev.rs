pub(crate) mod contracts;
mod native;
mod web;

pub(crate) use native::run_native_contract;
pub(crate) use web::run_web_contract;

fn resolve_bool_override(enabled: bool, disabled: bool) -> Option<bool> {
    match (enabled, disabled) {
        (true, false) => Some(true),
        (false, true) => Some(false),
        _ => None,
    }
}
