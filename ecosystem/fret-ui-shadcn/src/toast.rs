//! Deprecated Toast surface (shadcn/ui v4).
//!
//! Upstream note: the Radix Toast-based component is deprecated in shadcn/ui v4 in favor of
//! Sonner. The docs keep a `toast` page that points to `sonner`.
//!
//! Reference:
//! - `repo-ref/ui/apps/v4/content/docs/components/toast.mdx`
//!
//! In Fret we expose the Sonner-shaped API from [`crate::sonner`] under the `toast` module so
//! downstream code can keep importing a `toast` surface while migrating.

pub use crate::sonner::{
    Sonner, ToastAction, ToastIconOverride, ToastIconOverrides, ToastId, ToastMessageOptions,
    ToastOffset, ToastPosition, ToastPromise, ToastPromiseAsyncOptions, ToastPromiseHandle,
    ToastPromiseUnwrapError, ToastRequest, ToastVariant, Toaster,
};
