use super::core::{ComponentOwner, NodeState};
use crate::utils::arena::{NodeId, ScopeId};

pub struct ScopeGuard<'a> {
    owner: &'a ComponentOwner,
    prev_scope: ScopeId,
}

impl<'a> Drop for ScopeGuard<'a> {
    fn drop(&mut self) {
        *self.owner.active_scope.borrow_mut() = self.prev_scope;
    }
}

pub struct ComputationGuard<'a> {
    owner: &'a ComponentOwner,
    node_id: NodeId,
    prev_active: Option<NodeId>,
    pub completed: bool,
}

impl<'a> Drop for ComputationGuard<'a> {
    fn drop(&mut self) {
        *self.owner.active_computation.borrow_mut() = self.prev_active;

        if let Some(node) = self.owner.nodes.borrow_mut().get_mut(self.node_id) {
            node.state = if self.completed {
                NodeState::Clean
            } else {
                NodeState::Dirty
            };
        }
    }
}
