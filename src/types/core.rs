use std::{any::Any, cell::RefCell, collections::HashSet, rc::Rc};

use crate::utils::arena::{Arena, NodeId, ScopeId, SignalId};

pub struct ScopeData {
    pub parent: Option<ScopeId>,
    pub children: Vec<ScopeId>,
    pub signals: Vec<SignalId>,
    pub nodes: Vec<NodeId>,
    pub cleanups: Vec<Box<dyn FnOnce()>>,
}

pub struct SignalData {
    value: Box<dyn Any>,
    subscribers: HashSet<NodeId>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NodeState {
    Clean,
    Dirty,
    Computing,
}

pub struct ReactiveNode {
    owning_scope: ScopeId,
    computation_scope: Option<ScopeId>,

    state: NodeState,
    signal_dependencies: HashSet<SignalId>,
    node_dependencies: HashSet<NodeId>,
    subscribers: HashSet<NodeId>,

    computation: Rc<dyn Fn() -> Box<dyn Any>>,
    value: Option<Box<dyn Any>>,
}

pub struct ComponentOwner {
    pub signals: RefCell<Arena<SignalData>>,
    pub nodes: RefCell<Arena<ReactiveNode>>,
    pub scopes: RefCell<Arena<ScopeData>>,

    root_scope: ScopeId,
    active_scope: RefCell<ScopeId>,
    active_computation: RefCell<Option<NodeId>>,

    notifier: RefCell<Option<Rc<dyn Fn()>>>,
}
