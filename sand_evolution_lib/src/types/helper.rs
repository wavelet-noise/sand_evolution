use crate::cs;

use super::Palette;

pub fn sand_faling_helper(my_den: i8, i: u16, j: u16, container: &mut [u8], pal_container: &Palette, cur: usize) -> bool {
    let down = cs::xy_to_index(i, j - 1);
    let down_v = container[down] as usize;
    let down_c = &pal_container.pal[down_v];
    let dl = cs::xy_to_index(i - 1, j - 1);
    let dl_v = container[dl] as usize;
    let dl_c = &pal_container.pal[dl_v];
    let dr = cs::xy_to_index(i + 1, j - 1);
    let dr_v = container[dr] as usize;
    let dr_c = &pal_container.pal[dr_v];
    
    if down_c.den() < my_den && !down_c.stat()
    {
        container.swap(cur, down);
        return true;
    }

    if dr_c.den() < my_den && !dr_c.stat()
    {
        container.swap(cur, dr);
        return true;
    }

    if dl_c.den() < my_den && !dl_c.stat()
    {
        container.swap(cur, dl);
        return true;
    }

    return false;
}