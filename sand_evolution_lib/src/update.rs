use crate::evolution_app::EvolutionApp;
use crate::resources::rhai_resource::{RhaiResource, RhaiResourceStorage};
use crate::shared_state::SharedState;
use crate::{cs, State};
use crate::ecs::systems::{EntityScriptSystem, GravitySystem, MoveSystem};
use crate::rhai_lib;
use std::cell::RefCell;
use std::rc::Rc;
use specs::RunNow;

fn set_frame_vars(state: &mut State, storage: &mut RhaiResourceStorage) {
    let frame_start_time = (instant::now() - state.start_time) / 1000.0;

    storage.scope.set_value("time", frame_start_time);
    storage.scope.set_value("frame", state.frame);
    // Re-set GRID_WIDTH and GRID_HEIGHT after scope.clear() - scripts need these variables
    storage.scope.set_value("GRID_WIDTH", 1024i64);
    storage.scope.set_value("GRID_HEIGHT", 512i64);
}

pub fn update_tick(
    state: &mut State,
    sim_steps: i32,
    _dimensions: (u32, u32),
    evolution_app: &mut EvolutionApp,
    world: &mut specs::World,
    shared_state: &Rc<RefCell<SharedState>>,
    _update_start_time: f64,
) {
    //let mut output = ImageBuffer::new(texture_size.width, texture_size.height);
    let mut b_index = 0;
    state.frame += 1;

    const BUF_SIZE: usize = 50;
    let mut buf = [0u8; BUF_SIZE];
    _ = getrandom::getrandom(&mut buf);

    let one_tick_delta = 1.0 / evolution_app.simulation_steps_per_second as f64;
    
    // Set frame variables once before the loop
    if state.toggled {
        if let Some(rhai_resource) = world.get_mut::<RhaiResource>() {
            if let Some(storage) = &mut rhai_resource.storage {
                set_frame_vars(state, storage);
                // Update state pointer in thread_local
                let state_ptr: *mut State = state;
                storage.state_ptr.set(state_ptr);
                rhai_lib::set_state_ptr(state_ptr);
            }
        }
    }
    
    for _sim_update in 0..sim_steps {
        state.tick += 1;
        if state.toggled {
            // Set the tick variable in scope and update state pointer
            {
                if let Some(rhai_resource) = world.get_mut::<RhaiResource>() {
                    if let Some(storage) = &mut rhai_resource.storage {
                        storage.scope.set_value("tick", state.tick);
                        // Update state pointer each tick to ensure it's valid
                        let state_ptr: *mut State = state;
                        storage.state_ptr.set(state_ptr);
                        rhai_lib::set_state_ptr(state_ptr);
                        if state.tick % 500 == 0 {
                            storage.scope.clear();
                            set_frame_vars(state, storage);
                            // Re-set state pointer after clear
                            storage.state_ptr.set(state_ptr);
                            rhai_lib::set_state_ptr(state_ptr);
                        }
                    }
                }
            }
            
            // Execute ECS systems on each simulation tick
            // This includes EntityScriptSystem, which executes object scripts
            // Call systems after releasing the borrow of rhai_resource
            {
                use specs::WorldExt;
                let mut script_system = EntityScriptSystem;
                script_system.run_now(world);
                world.maintain();
                
                let mut gravity_system = GravitySystem;
                gravity_system.run_now(world);
                world.maintain();
                
                let mut move_system = MoveSystem;
                move_system.run_now(world);
                world.maintain();
            }
        }
        
        for (p, c) in shared_state.borrow_mut().points.iter() {
            if (0..cs::SECTOR_SIZE.x as i32).contains(&p.x)
                && (0..cs::SECTOR_SIZE.y as i32).contains(&p.y)
            {
                state
                    .diffuse_rgba
                    .put_pixel(p.x as u32, p.y as u32, image::Luma([*c]));
            }
        }
        shared_state.borrow_mut().points.clear();

        state.flip ^= 1;
        if state.flip == 0 {
            state.flop ^= 1;
        }

        state.prng.gen();

        // Fast temperature diffusion - processes all cells of the reduced grid each frame
        // Called less frequently for optimization (every 2 ticks)
        if state.tick % 2 == 0 {
            state.diffuse_temperature_fast();
        }

        // Create temperature context ONCE before the loop for reuse
        // Use a pointer to work around borrowing issues
        let state_ptr: *mut State = state;
        let mut temp_context = crate::cells::TemperatureContext {
            get_temp: Box::new(move |x: cs::PointType, y: cs::PointType| {
                unsafe { (*state_ptr).get_temperature(x, y) }
            }),
            add_temp: Box::new(move |x: cs::PointType, y: cs::PointType, delta: f32| {
                unsafe { (*state_ptr).add_temperature(x, y, delta); }
            }),
        };

        for i in (1..(cs::SECTOR_SIZE.x - 2 - state.flip)).rev().step_by(2) {
            for j in (1..(cs::SECTOR_SIZE.y - 2 - state.flop)).rev().step_by(2) {
                b_index += 1;
                if b_index >= BUF_SIZE {
                    b_index = 0;
                }

                // 21.5 % to skip each cell
                if buf[b_index] > 200 {
                    continue;
                }

                let cur = cs::xy_to_index(i, j);
                let cur_v = *state.diffuse_rgba.get(cur).unwrap();

                // Pass temp_context for cells with temperature interactions:
                // water (2), steam (3), fire (4), burning wood (6), burning coal (7), coal (8),
                // acid (9), diluted acid (12), wood (50), ice (55), crushed ice (56), snow (57)
                // For other cells pass None for optimization
                let needs_temp = cur_v == 2 || cur_v == 3 || cur_v == 4 || cur_v == 6 || 
                                 cur_v == 7 || cur_v == 8 || cur_v == 9 || cur_v == 12 ||
                                 cur_v == 50 || cur_v == 55 || cur_v == 56 || cur_v == 57;
                
                state.pal_container.pal[cur_v as usize].update(
                    i,
                    j,
                    cur,
                    state.diffuse_rgba.as_mut(),
                    &state.pal_container,
                    &mut state.prng,
                    if needs_temp { Some(&mut temp_context) } else { None },
                );
            }
        }
    }
}
