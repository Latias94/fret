use std::any::Any;
use std::sync::Arc;

use fret_query::ui::QueryElementContextExt as _;
pub use fret_query::with_query_client;
pub use fret_query::{
    CancellationToken, QueryCancelMode, QueryClient, QueryHandle, QueryKey, QueryPolicy,
    QueryState, QueryStatus,
};
use fret_ui::UiHost;

use crate::UiWriter;

/// Query-style async resource helpers for authoring frontends.
///
/// This is a thin wrapper over `fret_query::ui::QueryElementContextExt` that works with any
/// authoring surface implementing `UiWriter`.
pub trait UiWriterQueryExt<H: UiHost>: UiWriter<H> {
    fn use_query<T: Any + Send + Sync + 'static>(
        &mut self,
        key: QueryKey<T>,
        policy: QueryPolicy,
        fetch: impl FnOnce(CancellationToken) -> Result<T, Arc<str>> + Send + 'static,
    ) -> QueryHandle<T>;

    fn invalidate_query<T: Any + Send + Sync + 'static>(&mut self, key: QueryKey<T>);

    fn invalidate_query_namespace(&mut self, namespace: &'static str);
}

impl<H: UiHost, W: UiWriter<H> + ?Sized> UiWriterQueryExt<H> for W {
    fn use_query<T: Any + Send + Sync + 'static>(
        &mut self,
        key: QueryKey<T>,
        policy: QueryPolicy,
        fetch: impl FnOnce(CancellationToken) -> Result<T, Arc<str>> + Send + 'static,
    ) -> QueryHandle<T> {
        self.with_cx_mut(|cx| cx.use_query(key, policy, fetch))
    }

    fn invalidate_query<T: Any + Send + Sync + 'static>(&mut self, key: QueryKey<T>) {
        self.with_cx_mut(|cx| {
            let _ = with_query_client(cx.app, |client, app| {
                client.invalidate(app, key);
            });
            cx.app.request_redraw(cx.window);
        });
    }

    fn invalidate_query_namespace(&mut self, namespace: &'static str) {
        self.with_cx_mut(|cx| {
            let _ = with_query_client(cx.app, |client, _app| {
                client.invalidate_namespace(namespace);
            });
            cx.app.request_redraw(cx.window);
        });
    }
}
