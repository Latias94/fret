use super::*;

#[derive(Debug, Clone)]
pub struct NodeGraphCanvasMiddlewareChain<A, B> {
    pub first: A,
    pub second: B,
}

impl<A, B> NodeGraphCanvasMiddlewareChain<A, B> {
    pub fn new(first: A, second: B) -> Self {
        Self { first, second }
    }
}

impl<A, B> NodeGraphCanvasMiddleware for NodeGraphCanvasMiddlewareChain<A, B>
where
    A: NodeGraphCanvasMiddleware,
    B: NodeGraphCanvasMiddleware,
{
    fn before_commit<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        ctx: &NodeGraphCanvasMiddlewareCx<'_>,
        tx: &mut GraphTransaction,
    ) -> NodeGraphCanvasCommitOutcome {
        match self.first.before_commit(host, window, ctx, tx) {
            NodeGraphCanvasCommitOutcome::Continue => {
                self.second.before_commit(host, window, ctx, tx)
            }
            other => other,
        }
    }
}
