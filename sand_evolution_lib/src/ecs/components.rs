use specs::Component;

#[derive(Debug)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Component for Position {
    type Storage = specs::VecStorage<Self>;
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
    World,   // Мировой скрипт
    Entity,  // Скрипт объекта
}

#[derive(Debug)]
pub struct Script {
    pub script: String,
    pub ast: Option<rhai::AST>,
    pub raw: bool,
    pub script_type: ScriptType,
}

impl Default for Script {
    fn default() -> Self {
        Self {
            script: "".to_owned(),
            ast: None,
            raw: true,
            script_type: ScriptType::Entity,
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
