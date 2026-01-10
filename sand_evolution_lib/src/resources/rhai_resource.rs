use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

#[derive(Debug)]
pub struct RhaiResourceStorage {
    pub engine: rhai::Engine,
    pub scope: rhai::Scope<'static>,
    pub state_ptr: std::cell::Cell<*mut crate::State>,
    pub script_log: Rc<RefCell<VecDeque<String>>>,
}

#[derive(Debug)]
pub struct RhaiResource {
    pub storage: Option<RhaiResourceStorage>,
}

impl Default for RhaiResource {
    fn default() -> Self {
        Self { storage: None }
    }
}
