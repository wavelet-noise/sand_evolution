use crate::cs;

use super::{smoke::Smoke, void::Void, CellRegistry, CellType, PointType, Prng};

pub fn sand_falling_helper(
    my_den: i8,
    i: u16,
    j: u16,
    container: &mut [u8],
    pal_container: &CellRegistry,
    cur: usize,
    rpng: &mut Prng,
) -> bool {
    const ORDER: [[usize; 2]; 2] = [[0, 1], [1, 0]];
    let selected_order = ORDER[(rpng.next() % 2) as usize];

    let down = cs::xy_to_index(i, j - 1);
    let down_v = container[down] as usize;
    let down_c = &pal_container.pal[down_v];
    if down_c.den() < my_den && !down_c.stat() {
        container.swap(cur, down);
        return true;
    }

    for k in 0..2 {
        match selected_order[k] {
            0 => {
                let dr = cs::xy_to_index(i + 1, j - 1);
                let dr_v = container[dr] as usize;
                let dr_c = &pal_container.pal[dr_v];
                if dr_c.den() < my_den && !dr_c.stat() {
                    container.swap(cur, dr);
                    return true;
                }
            }
            1 => {
                let dl = cs::xy_to_index(i - 1, j - 1);
                let dl_v = container[dl] as usize;
                let dl_c = &pal_container.pal[dl_v];
                if dl_c.den() < my_den && !dl_c.stat() {
                    container.swap(cur, dl);
                    return true;
                }
            }
            _ => (),
        }
    }

    false
}

pub fn snow_falling_helper(
    my_den: i8,
    i: u16,
    j: u16,
    container: &mut [u8],
    pal_container: &CellRegistry,
    cur: usize,
    rpng: &mut Prng,
) -> bool {
    const ORDER: [[usize; 2]; 2] = [[0, 1], [1, 0]];
    let selected_order = ORDER[(rpng.next() % 2) as usize];

    let down = cs::xy_to_index(i, j - 1);
    let down_v = container[down] as usize;
    let down_c = &pal_container.pal[down_v];
    if down_c.den() < my_den && !down_c.stat() {
        container.swap(cur, down);
        return true;
    }

    for k in 0..2 {
        match selected_order[k] {
            0 => {
                let dr = cs::xy_to_index(i + 1, j - 1);
                let dr_v = container[dr] as usize;
                let dr_c = &pal_container.pal[dr_v];
                if dr_c.den() < my_den && !dr_c.stat() {
                    container.swap(cur, dr);
                    return true;
                }
            }
            1 => {
                let dl = cs::xy_to_index(i - 1, j - 1);
                let dl_v = container[dl] as usize;
                let dl_c = &pal_container.pal[dl_v];
                if dl_c.den() < my_den && !dl_c.stat() {
                    container.swap(cur, dl);
                    return true;
                }
            }
            _ => (),
        }
    }

    false
}

#[inline(always)]
pub fn fluid_falling_helper(
    my_den: i8,
    i: u16,
    j: u16,
    container: &mut [u8],
    pal_container: &CellRegistry,
    cur: usize,
    rpng: &mut Prng,
    thickness: u8,
) -> bool {
    const ORDER: [[usize; 2]; 2] = [[0, 1], [1, 0]];
    let selected_order = [0, 1]; //ORDER[(rpng.next() % 2) as usize];

    let down = cs::xy_to_index(i, j - 1);
    let down_v = container[down] as usize;
    let down_c = &pal_container.pal[down_v];
    if down_c.den() < my_den && !down_c.stat() {
        container.swap(cur, down);
        return true;
    }

    for k in 0..2 {
        match selected_order[k] {
            0 => {
                let dr = cs::xy_to_index(i + 1, j - 1);
                let dr_v = container[dr] as usize;
                let dr_c = &pal_container.pal[dr_v];
                if dr_c.den() < my_den && !dr_c.stat() {
                    container.swap(cur, dr);
                    return true;
                }
            }
            1 => {
                let dl = cs::xy_to_index(i - 1, j - 1);
                let dl_v = container[dl] as usize;
                let dl_c = &pal_container.pal[dl_v];
                if dl_c.den() < my_den && !dl_c.stat() {
                    container.swap(cur, dl);
                    return true;
                }
            }
            _ => (),
        }
    }

    if thickness == 1 || rpng.next() > (255 - 255 / thickness) {
        for k in 0..2 {
            match selected_order[k] {
                0 => {
                    let dr = cs::xy_to_index(i + 1, j);
                    let dr_v = container[dr] as usize;
                    let dr_c = &pal_container.pal[dr_v];
                    if dr_c.den() < my_den && !dr_c.stat() {
                        container.swap(cur, dr);
                        return true;
                    }
                }
                1 => {
                    let dl = cs::xy_to_index(i - 1, j);
                    let dl_v = container[dl] as usize;
                    let dl_c = &pal_container.pal[dl_v];
                    if dl_c.den() < my_den && !dl_c.stat() {
                        container.swap(cur, dl);
                        return true;
                    }
                }
                _ => (),
            }
        }
    }

    false
}

