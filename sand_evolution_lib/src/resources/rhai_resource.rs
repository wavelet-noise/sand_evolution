#[derive(Debug)]
pub struct RhaiResourceStorage {
    pub engine: rhai::Engine,
    pub scope: rhai::Scope<'static>,
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
