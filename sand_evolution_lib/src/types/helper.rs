use crate::cs;

use super::{Dim, Palette};

pub fn sand_faling_helper(
    my_den: i8,
    i: u16,
    j: u16,
    container: &mut [u8],
    pal_container: &Palette,
    cur: usize,
    rpng: &mut Dim,
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

    return false;
}

#[inline(always)]
pub fn fluid_falling_helper(
    my_den: i8,
    i: u16,
    j: u16,
    container: &mut [u8],
    pal_container: &Palette,
    cur: usize,
    rpng: &mut Dim,
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

    return false;
}

#[inline(always)]
pub fn fluid_flying_helper(
    my_den: i8,
    i: u16,
    j: u16,
    container: &mut [u8],
    pal_container: &Palette,
    cur: usize,
    rpng: &mut Dim,
) -> bool {
    const ORDER: [[usize; 2]; 2] = [[0, 1], [1, 0]];
    let selected_order = ORDER[(rpng.next() % 2) as usize];

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

    return false;
}
