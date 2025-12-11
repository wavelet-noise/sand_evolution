use crate::ecs::components::{Position, Script, Velocity};
use crate::resources::rhai_resource::RhaiResource;
use specs::{Entities, Join, ReadStorage, System, Write, WriteStorage};

pub struct MoveSystem;

impl<'a> System<'a> for MoveSystem {
    type SystemData = (ReadStorage<'a, Velocity>, WriteStorage<'a, Position>);

    fn run(&mut self, (vel, mut pos): Self::SystemData) {
        for (vel, pos) in (&vel, &mut pos).join() {
            pos.x += vel.x;
            pos.y += vel.y;
        }
    }
}

pub struct GravitySystem;

impl<'a> System<'a> for GravitySystem {
    type SystemData = (WriteStorage<'a, Velocity>, ReadStorage<'a, Position>);

    fn run(&mut self, (mut vel, pos): Self::SystemData) {
        for (vel, pos) in (&mut vel, &pos).join() {
            vel.y -= 0.1;
        }
    }
}

pub struct EntityScriptSystem;

impl<'a> System<'a> for EntityScriptSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Script>,
        ReadStorage<'a, Position>,
        Write<'a, RhaiResource>,
    );

    fn run(&mut self, (entities, mut scripts, positions, mut rhai_resource): Self::SystemData) {
        if let Some(rhai) = &mut rhai_resource.storage {
            // Unified processing of all scripts
            Self::compile_and_run_all_scripts(&entities, &mut scripts, &positions, &rhai.engine, &mut rhai.scope);
        }
    }
}

impl EntityScriptSystem {
    // Helper function to compile a single script
    fn compile_script(
        engine: &rhai::Engine,
        scope: &rhai::Scope,
        script_text: &str,
    ) -> Option<rhai::AST> {
        match engine.compile_with_scope(scope, script_text) {
            Ok(val) => Some(val),
            Err(_) => None,
        }
    }

    fn run_script(engine: &rhai::Engine, scope: &mut rhai::Scope, ast: &rhai::AST) {
        match engine.run_ast_with_scope(scope, ast) {
            Ok(_) => {}
            Err(_) => {}
        }
    }

    // Unified function for compiling and executing all scripts
    fn compile_and_run_all_scripts(
        entities: &Entities,
        scripts: &mut WriteStorage<Script>,
        positions: &ReadStorage<Position>,
        engine: &rhai::Engine,
        scope: &mut rhai::Scope,
    ) {
        // First, compile all scripts that need to be compiled
        let mut entities_to_compile: Vec<(specs::Entity, String)> = Vec::new();
        {
            // Use a temporary reference for reading
            let scripts_read: &WriteStorage<Script> = scripts;
            for (entity, script) in (entities, scripts_read).join() {
                if script.raw {
                    entities_to_compile.push((entity, script.script.clone()));
                }
            }
        }
        
        for (entity, script_text) in entities_to_compile {
            // Now we can get mutable access
            if let Some(script) = scripts.get_mut(entity) {
                script.ast = Self::compile_script(engine, scope, &script_text);
                script.raw = false;
            }
        }

        // Then execute all compiled scripts
        {
            let scripts_read: &WriteStorage<Script> = scripts;
            for (_entity, script) in (entities, scripts_read).join() {
                if let Some(ast) = &script.ast {
                    Self::run_script(engine, scope, ast);
                }
            }
        }
    }
}
