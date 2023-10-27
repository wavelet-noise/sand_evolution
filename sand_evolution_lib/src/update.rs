use crate::evolution_app::EvolutionApp;
use crate::resources::rhai_resource::{RhaiResource, RhaiResourceStorage};
use crate::shared_state::SharedState;
use crate::{cs, GameContext, State};
use log::error;
use std::cell::RefCell;
use std::rc::Rc;

pub fn update_tick(
    state: &mut State,
    sim_steps: i32,
    _dimensions: (u32, u32),
    evolution_app: &mut EvolutionApp,
    world: &mut specs::World,
    shared_state: &Rc<RefCell<SharedState>>,
    update_start_time: f64,
) {
    //let mut output = ImageBuffer::new(texture_size.width, texture_size.height);
    let mut b_index = 0;

    const BUF_SIZE: usize = 50;
    let mut buf = [0u8; BUF_SIZE];
    _ = getrandom::getrandom(&mut buf);

    let one_tick_delta = 1.0 / evolution_app.simulation_steps_per_second as f64;
    for _sim_update in 0..sim_steps {
        if state.toggled {
            if let Some(mut rhai_resource) = world.get_mut::<RhaiResource>() {
                if let Some(storage) = &mut rhai_resource.storage {
                    storage
                        .scope
                        .set_value("time", f64::fract(update_start_time) /*TODO*/);
                    if let Some(ast) = &evolution_app.ast {
                        let result = storage
                            .engine
                            .eval_ast_with_scope::<()>(&mut storage.scope, ast);
                        if let Err(err) = &result {
                            evolution_app.script_error = err.to_string();
                        }
                    }
                } else {
                    error!("Warning: RhaiResource.storage is None");
                }
            } else {
                error!("Warning: RhaiResource not found in the world");
            }

            //dispatcher.dispatch(world);
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

                state.pal_container.pal[cur_v as usize].update(
                    i,
                    j,
                    cur,
                    state.diffuse_rgba.as_mut(),
                    &state.pal_container,
                    &mut state.prng,
                );
            }
        }
    }
}
