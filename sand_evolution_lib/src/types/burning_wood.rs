use crate::cs::{self, PointType};
use super::*;

pub const fn new() -> Cell { Cell }
pub fn boxed() -> Box<Cell> { Box::new(new()) }
pub fn id() -> CellType { 6 }

pub struct Cell;
impl CellTrait for Cell {

    fn update(&self, i: PointType, j: PointType, cur: usize, container: & mut [CellType], pal_container: &Palette, prng: &mut Dim)
    {
        if prng.next() > 200
        {
		    return;
        }

        if prng.next() > 250
        {
            container[cur] = burning_coal::id();
            prng.add_carb();
		    return;
        }

        let top = cs::xy_to_index(i, j + 1);
        let topl = cs::xy_to_index(i - 1, j + 1);
        let topr = cs::xy_to_index(i + 1, j + 1);
        
	    let arr = [top, topl, topr];
	    let cc = arr[(prng.next() % 3) as usize];
        let top_v = container[cc];

        if top_v == Void::id() {
            container[cc] = fire::id();
        }
    }

    fn stat(&self) -> bool { true }
}