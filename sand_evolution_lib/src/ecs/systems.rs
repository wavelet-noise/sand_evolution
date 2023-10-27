use crate::ecs::components::{Position, Script, Velocity};
use crate::resources::rhai_resource::RhaiResource;
use specs::{Join, Read, ReadStorage, System, Write, WriteStorage};

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
        WriteStorage<'a, Script>,
        ReadStorage<'a, Position>,
        Write<'a, RhaiResource>,
    );

    fn run(&mut self, (mut scripts, positions, mut rhai_resource): Self::SystemData) {
        if let Some(rhai) = &mut rhai_resource.storage {
            Self::compile_and_run_scripts(&mut scripts, &positions, &rhai.engine, &mut rhai.scope);
        } else {
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

    // Helper function to compile scripts
    fn compile_and_run_scripts(
        scripts: &mut WriteStorage<Script>,
        positions: &ReadStorage<Position>,
        engine: &rhai::Engine,   // Assuming Engine is the type of your engine
        scope: &mut rhai::Scope, // Assuming Scope is the type of your scope
    ) {
        for (script, _position) in (scripts, positions).join() {
            if script.raw {
                script.ast = Self::compile_script(engine, scope, &script.script);
            }

            if let Some(ast) = &script.ast {
                Self::run_script(engine, scope, ast);
            }
        }
    }
}
