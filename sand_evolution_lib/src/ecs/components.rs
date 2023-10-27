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

#[derive(Debug)]
pub struct Script {
    pub script: String,
    pub ast: Option<rhai::AST>,
    pub raw: bool,
}

impl Default for Script {
    fn default() -> Self {
        Self {
            script: "".to_owned(),
            ast: None,
            raw: true,
        }
    }
}

impl Component for Script {
    type Storage = specs::HashMapStorage<Self>;
}
