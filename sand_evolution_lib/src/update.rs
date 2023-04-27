use crate::{cs, State};

pub fn update_dim(state: &mut State, sim_steps: u8, dimensions: (u32, u32)) {
    //let mut output = ImageBuffer::new(texture_size.width, texture_size.height);

    let mut b_index = 0;

    const BUF_SIZE: usize = 50;
    let mut buf = [0u8; BUF_SIZE];
    _ = getrandom::getrandom(&mut buf);

    for _sim_update in 0..sim_steps {
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
