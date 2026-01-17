use crate::ecs::components::{Name, Position, Script, Velocity};
use crate::resources::rhai_resource::RhaiResource;
use specs::{Entities, Join, ReadStorage, System, Write, WriteStorage};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

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
        ReadStorage<'a, Name>,
        Write<'a, RhaiResource>,
    );

    fn run(
        &mut self,
        (entities, mut scripts, positions, names, mut rhai_resource): Self::SystemData,
    ) {
        if let Some(rhai) = &mut rhai_resource.storage {
            // Unified processing of all scripts
            Self::compile_and_run_all_scripts(
                &entities,
                &mut scripts,
                &positions,
                &names,
                &rhai.engine,
                &mut rhai.scope,
                &rhai.script_log,
            );
        }
    }
}

impl EntityScriptSystem {
    // Helper function to compile a single script
    fn compile_script(
        engine: &rhai::Engine,
        scope: &rhai::Scope,
        script_text: &str,
        script_log: &Rc<RefCell<VecDeque<String>>>,
    ) -> Option<rhai::AST> {
        match engine.compile_with_scope(scope, script_text) {
            Ok(val) => Some(val),
            Err(err) => {
                // Mirror compile errors to Script Log so failures aren't silent.
                let mut log = script_log.borrow_mut();
                if log.len() >= 30 {
                    log.pop_front();
                }
                log.push_back(format!("Rhai compile error: {err}"));
                None
            }
        }
    }

    fn run_script(
        engine: &rhai::Engine,
        scope: &mut rhai::Scope,
        ast: &rhai::AST,
        script_log: &Rc<RefCell<VecDeque<String>>>,
    ) {
        match engine.run_ast_with_scope(scope, ast) {
            Ok(_) => {}
            Err(err) => {
                let mut log = script_log.borrow_mut();
                if log.len() >= 30 {
                    log.pop_front();
                }
                log.push_back(format!("Rhai runtime error: {err}"));
            }
        }
    }

    // Unified function for compiling and executing all scripts
    fn compile_and_run_all_scripts(
        entities: &Entities,
        scripts: &mut WriteStorage<Script>,
        positions: &ReadStorage<Position>,
        names: &ReadStorage<Name>,
        engine: &rhai::Engine,
        scope: &mut rhai::Scope,
        script_log: &Rc<RefCell<VecDeque<String>>>,
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
                script.ast = Self::compile_script(engine, scope, &script_text, script_log);
                script.raw = false;
            }
        }

        // Then execute all compiled scripts
        // Scripts with run_once=false execute every frame
        // Scripts with run_once=true execute only once
        let mut entities_to_run: Vec<specs::Entity> = Vec::new();
        {
            let scripts_read: &WriteStorage<Script> = scripts;
            for (entity, script) in (entities, scripts_read).join() {
                let runnable = script.ast.is_some();
                // For run_once=false scripts, always run (execute every frame)
                // For run_once=true scripts, only run if not yet executed
                let should_run = if script.run_once {
                    !script.has_run
                } else {
                    true // Always run scripts that are not one-shot
                };
                if runnable && should_run {
                    entities_to_run.push(entity);
                }
            }
        }

        for entity in entities_to_run {
            if let Some(script) = scripts.get_mut(entity) {
                if let Some(ast) = script.ast.as_ref() {
                    Self::run_script(engine, scope, ast, script_log);
                    if script.run_once {
                        script.has_run = true;
                    }
                }
            }
        }
    }
}
