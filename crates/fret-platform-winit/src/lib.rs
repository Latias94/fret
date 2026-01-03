#[cfg(feature = "accesskit")]
pub mod accessibility;

#[cfg(not(feature = "accesskit"))]
#[path = "accessibility_stub.rs"]
pub mod accessibility;
