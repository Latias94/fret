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
    fn handle_event<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        ctx: &NodeGraphCanvasMiddlewareCx<'_>,
        event: &Event,
    ) -> NodeGraphCanvasEventOutcome {
        match self.first.handle_event(cx, ctx, event) {
            NodeGraphCanvasEventOutcome::NotHandled => self.second.handle_event(cx, ctx, event),
            other => other,
        }
    }

    fn handle_command<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        ctx: &NodeGraphCanvasMiddlewareCx<'_>,
        command: &CommandId,
    ) -> NodeGraphCanvasCommandOutcome {
        match self.first.handle_command(cx, ctx, command) {
            NodeGraphCanvasCommandOutcome::NotHandled => {
                self.second.handle_command(cx, ctx, command)
            }
            other => other,
        }
    }

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
