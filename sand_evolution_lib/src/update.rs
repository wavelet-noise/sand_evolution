use std::cell::RefCell;
use std::rc::Rc;
use crate::{cs, State};
use crate::evolution_app::EvolutionApp;
use crate::shared_state::SharedState;

pub fn update_dim(state: &mut State, sim_steps: u8, _dimensions: (u32, u32), evolution_app: &mut EvolutionApp, event_loop_shared_state: Rc<RefCell<SharedState>>) {
    //let mut output = ImageBuffer::new(texture_size.width, texture_size.height);

    let mut b_index = 0;

    const BUF_SIZE: usize = 50;
    let mut buf = [0u8; BUF_SIZE];
    _ = getrandom::getrandom(&mut buf);

    for _sim_update in 0..sim_steps {

        if state.toggled {
            if let Some(ast) = &evolution_app.ast {
                let result = state.rhai.eval_ast_with_scope::<()>(&mut state.rhai_scope, ast);
                if let Err(err) = &result {
                    evolution_app.script_error = err.to_string();
                }
            }
        }
        for (p, c) in event_loop_shared_state.borrow_mut().points.iter() {
            if (0..cs::SECTOR_SIZE.x as i32).contains(&p.x) &&
                (0..cs::SECTOR_SIZE.y as i32).contains(&p.y) {
                state.diffuse_rgba.put_pixel(p.x as u32, p.y as u32, image::Luma([*c]));
            }
        }
        event_loop_shared_state.borrow_mut().points.clear();

        state.a ^= 1;
        if state.a == 0 {
            state.b ^= 1;
        }

        state.prng.gen();

        for i in (1..(cs::SECTOR_SIZE.x - 2 - state.a)).rev().step_by(2) {
            for j in (1..(cs::SECTOR_SIZE.y - 2 - state.b)).rev().step_by(2) {
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