#[inline(always)]
pub fn fluid_flying_helper(
    my_den: i8,
    i: u16,
    j: u16,
    container: &mut [u8],
    pal_container: &CellRegistry,
    cur: usize,
    dim: &mut Prng,
) -> bool {
    const ORDER: [[usize; 2]; 2] = [[0, 1], [1, 0]];
    let selected_order = ORDER[(dim.next() % 2) as usize];

    for k in 0..2 {
        match selected_order[k] {
            0 => {
                let dr = cs::xy_to_index(i + 1, j + 1);
                let dr_v = container[dr] as usize;
                let dr_c = &pal_container.pal[dr_v];
                if dr_c.den() > my_den && !dr_c.stat() {
                    container.swap(cur, dr);
                    return true;
                }
            }
            1 => {
                let dl = cs::xy_to_index(i - 1, j + 1);
                let dl_v = container[dl] as usize;
                let dl_c = &pal_container.pal[dl_v];
                if dl_c.den() > my_den && !dl_c.stat() {
                    container.swap(cur, dl);
                    return true;
                }
            }
            _ => (),
        }
    }

    for k in 0..2 {
        match selected_order[k] {
            0 => {
                let dr = cs::xy_to_index(i + 1, j);
                let dr_v = container[dr] as usize;
                let dr_c = &pal_container.pal[dr_v];
                if dr_c.den() > my_den && !dr_c.stat() {
                    container.swap(cur, dr);
                    return true;
                }
            }
            1 => {
                let dl = cs::xy_to_index(i - 1, j);
                let dl_v = container[dl] as usize;
                let dl_c = &pal_container.pal[dl_v];
                if dl_c.den() > my_den && !dl_c.stat() {
                    container.swap(cur, dl);
                    return true;
                }
            }
            _ => (),
        }
    }

    false
}

pub fn try_spawn_smoke(
    i: PointType,
    j: PointType,
    container: &mut [CellType],
    prng: &mut Prng,
    target_count: usize,
) -> usize {
    let mut spawned = 0;
    let mut candidates = Vec::new();

    if j + 1 < cs::SECTOR_SIZE.y {
        let top = cs::xy_to_index(i, j + 1);
        if container[top] == Void::id() {
            candidates.push(top);
        }
    }
    if j > 0 {
        let bot = cs::xy_to_index(i, j - 1);
        if container[bot] == Void::id() {
            candidates.push(bot);
        }
    }
    if i + 1 < cs::SECTOR_SIZE.x {
        let right = cs::xy_to_index(i + 1, j);
        if container[right] == Void::id() {
            candidates.push(right);
        }
    }
    if i > 0 {
        let left = cs::xy_to_index(i - 1, j);
        if container[left] == Void::id() {
            candidates.push(left);
        }
    }

    while spawned < target_count && !candidates.is_empty() {
        let idx = (prng.next() as usize) % candidates.len();
        let cell_idx = candidates.remove(idx);
        container[cell_idx] = Smoke::id();
        spawned += 1;
    }

    spawned
}
