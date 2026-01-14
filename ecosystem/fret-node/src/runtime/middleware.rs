//! Store middleware hooks for `NodeGraphStore`.
//!
//! This module is intentionally headless-safe (no `fret-ui` dependency).

use crate::ops::GraphTransaction;
use crate::profile::ApplyPipelineError;
use crate::runtime::changes::NodeGraphChanges;
use crate::runtime::events::NodeGraphStoreSnapshot;

pub trait NodeGraphStoreMiddleware: 'static {
    fn before_dispatch(
        &mut self,
        _snapshot: NodeGraphStoreSnapshot<'_>,
        _tx: &mut GraphTransaction,
    ) -> Result<(), ApplyPipelineError> {
        Ok(())
    }

    fn after_dispatch(
        &mut self,
        _snapshot: NodeGraphStoreSnapshot<'_>,
        _committed: &GraphTransaction,
        _changes: &NodeGraphChanges,
    ) {
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct NoopNodeGraphStoreMiddleware;

impl NodeGraphStoreMiddleware for NoopNodeGraphStoreMiddleware {}

#[derive(Debug, Clone)]
pub struct NodeGraphStoreMiddlewareChain<A, B> {
    pub first: A,
    pub second: B,
}

impl<A, B> NodeGraphStoreMiddlewareChain<A, B> {
    pub fn new(first: A, second: B) -> Self {
        Self { first, second }
    }
}

impl<A, B> NodeGraphStoreMiddleware for NodeGraphStoreMiddlewareChain<A, B>
where
    A: NodeGraphStoreMiddleware,
    B: NodeGraphStoreMiddleware,
{
    fn before_dispatch(
        &mut self,
        snapshot: NodeGraphStoreSnapshot<'_>,
        tx: &mut GraphTransaction,
    ) -> Result<(), ApplyPipelineError> {
        self.first.before_dispatch(snapshot, tx)?;
        self.second.before_dispatch(snapshot, tx)?;
        Ok(())
    }

    fn after_dispatch(
        &mut self,
        snapshot: NodeGraphStoreSnapshot<'_>,
        committed: &GraphTransaction,
        changes: &NodeGraphChanges,
    ) {
        self.first.after_dispatch(snapshot, committed, changes);
        self.second.after_dispatch(snapshot, committed, changes);
    }
}
