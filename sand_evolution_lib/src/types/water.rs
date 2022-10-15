use crate::cs::{self, PointType};
use super::*;

pub const fn new() -> Cell { Cell }
pub fn boxed() -> Box<Cell> { Box::new(new()) }
pub fn id() -> CellType { 2 }

pub struct Cell;
impl CellTrait for Cell {

    fn update(&self, i: PointType, j: PointType, cur: usize, container: & mut [CellType], pal_container: &Palette, prng: &mut Dim)
    {
        let down = cs::xy_to_index(i, j - 1);
        let down_v = container[down] as usize;
        let down_c = &pal_container.pal[down_v];

        let dl = cs::xy_to_index(i - 1, j - 1);
        let dl_v = container[dl] as usize;
        let dl_c = &pal_container.pal[dl_v];

        let dr = cs::xy_to_index(i + 1, j - 1);
        let dr_v = container[dr] as usize;
        let dr_c = &pal_container.pal[dr_v];

        let r = cs::xy_to_index(i + 1, j);
        let r_v = container[r] as usize;
        let r_c = &pal_container.pal[r_v];

        let l = cs::xy_to_index(i - 1, j);
        let l_v = container[l] as usize;
        let l_c = &pal_container.pal[l_v];

        if down_c.den() < self.den() && !down_c.stat()
        {
            container.swap(cur, down);
        }
        else if dr_c.den() < self.den() && !dr_c.stat()
        {
            container.swap(cur, dr);
        }
        else if dl_c.den() < self.den() && !dl_c.stat()
        {
            container.swap(cur, dl);
        }
        else if r_c.den() < self.den() && !r_c.stat()
        {
            container.swap(cur, r);
        }
        else if l_c.den() < self.den() && !l_c.stat()
        {
            container.swap(cur, l);
        }
        else if prng.next() == 0 && prng.next() == 0
        {
            container[cur] = 3;
        }
    }

    fn den(&self) -> i8 { 1 }
}