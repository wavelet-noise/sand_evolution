use specs::World;
use crate::editor::state::EditorState;

pub trait Command {
    fn execute(&mut self, world: &mut World, editor_state: &mut EditorState);
    fn undo(&mut self, world: &mut World, editor_state: &mut EditorState);
    fn name(&self) -> &str;
}

pub struct UndoRedo {
    undo_stack: Vec<Box<dyn Command>>,
    redo_stack: Vec<Box<dyn Command>>,
    max_history: usize,
}

impl UndoRedo {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history: 100,
        }
    }
    
    pub fn execute(&mut self, mut command: Box<dyn Command>, world: &mut World, editor_state: &mut EditorState) {
        command.execute(world, editor_state);
        self.undo_stack.push(command);
        self.redo_stack.clear();
        
        // Limit history size
        if self.undo_stack.len() > self.max_history {
            self.undo_stack.remove(0);
        }
    }
    
    pub fn undo(&mut self, world: &mut World, editor_state: &mut EditorState) -> bool {
        if let Some(mut command) = self.undo_stack.pop() {
            command.undo(world, editor_state);
            self.redo_stack.push(command);
            true
        } else {
            false
        }
    }
    
    pub fn redo(&mut self, world: &mut World, editor_state: &mut EditorState) -> bool {
        if let Some(mut command) = self.redo_stack.pop() {
            command.execute(world, editor_state);
            self.undo_stack.push(command);
            true
        } else {
            false
        }
    }
    
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }
    
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
    
    pub fn undo_name(&self) -> Option<String> {
        self.undo_stack.last().map(|c| c.name().to_string())
    }
    
    pub fn redo_name(&self) -> Option<String> {
        self.redo_stack.last().map(|c| c.name().to_string())
    }
}

// Example command implementations

pub struct MoveCommand {
    entity: specs::Entity,
    old_x: f32,
    old_y: f32,
    new_x: f32,
    new_y: f32,
}

impl MoveCommand {
    pub fn new(entity: specs::Entity, old_x: f32, old_y: f32, new_x: f32, new_y: f32) -> Self {
        Self {
            entity,
            old_x,
            old_y,
            new_x,
            new_y,
        }
    }
}

impl Command for MoveCommand {
    fn execute(&mut self, world: &mut World, _editor_state: &mut EditorState) {
        use crate::ecs::components::Position;
        use specs::WorldExt;
        
        if let Some(pos) = world.write_storage::<Position>().get_mut(self.entity) {
            pos.x = self.new_x;
            pos.y = self.new_y;
        }
    }
    
    fn undo(&mut self, world: &mut World, _editor_state: &mut EditorState) {
        use crate::ecs::components::Position;
        use specs::WorldExt;
        
        if let Some(pos) = world.write_storage::<Position>().get_mut(self.entity) {
            pos.x = self.old_x;
            pos.y = self.old_y;
        }
    }
    
    fn name(&self) -> &str {
        "Move"
    }
}
