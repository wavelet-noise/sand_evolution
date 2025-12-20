use crate::ecs::systems::{EntityScriptSystem, GravitySystem, MoveSystem};
use crate::evolution_app::EvolutionApp;
use crate::resources::rhai_resource::{RhaiResource, RhaiResourceStorage};
use crate::rhai_lib;
use crate::shared_state::SharedState;
use crate::{cs, State};
use specs::RunNow;
use std::cell::RefCell;
use std::rc::Rc;

fn set_frame_vars(state: &mut State, storage: &mut RhaiResourceStorage) {
    // Make `time` deterministic and tied to simulation time (starts from 0).
    // If scripts need wall clock, they should derive it themselves.
    let frame_time = state.sim_time_seconds as f32;
    storage.scope.set_value("time", frame_time);
    storage
        .scope
        .set_value("sim_time", state.sim_time_seconds as f32);
    storage
        .scope
        .set_value("time_of_day", state.day_night.time_of_day_seconds);
    storage
        .scope
        .set_value("day_length", state.day_night.day_length_seconds);
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

        // Advance simulation time + day/night cycle (simulation-time based).
        state.sim_time_seconds += one_tick_delta;
        let (sun_x, sun_y) = state.day_night.advance(one_tick_delta as f32);
        state.world_settings.sun_dir_x = sun_x;
        state.world_settings.sun_dir_y = sun_y;
        // Shadow params for shader:
        // - shadow_strength: strength (0..2), where >1 pushes shadows towards pure black
        // - shadow_length_steps: length in raymarch steps (1..64)
        // - shadow_distance_falloff: distance falloff exponent (0 disables distance attenuation)
        state.world_settings.shadow_strength = state.day_night.shadow_strength.clamp(0.0, 2.0);
        state.world_settings.shadow_length_steps =
            state.day_night.shadow_length_steps.clamp(1.0, 64.0);
        state.world_settings.shadow_distance_falloff =
            state.day_night.shadow_distance_falloff.clamp(0.0, 4.0);

        if state.toggled {
            // Set the tick variable in scope and update state pointer
            {
                if let Some(rhai_resource) = world.get_mut::<RhaiResource>() {
                    if let Some(storage) = &mut rhai_resource.storage {
                        storage.scope.set_value("tick", state.tick);
                        storage
                            .scope
                            .set_value("sim_time", state.sim_time_seconds as f32);
                        storage
                            .scope
                            .set_value("time_of_day", state.day_night.time_of_day_seconds);
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

        if state.tick % 2 == 0 {
            let iters = evolution_app
                .cell_diffusion_iterations
                .clamp(1, 48) as usize;
            for _ in 0..iters {
                state.diffuse_temperature_fast();
            }
            state.diffuse_ambient_temperature_fast();
        }

        // Create temperature context ONCE before the loop for reuse
        // Use a pointer to work around borrowing issues
        let state_ptr: *mut State = state;
        let mut temp_context = crate::cells::TemperatureContext {
            get_temp: Box::new(move |x: cs::PointType, y: cs::PointType| unsafe {
                (*state_ptr).get_ambient_temperature(x, y)
            }),
            add_temp: Box::new(
                move |x: cs::PointType, y: cs::PointType, delta: f32| unsafe {
                    (*state_ptr).add_temperature(x, y, delta);
                },
            ),
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
                // acid (9), gas (10), burning gas (11), diluted acid (12), liquid gas (17),
                // wood (50), ice (55), crushed ice (56), snow (57)
                // For other cells pass None for optimization
                let needs_temp = cur_v == 2
                    || cur_v == 3
                    || cur_v == 4
                    || cur_v == 6
                    || cur_v == 7
                    || cur_v == 8
                    || cur_v == 9
                    || cur_v == 10
                    || cur_v == 11
                    || cur_v == 12
                    || cur_v == 17
                    || cur_v == 50
                    || cur_v == 55
                    || cur_v == 56
                    || cur_v == 57;

                state.pal_container.pal[cur_v as usize].update(
                    i,
                    j,
                    cur,
                    state.diffuse_rgba.as_mut(),
                    &state.pal_container,
                    &mut state.prng,
                    if needs_temp {
                        Some(&mut temp_context)
                    } else {
                        None
                    },
                );
            }
        }
    }
}
