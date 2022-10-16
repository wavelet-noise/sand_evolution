use crate::cs::{PointType};
use super::{*, helper::fluid_falling_helper};

pub const fn new() -> Cell { Cell }
pub fn boxed() -> Box<Cell> { Box::new(new()) }
pub fn id() -> CellType { 2 }

pub struct Cell;
impl CellTrait for Cell {

    fn update(&self, i: PointType, j: PointType, cur: usize, container: & mut [CellType], pal_container: &Palette, dim: &mut Dim)
    {
        fluid_falling_helper(self.den(), i, j, container, pal_container, cur, dim);
    }

    fn den(&self) -> i8 { 1 }
}