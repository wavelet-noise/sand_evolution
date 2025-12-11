use specs::Component;

#[derive(Debug)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Component for Position {
    type Storage = specs::VecStorage<Self>;
}

/// Hierarchy parent pointer (for editor scene graph).
#[derive(Debug, Clone, Copy)]
pub struct Parent {
    pub entity: specs::Entity,
}

impl Component for Parent {
    type Storage = specs::HashMapStorage<Self>;
}

/// Hierarchy children list (for editor scene graph).
#[derive(Debug, Clone, Default)]
pub struct Children {
    pub entities: Vec<specs::Entity>,
}

impl Component for Children {
    type Storage = specs::HashMapStorage<Self>;
}

#[derive(Debug)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

impl Component for Velocity {
    type Storage = specs::VecStorage<Self>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptType {
    World,   // World script
    Entity,  // Object script
}

#[derive(Debug)]
pub struct Script {
    pub script: String,
    pub ast: Option<rhai::AST>,
    pub raw: bool,
    pub script_type: ScriptType,
    /// If true, the script should execute only once: on the first tick after it becomes runnable.
    pub run_once: bool,
    /// Internal flag for one-shot scripts to prevent re-running.
    pub has_run: bool,
}

impl Default for Script {
    fn default() -> Self {
        Self {
            script: "".to_owned(),
            ast: None,
            raw: true,
            script_type: ScriptType::Entity,
            run_once: false,
            has_run: false,
        }
    }
}

impl Component for Script {
    type Storage = specs::HashMapStorage<Self>;
}

#[derive(Debug, Clone)]
pub struct Name {
    pub name: String,
}

impl Component for Name {
    type Storage = specs::HashMapStorage<Self>;
}

#[derive(Debug, Clone, Copy)]
pub struct Rotation {
    pub angle: f32, // in radians
}

impl Default for Rotation {
    fn default() -> Self {
        Self { angle: 0.0 }
    }
}

impl Component for Rotation {
    type Storage = specs::VecStorage<Self>;
}

#[derive(Debug, Clone, Copy)]
pub struct Scale {
    pub x: f32,
    pub y: f32,
}

impl Default for Scale {
    fn default() -> Self {
        Self { x: 1.0, y: 1.0 }
    }
}

impl Component for Scale {
    type Storage = specs::VecStorage<Self>;
}
